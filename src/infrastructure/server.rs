use bytes::{BufMut, BytesMut};
use log::{debug, error, info, warn};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::{
    application::use_cases::*,
    domain::{
        entities::Message,
        repositories::{OffsetRepository, TopicRepository},
        services::{MessageService, OffsetManagementService},
    },
    infrastructure::protocol::{
        encode_bytes, encode_i16, encode_i32, encode_i64, encode_i8, encode_string, ApiKey,
        FetchRequest, KafkaDecodable, KafkaEncodable, ProduceRequest, RequestHeader,
        ResponseHeader,
    },
};

/// Main Kafka server that handles TCP connections
pub struct KafkaServer {
    host: String,
    port: u16,
    send_message_use_case: Arc<SendMessageUseCase>,
    consume_messages_use_case: Arc<ConsumeMessagesUseCase>,
    topic_management_use_case: Arc<TopicManagementUseCase>,
}

impl KafkaServer {
    pub fn new(
        host: String,
        port: u16,
        topic_repo: Arc<dyn TopicRepository>,
        offset_repo: Arc<dyn OffsetRepository>,
    ) -> Self {
        // Create domain services
        let message_service = Arc::new(MessageService::new(topic_repo));
        let offset_service = Arc::new(OffsetManagementService::new(offset_repo));

        // Create use cases
        let send_message_use_case = Arc::new(SendMessageUseCase::new(message_service.clone()));
        let consume_messages_use_case = Arc::new(ConsumeMessagesUseCase::new(
            message_service.clone(),
            offset_service,
        ));
        let topic_management_use_case = Arc::new(TopicManagementUseCase::new(message_service));

        Self {
            host,
            port,
            send_message_use_case,
            consume_messages_use_case,
            topic_management_use_case,
        }
    }

    /// Start the server and listen for connections
    pub async fn start(&self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port)).await?;
        info!("Kafka-RS server listening on {}:{}", self.host, self.port);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New connection from: {}", addr);
                    let handler = ConnectionHandler::new(
                        stream,
                        self.send_message_use_case.clone(),
                        self.consume_messages_use_case.clone(),
                        self.topic_management_use_case.clone(),
                    );
                    tokio::spawn(async move {
                        if let Err(e) = handler.handle().await {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

/// Handles individual client connections
struct ConnectionHandler {
    stream: TcpStream,
    send_message_use_case: Arc<SendMessageUseCase>,
    consume_messages_use_case: Arc<ConsumeMessagesUseCase>,
    topic_management_use_case: Arc<TopicManagementUseCase>,
}

impl ConnectionHandler {
    fn new(
        stream: TcpStream,
        send_message_use_case: Arc<SendMessageUseCase>,
        consume_messages_use_case: Arc<ConsumeMessagesUseCase>,
        topic_management_use_case: Arc<TopicManagementUseCase>,
    ) -> Self {
        Self {
            stream,
            send_message_use_case,
            consume_messages_use_case,
            topic_management_use_case,
        }
    }

    /// Main connection handling loop
    async fn handle(mut self) -> anyhow::Result<()> {
        let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer

        loop {
            // Read message size
            let bytes_read = self.stream.read(&mut buffer[..4]).await?;
            if bytes_read == 0 {
                debug!("Client disconnected");
                break;
            }
            if bytes_read < 4 {
                warn!("Incomplete message size received");
                continue;
            }

            let message_size = i32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
            debug!("Received message of size: {} bytes", message_size);

            if message_size <= 0 || message_size > 1024 * 1024 {
                error!("Invalid message size: {}", message_size);
                break;
            }

            // Read the full message
            let mut message_data = vec![0u8; message_size as usize];
            let bytes_read = self.stream.read_exact(&mut message_data).await?;
            if bytes_read != message_size as usize {
                error!("Failed to read complete message");
                break;
            }

            // Process the message
            let mut buf = BytesMut::from(&message_data[..]);
            if let Err(e) = self.process_message(&mut buf).await {
                error!("Error processing message: {}", e);
                // Don't break the connection on processing errors
            }
        }

        Ok(())
    }

    /// Process a single Kafka protocol message
    async fn process_message(&mut self, buf: &mut BytesMut) -> anyhow::Result<()> {
        // Parse request header
        let header = RequestHeader::decode(buf)?;
        debug!(
            "Processing {} request (correlation_id: {})",
            match header.api_key {
                ApiKey::Produce => "PRODUCE",
                ApiKey::Fetch => "FETCH",
                ApiKey::Metadata => "METADATA",
                ApiKey::OffsetCommit => "OFFSET_COMMIT",
                ApiKey::OffsetFetch => "OFFSET_FETCH",
                _ => "UNKNOWN",
            },
            header.correlation_id
        );

        match header.api_key {
            ApiKey::Produce => {
                self.handle_produce_request(header, buf).await?;
            }
            ApiKey::Fetch => {
                self.handle_fetch_request(header, buf).await?;
            }
            ApiKey::Metadata => {
                self.handle_metadata_request(header, buf).await?;
            }
            ApiKey::OffsetCommit => {
                self.handle_offset_commit_request(header, buf).await?;
            }
            ApiKey::OffsetFetch => {
                self.handle_offset_fetch_request(header, buf).await?;
            }
            _ => {
                warn!("Unsupported API key: {:?}", header.api_key);
                self.send_error_response(header.correlation_id, -1).await?;
            }
        }

        Ok(())
    }

    /// Handle produce requests (send messages)
    async fn handle_produce_request(
        &mut self,
        header: RequestHeader,
        buf: &mut BytesMut,
    ) -> anyhow::Result<()> {
        match ProduceRequest::decode(buf) {
            Ok(request) => {
                debug!("Produce request for topic: {}", request.topic);

                let mut offsets = Vec::new();
                for msg in request.messages {
                    let key = msg.key.map(|k| String::from_utf8_lossy(&k).to_string());
                    let value = msg.value.unwrap_or_default();

                    match self
                        .send_message_use_case
                        .execute(request.topic.clone(), key, value)
                        .await
                    {
                        Ok(offset) => {
                            debug!("Message stored at offset: {}", offset);
                            offsets.push(offset.value());
                        }
                        Err(e) => {
                            error!("Failed to store message: {}", e);
                            self.send_error_response(header.correlation_id, -1).await?;
                            return Ok(());
                        }
                    }
                }

                self.send_produce_response(header.correlation_id, &request.topic, offsets)
                    .await?;
            }
            Err(e) => {
                error!("Failed to decode produce request: {}", e);
                self.send_error_response(header.correlation_id, -1).await?;
            }
        }

        Ok(())
    }

    /// Handle fetch requests (consume messages)
    async fn handle_fetch_request(
        &mut self,
        header: RequestHeader,
        buf: &mut BytesMut,
    ) -> anyhow::Result<()> {
        match FetchRequest::decode(buf) {
            Ok(request) => {
                debug!(
                    "Fetch request for topic: {}, offset: {}, max_bytes: {}",
                    request.topic, request.offset, request.max_bytes
                );

                // Use a simple consumer ID based on connection
                let consumer_id = format!("consumer-{}", header.correlation_id);

                match self
                    .consume_messages_use_case
                    .execute(consumer_id, request.topic.clone(), 100) // Max 100 messages
                    .await
                {
                    Ok(messages) => {
                        debug!("Retrieved {} messages", messages.len());
                        self.send_fetch_response(
                            header.correlation_id,
                            &request.topic,
                            messages,
                            request.offset as u64,
                        )
                        .await?;
                    }
                    Err(e) => {
                        error!("Failed to fetch messages: {}", e);
                        self.send_error_response(header.correlation_id, -1).await?;
                    }
                }
            }
            Err(e) => {
                error!("Failed to decode fetch request: {}", e);
                self.send_error_response(header.correlation_id, -1).await?;
            }
        }

        Ok(())
    }

    /// Handle metadata requests
    async fn handle_metadata_request(
        &mut self,
        header: RequestHeader,
        _buf: &mut BytesMut,
    ) -> anyhow::Result<()> {
        debug!("Metadata request");

        match self.topic_management_use_case.list_topics().await {
            Ok(topics) => {
                self.send_metadata_response(header.correlation_id, topics)
                    .await?;
            }
            Err(e) => {
                error!("Failed to list topics: {}", e);
                self.send_error_response(header.correlation_id, -1).await?;
            }
        }

        Ok(())
    }

    /// Handle offset commit requests
    async fn handle_offset_commit_request(
        &mut self,
        header: RequestHeader,
        _buf: &mut BytesMut,
    ) -> anyhow::Result<()> {
        debug!("Offset commit request");
        // For now, just send success response
        self.send_offset_commit_response(header.correlation_id)
            .await?;
        Ok(())
    }

    /// Handle offset fetch requests
    async fn handle_offset_fetch_request(
        &mut self,
        header: RequestHeader,
        _buf: &mut BytesMut,
    ) -> anyhow::Result<()> {
        debug!("Offset fetch request");
        // For now, just send empty response
        self.send_offset_fetch_response(header.correlation_id)
            .await?;
        Ok(())
    }

    /// Send produce response
    async fn send_produce_response(
        &mut self,
        correlation_id: i32,
        topic: &str,
        offsets: Vec<u64>,
    ) -> anyhow::Result<()> {
        let mut response = BytesMut::new();

        // Response header
        let header = ResponseHeader { correlation_id };
        header.encode(&mut response)?;

        // Throttle time
        encode_i32(&mut response, 0);

        // Topic responses array
        encode_i32(&mut response, 1); // One topic

        // Topic name
        encode_string(&mut response, Some(topic))?;

        // Partition responses array
        encode_i32(&mut response, 1); // One partition

        // Partition index
        encode_i32(&mut response, 0);

        // Error code (no error)
        encode_i16(&mut response, 0);

        // Base offset
        encode_i64(&mut response, offsets.first().copied().unwrap_or(0) as i64);

        // Log append time
        encode_i64(&mut response, -1);

        // Log start offset
        encode_i64(&mut response, 0);

        self.send_response(response).await
    }

    /// Send fetch response
    async fn send_fetch_response(
        &mut self,
        correlation_id: i32,
        topic: &str,
        messages: Vec<Message>,
        start_offset: u64,
    ) -> anyhow::Result<()> {
        let mut response = BytesMut::new();

        // Response header
        let header = ResponseHeader { correlation_id };
        header.encode(&mut response)?;

        // Throttle time
        encode_i32(&mut response, 0);

        // Error code
        encode_i16(&mut response, 0);

        // Session ID
        encode_i32(&mut response, 0);

        // Topic responses
        encode_i32(&mut response, 1); // One topic

        // Topic name
        encode_string(&mut response, Some(topic))?;

        // Partition responses
        encode_i32(&mut response, 1); // One partition

        // Partition index
        encode_i32(&mut response, 0);

        // Error code
        encode_i16(&mut response, 0);

        // High watermark
        encode_i64(&mut response, start_offset as i64 + messages.len() as i64);

        // Last stable offset
        encode_i64(&mut response, -1);

        // Log start offset
        encode_i64(&mut response, 0);

        // Aborted transactions (empty)
        encode_i32(&mut response, 0);

        // Preferred read replica
        encode_i32(&mut response, -1);

        // Records (simplified)
        if messages.is_empty() {
            encode_i32(&mut response, 0); // Empty record set
        } else {
            // Simple message encoding
            let mut records = BytesMut::new();
            for (i, message) in messages.iter().enumerate() {
                // Offset
                encode_i64(&mut records, start_offset as i64 + i as i64);

                // Message size calculation
                let key_size = message.key.as_ref().map(|k| k.len()).unwrap_or(0);
                let value_size = message.value.len();
                let message_size = 4 + 1 + 1 + 8 + 4 + key_size + 4 + value_size;

                encode_i32(&mut records, message_size as i32);

                // CRC (dummy)
                encode_i32(&mut records, 0);

                // Magic byte
                encode_i8(&mut records, 1);

                // Attributes
                encode_i8(&mut records, 0);

                // Timestamp
                encode_i64(&mut records, message.timestamp.timestamp_millis());

                // Key
                encode_bytes(&mut records, message.key.as_ref().map(|k| k.as_bytes()))?;

                // Value
                encode_bytes(&mut records, Some(&message.value))?;
            }

            encode_i32(&mut response, records.len() as i32);
            response.extend_from_slice(&records);
        }

        self.send_response(response).await
    }

    /// Send metadata response
    async fn send_metadata_response(
        &mut self,
        correlation_id: i32,
        topics: Vec<String>,
    ) -> anyhow::Result<()> {
        let mut response = BytesMut::new();

        // Response header
        let header = ResponseHeader { correlation_id };
        header.encode(&mut response)?;

        // Throttle time
        encode_i32(&mut response, 0);

        // Brokers
        encode_i32(&mut response, 1); // One broker (ourselves)

        // Broker ID
        encode_i32(&mut response, 0);

        // Host
        encode_string(&mut response, Some("localhost"))?;

        // Port
        encode_i32(&mut response, self.stream.local_addr()?.port() as i32);

        // Rack (nullable)
        encode_string(&mut response, None)?;

        // Cluster ID (nullable)
        encode_string(&mut response, Some("kafka-rs-cluster"))?;

        // Controller ID
        encode_i32(&mut response, 0);

        // Topics
        encode_i32(&mut response, topics.len() as i32);

        for topic in topics {
            // Error code
            encode_i16(&mut response, 0);

            // Topic name
            encode_string(&mut response, Some(&topic))?;

            // Is internal
            encode_i8(&mut response, 0);

            // Partitions
            encode_i32(&mut response, 1); // One partition per topic

            // Error code
            encode_i16(&mut response, 0);

            // Partition index
            encode_i32(&mut response, 0);

            // Leader
            encode_i32(&mut response, 0);

            // Leader epoch
            encode_i32(&mut response, 0);

            // Replicas
            encode_i32(&mut response, 1);
            encode_i32(&mut response, 0);

            // ISR
            encode_i32(&mut response, 1);
            encode_i32(&mut response, 0);

            // Offline replicas
            encode_i32(&mut response, 0);
        }

        self.send_response(response).await
    }

    /// Send offset commit response
    async fn send_offset_commit_response(&mut self, correlation_id: i32) -> anyhow::Result<()> {
        let mut response = BytesMut::new();

        // Response header
        let header = ResponseHeader { correlation_id };
        header.encode(&mut response)?;

        // Throttle time
        encode_i32(&mut response, 0);

        // Topics (empty for now)
        encode_i32(&mut response, 0);

        self.send_response(response).await
    }

    /// Send offset fetch response
    async fn send_offset_fetch_response(&mut self, correlation_id: i32) -> anyhow::Result<()> {
        let mut response = BytesMut::new();

        // Response header
        let header = ResponseHeader { correlation_id };
        header.encode(&mut response)?;

        // Throttle time
        encode_i32(&mut response, 0);

        // Topics (empty for now)
        encode_i32(&mut response, 0);

        // Error code
        encode_i16(&mut response, 0);

        self.send_response(response).await
    }

    /// Send error response
    async fn send_error_response(
        &mut self,
        correlation_id: i32,
        error_code: i16,
    ) -> anyhow::Result<()> {
        let mut response = BytesMut::new();

        // Response header
        let header = ResponseHeader { correlation_id };
        header.encode(&mut response)?;

        // Error code
        encode_i16(&mut response, error_code);

        self.send_response(response).await
    }

    /// Send response to client
    async fn send_response(&mut self, response: BytesMut) -> anyhow::Result<()> {
        // Prepend message size
        let size = response.len() as i32;
        let mut full_response = BytesMut::new();
        encode_i32(&mut full_response, size);
        full_response.extend_from_slice(&response);

        self.stream.write_all(&full_response).await?;
        self.stream.flush().await?;

        debug!("Sent response ({} bytes)", full_response.len());
        Ok(())
    }
}
