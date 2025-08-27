use bytes::{Buf, BytesMut};
use crc32fast::Hasher as Crc32Hasher;
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
        decode_i32, decode_i64, decode_string, encode_bytes, encode_i16, encode_i32, encode_i64,
        encode_i8, encode_string, ApiKey, ApiVersionsRequest, ApiVersionsResponse, FetchRequest,
        KafkaDecodable, KafkaEncodable, ProduceRequest, RequestHeader, ResponseHeader,
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
    subscribed_topics: Vec<String>, // Track consumer subscriptions
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
            subscribed_topics: Vec::new(),
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
                ApiKey::ListOffsets => "LIST_OFFSETS",
                ApiKey::Metadata => "METADATA",
                ApiKey::OffsetCommit => "OFFSET_COMMIT",
                ApiKey::OffsetFetch => "OFFSET_FETCH",
                ApiKey::FindCoordinator => "FIND_COORDINATOR",
                ApiKey::JoinGroup => "JOIN_GROUP",
                ApiKey::Heartbeat => "HEARTBEAT",
                ApiKey::LeaveGroup => "LEAVE_GROUP",
                ApiKey::SyncGroup => "SYNC_GROUP",
                ApiKey::ApiVersions => "API_VERSIONS",
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
            ApiKey::ListOffsets => {
                self.handle_list_offsets_request(header, buf).await?;
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
            ApiKey::ApiVersions => {
                self.handle_api_versions_request(header, buf).await?;
            }
            ApiKey::FindCoordinator => {
                self.handle_find_coordinator_request(header, buf).await?;
            }
            ApiKey::JoinGroup => {
                self.handle_join_group_request(header, buf).await?;
            }
            ApiKey::SyncGroup => {
                self.handle_sync_group_request(header, buf).await?;
            }
            ApiKey::Heartbeat => {
                self.handle_heartbeat_request(header, buf).await?;
            }
            ApiKey::LeaveGroup => {
                self.handle_leave_group_request(header, buf).await?;
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
        let fetch_req = if header.api_version <= 1 {
            self.decode_fetch_request_v1(buf)
        } else {
            FetchRequest::decode(buf).map_err(|e| anyhow::anyhow!(e))
        };
        match fetch_req {
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
                            header.api_version,
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

    /// Decode Fetch v1 (and v0) request shape
    fn decode_fetch_request_v1(&mut self, buf: &mut BytesMut) -> anyhow::Result<FetchRequest> {
        let replica_id = decode_i32(buf)?;
        let max_wait = decode_i32(buf)?;
        let min_bytes = decode_i32(buf)?;

        let topic_count = decode_i32(buf)?;
        if topic_count != 1 {
            return Err(anyhow::anyhow!("Expected exactly one topic in Fetch v1"));
        }
        let topic = decode_string(buf)?
            .ok_or_else(|| anyhow::anyhow!("Topic cannot be null in Fetch v1"))?;

        let part_count = decode_i32(buf)?;
        if part_count != 1 {
            return Err(anyhow::anyhow!(
                "Expected exactly one partition in Fetch v1"
            ));
        }
        let partition = decode_i32(buf)?;
        let offset = decode_i64(buf)?;
        let max_bytes = decode_i32(buf)?;

        Ok(FetchRequest {
            replica_id,
            max_wait,
            min_bytes,
            topic,
            partition,
            offset,
            max_bytes,
        })
    }

    /// Decode metadata request to extract requested topics
    fn decode_metadata_request(
        &mut self,
        buf: &mut BytesMut,
    ) -> anyhow::Result<Option<Vec<String>>> {
        debug!(
            "Decoding metadata request, buffer size: {}",
            buf.remaining()
        );

        if buf.remaining() < 4 {
            debug!("Buffer too small for topics count");
            return Ok(None);
        }

        // Read topics array length
        let topics_count = decode_i32(buf)?;
        debug!("Metadata request has {} topics", topics_count);

        if topics_count == -1 {
            // Null array means all topics
            debug!("Null topics array - requesting all topics");
            return Ok(None);
        }

        if topics_count == 0 {
            // Empty array means all topics
            debug!("Empty topics array - requesting all topics");
            return Ok(None);
        }

        let mut topics = Vec::new();
        for i in 0..topics_count {
            debug!("Decoding topic {} of {}", i + 1, topics_count);
            if let Some(topic) = decode_string(buf)? {
                debug!("Decoded topic: {}", topic);
                topics.push(topic);
            } else {
                debug!("Failed to decode topic {}", i + 1);
            }
        }

        debug!("Successfully decoded {} topics: {:?}", topics.len(), topics);
        Ok(Some(topics))
    }

    /// Decode LIST_OFFSETS request to extract topic and timestamp
    async fn decode_list_offsets_request(
        &mut self,
        buf: &mut BytesMut,
    ) -> anyhow::Result<Option<(String, i64)>> {
        debug!(
            "Decoding LIST_OFFSETS request, buffer size: {}",
            buf.remaining()
        );

        if buf.remaining() < 4 {
            debug!("Buffer too small for replica_id");
            return Ok(None);
        }

        // Skip replica_id (int32)
        let _replica_id = decode_i32(buf)?;

        if buf.remaining() < 4 {
            debug!("Buffer too small for isolation_level");
            return Ok(None);
        }

        // Skip isolation_level (int8, but aligned as int32 in older versions)
        let _isolation_level = decode_i32(buf)?;

        if buf.remaining() < 4 {
            debug!("Buffer too small for topics array");
            return Ok(None);
        }

        // Read topics array length
        let topics_count = decode_i32(buf)?;
        debug!("LIST_OFFSETS request has {} topics", topics_count);

        if topics_count <= 0 {
            return Ok(None);
        }

        // Decode first topic
        if let Some(topic) = decode_string(buf)? {
            debug!("Decoded LIST_OFFSETS topic: {}", topic);

            // Read partitions array
            if buf.remaining() >= 4 {
                let partitions_count = decode_i32(buf)?;
                debug!("Topic has {} partitions", partitions_count);

                if partitions_count > 0 && buf.remaining() >= 12 {
                    // Skip partition index (int32)
                    let _partition = decode_i32(buf)?;

                    // Read timestamp (int64) - this is what we need
                    let timestamp = decode_i64(buf)?;
                    debug!("Decoded timestamp: {} (-2=earliest, -1=latest)", timestamp);

                    return Ok(Some((topic, timestamp)));
                }
            }

            // Fallback if partition data is incomplete
            return Ok(Some((topic, -2))); // Default to earliest
        }

        Ok(None)
    }

    /// Handle metadata requests
    async fn handle_metadata_request(
        &mut self,
        header: RequestHeader,
        buf: &mut BytesMut,
    ) -> anyhow::Result<()> {
        debug!("Metadata request for API version {}", header.api_version);

        // Try to decode the metadata request to see what topics are requested
        let requested_topics = self.decode_metadata_request(buf)?;
        debug!("Requested topics: {:?}", requested_topics);

        // Store subscribed topics for later use in consumer group assignments
        if let Some(ref topics) = requested_topics {
            self.subscribed_topics = topics.clone();
            debug!("Updated subscribed topics: {:?}", self.subscribed_topics);
        }

        match self.topic_management_use_case.list_topics().await {
            Ok(mut topics) => {
                // If specific topics were requested, create them if they don't exist
                if let Some(requested) = requested_topics {
                    for requested_topic in requested {
                        if !topics.contains(&requested_topic) {
                            debug!("Auto-creating topic: {}", requested_topic);
                            // Actually create the topic, not just add it to the response
                            match self
                                .topic_management_use_case
                                .create_topic(requested_topic.clone())
                                .await
                            {
                                Ok(_) => {
                                    debug!("Successfully created topic: {}", requested_topic);
                                    topics.push(requested_topic);
                                }
                                Err(e) => {
                                    warn!("Failed to create topic {}: {}", requested_topic, e);
                                    // Still add to response to avoid client errors
                                    topics.push(requested_topic);
                                }
                            }
                        }
                    }
                }

                debug!("Found {} topics: {:?}", topics.len(), topics);
                self.send_metadata_response(header.correlation_id, header.api_version, topics)
                    .await?;
            }
            Err(e) => {
                error!("Failed to list topics: {}", e);
                self.send_error_response(header.correlation_id, -1).await?;
            }
        }

        Ok(())
    }

    /// Handle find coordinator requests
    async fn handle_find_coordinator_request(
        &mut self,
        header: RequestHeader,
        _buf: &mut BytesMut,
    ) -> anyhow::Result<()> {
        debug!("Find coordinator request");
        self.send_find_coordinator_response(header.correlation_id)
            .await?;
        Ok(())
    }

    /// Handle join group requests for consumer group coordination
    async fn handle_join_group_request(
        &mut self,
        header: RequestHeader,
        _buf: &mut BytesMut,
    ) -> anyhow::Result<()> {
        debug!("Join group request");
        self.send_join_group_response(header.correlation_id).await?;
        Ok(())
    }

    /// Handle sync group requests for consumer group coordination
    async fn handle_sync_group_request(
        &mut self,
        header: RequestHeader,
        _buf: &mut BytesMut,
    ) -> anyhow::Result<()> {
        debug!("Sync group request");
        self.send_sync_group_response(header.correlation_id).await?;
        Ok(())
    }

    /// Handle heartbeat requests for consumer group coordination
    async fn handle_heartbeat_request(
        &mut self,
        header: RequestHeader,
        _buf: &mut BytesMut,
    ) -> anyhow::Result<()> {
        debug!("Heartbeat request");
        self.send_heartbeat_response(header.correlation_id).await?;
        Ok(())
    }

    /// Handle leave group requests for consumer group coordination
    async fn handle_leave_group_request(
        &mut self,
        header: RequestHeader,
        _buf: &mut BytesMut,
    ) -> anyhow::Result<()> {
        debug!("Leave group request");
        self.send_leave_group_response(header.correlation_id)
            .await?;
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

    /// Handle API versions requests - tells clients what APIs we support
    async fn handle_api_versions_request(
        &mut self,
        header: RequestHeader,
        buf: &mut BytesMut,
    ) -> anyhow::Result<()> {
        debug!("API versions request");

        // Try to decode the request (it might be empty for older clients)
        let _request = match ApiVersionsRequest::decode(buf) {
            Ok(req) => Some(req),
            Err(_) => {
                debug!("Failed to decode ApiVersions request, using defaults");
                None
            }
        };

        self.send_api_versions_response(header.correlation_id)
            .await?;
        Ok(())
    }

    /// Handle list offsets requests
    async fn handle_list_offsets_request(
        &mut self,
        header: RequestHeader,
        buf: &mut BytesMut,
    ) -> anyhow::Result<()> {
        debug!("List offsets request - decoding request");

        // Decode the LIST_OFFSETS request to get the requested topic and timestamp
        let result = self.decode_list_offsets_request(buf).await;
        let (topic, timestamp) = match result {
            Ok(Some((t, ts))) => (t, ts),
            Ok(None) => {
                debug!("No topics requested, using default");
                ("integration-test-topic".to_string(), -2) // -2 = earliest
            }
            Err(e) => {
                warn!(
                    "Failed to decode LIST_OFFSETS request: {}, using defaults",
                    e
                );
                ("integration-test-topic".to_string(), -2)
            }
        };

        debug!(
            "LIST_OFFSETS request for topic: {}, timestamp: {}",
            topic, timestamp
        );

        // Get the actual message count for this topic to return accurate offset
        let message_count = match self.topic_management_use_case.list_topics().await {
            Ok(topics) if topics.contains(&topic) => {
                // For now, return 1 to indicate we have messages
                // In a real implementation, we'd query the actual message count
                debug!("Topic {} exists, returning message count", topic);
                1u64
            }
            _ => {
                debug!("Topic {} doesn't exist or error, returning 0", topic);
                0u64
            }
        };

        self.send_list_offsets_response_for_topic(
            header.correlation_id,
            &topic,
            message_count,
            timestamp,
        )
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

        // Throttle time (v1+)
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

        // Log append time (v2+)
        encode_i64(&mut response, -1);

        // Log start offset (v5+, but KafkaJS might expect it in v3)
        encode_i64(&mut response, 0);

        // Record errors array (v8+ but might be expected in v3 by some clients)
        encode_i32(&mut response, 0); // No record errors

        // Error message (v8+ nullable string, send empty for compatibility)
        encode_string(&mut response, None)?;

        debug!(
            "Sending PRODUCE response: {} bytes, correlation_id: {}, topic: {}, offset: {:?}",
            response.len(),
            correlation_id,
            topic,
            offsets.first()
        );

        self.send_response(response).await
    }

    /// Send find coordinator response
    async fn send_find_coordinator_response(&mut self, correlation_id: i32) -> anyhow::Result<()> {
        let mut response = BytesMut::new();

        // Response header
        let header = ResponseHeader { correlation_id };
        header.encode(&mut response)?;

        // Throttle time (v1+)
        encode_i32(&mut response, 0);

        // Error code (0 = no error)
        encode_i16(&mut response, 0);

        // Error message (nullable string)
        encode_string(&mut response, None)?;

        // Node ID (coordinator broker ID)
        encode_i32(&mut response, 0);

        // Host (coordinator host)
        encode_string(&mut response, Some("127.0.0.1"))?;

        // Port (coordinator port)
        encode_i32(&mut response, 9092);

        debug!(
            "Sending FindCoordinator response: {} bytes, correlation_id: {}",
            response.len(),
            correlation_id
        );

        self.send_response(response).await
    }

    /// Send join group response for consumer group coordination
    async fn send_join_group_response(&mut self, correlation_id: i32) -> anyhow::Result<()> {
        let mut response = BytesMut::new();

        // Response header
        let header = ResponseHeader { correlation_id };
        header.encode(&mut response)?;

        // Throttle time (v2+)
        encode_i32(&mut response, 0);

        // Error code (0 = no error)
        encode_i16(&mut response, 0);

        // Generation ID
        encode_i32(&mut response, 1);

        // Protocol name (use standard Kafka protocol name that KafkaJS recognizes)
        encode_string(&mut response, Some("RoundRobinAssigner"))?;

        // Leader ID (make this member the leader for simplicity)
        let member_id = "consumer-1";
        encode_string(&mut response, Some(member_id))?;

        // Member ID (same as leader for single member)
        encode_string(&mut response, Some(member_id))?;

        // Members array
        encode_i32(&mut response, 1); // One member

        // Member ID
        encode_string(&mut response, Some(member_id))?;

        // Member metadata (empty for basic implementation)
        encode_i32(&mut response, 0); // Empty bytes

        debug!(
            "Sending JoinGroup response: {} bytes, correlation_id: {}",
            response.len(),
            correlation_id
        );

        self.send_response(response).await
    }

    /// Send sync group response for consumer group coordination
    async fn send_sync_group_response(&mut self, correlation_id: i32) -> anyhow::Result<()> {
        let mut response = BytesMut::new();

        // Response header
        let header = ResponseHeader { correlation_id };
        header.encode(&mut response)?;

        // Throttle time (v1+)
        encode_i32(&mut response, 0);

        // Error code (0 = no error)
        encode_i16(&mut response, 0);

        // Assignment (member assignment, serialize a proper ConsumerProtocolAssignment)
        // Based on Apache Kafka ConsumerProtocol.serializeAssignment format:
        // MessageUtil.toVersionPrefixedByteBuffer(version, ConsumerProtocolAssignment)
        let mut assignment = BytesMut::new();

        // Version prefix (int16) - as done by MessageUtil.toVersionPrefixedByteBuffer
        encode_i16(&mut assignment, 0); // version 0

        // ConsumerProtocolAssignment format:
        // assigned_partitions: [TopicPartition]
        // - topic: string
        // - partitions: [int32]
        // user_data: bytes (nullable)

        // Array of assigned topic partitions (int32 count)
        encode_i32(&mut assignment, 1); // One topic partition entry

        // Use the first subscribed topic, or fallback if none
        let topic_to_assign = self
            .subscribed_topics
            .first()
            .cloned()
            .unwrap_or_else(|| "integration-test-topic".to_string());

        // TopicPartition entry:
        // Topic name (string)
        encode_string(&mut assignment, Some(&topic_to_assign))?;

        // Partitions array for this topic (int32 count + partition IDs)
        encode_i32(&mut assignment, 1); // One partition
        encode_i32(&mut assignment, 0); // Partition 0

        // User data (bytes, nullable) - null for now
        encode_i32(&mut assignment, -1); // -1 indicates null

        debug!(
            "Partition assignment: topic={}, partitions=[0]",
            topic_to_assign
        );

        // Encode assignment as bytes
        encode_i32(&mut response, assignment.len() as i32);
        response.extend_from_slice(&assignment);

        debug!(
            "Sending SyncGroup response: {} bytes, correlation_id: {}",
            response.len(),
            correlation_id
        );

        debug!("Assignment data (hex): {:02x?}", assignment.as_ref());
        debug!("Assignment data (len): {} bytes", assignment.len());

        self.send_response(response).await
    }

    /// Send heartbeat response for consumer group coordination
    async fn send_heartbeat_response(&mut self, correlation_id: i32) -> anyhow::Result<()> {
        let mut response = BytesMut::new();

        // Response header
        let header = ResponseHeader { correlation_id };
        header.encode(&mut response)?;

        // Throttle time (v1+)
        encode_i32(&mut response, 0);

        // Error code (0 = no error)
        encode_i16(&mut response, 0);

        debug!(
            "Sending Heartbeat response: {} bytes, correlation_id: {}",
            response.len(),
            correlation_id
        );

        self.send_response(response).await
    }

    /// Send leave group response for consumer group coordination
    async fn send_leave_group_response(&mut self, correlation_id: i32) -> anyhow::Result<()> {
        let mut response = BytesMut::new();

        // Response header
        let header = ResponseHeader { correlation_id };
        header.encode(&mut response)?;

        // Throttle time (v1+)
        encode_i32(&mut response, 0);

        // Error code (0 = no error)
        encode_i16(&mut response, 0);

        debug!(
            "Sending LeaveGroup response: {} bytes, correlation_id: {}",
            response.len(),
            correlation_id
        );

        self.send_response(response).await
    }

    /// Send fetch response
    async fn send_fetch_response(
        &mut self,
        correlation_id: i32,
        api_version: i16,
        topic: &str,
        messages: Vec<Message>,
        start_offset: u64,
    ) -> anyhow::Result<()> {
        // If the client negotiated an old Fetch version (<=1), return a v1-style response
        if api_version <= 1 {
            let mut response = BytesMut::new();

            // Response header
            let header = ResponseHeader { correlation_id };
            header.encode(&mut response)?;

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

            // Build MessageSet (old format expected by Fetch v1)
            let mut message_set = BytesMut::new();
            for (i, message) in messages.iter().enumerate() {
                // Message offset within the set
                encode_i64(&mut message_set, start_offset as i64 + i as i64);

                // Build message body for magic v1 (with timestamp) into a temp buffer
                let mut msg_body = BytesMut::new();
                // Magic v1
                encode_i8(&mut msg_body, 1);
                // Attributes
                encode_i8(&mut msg_body, 0);
                // Timestamp (ms)
                encode_i64(&mut msg_body, message.timestamp.timestamp_millis());
                // Key
                if let Some(ref key) = message.key {
                    encode_bytes(&mut msg_body, Some(key.as_bytes()))?;
                } else {
                    encode_bytes(&mut msg_body, None)?;
                }
                // Value
                encode_bytes(&mut msg_body, Some(&message.value))?;

                // CRC32 over msg_body
                let mut hasher = Crc32Hasher::new();
                hasher.update(&msg_body);
                let crc = hasher.finalize();

                // message_size = 4 (CRC) + msg_body length
                let msg_size = 4 + msg_body.len();
                encode_i32(&mut message_set, msg_size as i32);
                encode_i32(&mut message_set, crc as i32);
                message_set.extend_from_slice(&msg_body);
            }

            // message_set_size then message_set bytes
            encode_i32(&mut response, message_set.len() as i32);
            response.extend_from_slice(&message_set);

            return self.send_response(response).await;
        }

        // Default to a simplified v4-like response (may not be fully compatible with all clients)
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

        // Records (simplified empty set for now)
        encode_i32(&mut response, 0);

        self.send_response(response).await
    }

    /// Send metadata response
    async fn send_metadata_response(
        &mut self,
        correlation_id: i32,
        api_version: i16,
        topics: Vec<String>,
    ) -> anyhow::Result<()> {
        debug!(
            "Sending metadata response v{} with {} topics",
            api_version,
            topics.len()
        );

        let mut response = BytesMut::new();

        // Response header
        let header = ResponseHeader { correlation_id };
        header.encode(&mut response)?;

        // NO throttle_time in metadata responses!

        // Brokers array
        encode_i32(&mut response, 1); // One broker (ourselves)
        {
            // Broker ID
            encode_i32(&mut response, 0);

            // Host
            encode_string(&mut response, Some("localhost"))?;

            // Port
            encode_i32(&mut response, self.stream.local_addr()?.port() as i32);

            // For Metadata v1+ we include rack
            if api_version >= 1 {
                encode_string(&mut response, None)?; // Rack (nullable)
            }
        }

        // For Metadata v2+ we include cluster ID
        if api_version >= 2 {
            encode_string(&mut response, Some("test-cluster"))?; // Cluster ID
        }

        // For Metadata v1+ we include controller ID
        if api_version >= 1 {
            encode_i32(&mut response, 0); // Controller ID
        }

        // Topics array - this is where the parsing error occurs
        debug!("Encoding {} topics", topics.len());
        encode_i32(&mut response, topics.len() as i32);

        for (i, topic) in topics.iter().enumerate() {
            debug!("Encoding topic {}: {}", i, topic);

            // Topic error code
            encode_i16(&mut response, 0);

            // Topic name
            encode_string(&mut response, Some(topic))?;

            // For Metadata v1+ we include is_internal flag
            if api_version >= 1 {
                encode_i8(&mut response, 0); // Is internal (false)
            }

            // Partitions array length
            encode_i32(&mut response, 1); // One partition per topic

            // Partition info
            {
                // Partition error code
                encode_i16(&mut response, 0);

                // Partition index
                encode_i32(&mut response, 0);

                // Leader
                encode_i32(&mut response, 0);

                // Replicas array
                encode_i32(&mut response, 1);
                encode_i32(&mut response, 0);

                // ISR array
                encode_i32(&mut response, 1);
                encode_i32(&mut response, 0);
            }
        }

        debug!("Final response size: {} bytes", response.len());
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

        // Topics array (1 topic)
        encode_i32(&mut response, 1);

        // Topic name - use the first subscribed topic or default
        let default_topic = "integration-test-topic".to_string();
        let topic_name = self.subscribed_topics.first().unwrap_or(&default_topic);
        encode_string(&mut response, Some(topic_name))?;

        // Partitions array (1 partition)
        encode_i32(&mut response, 1);

        // Partition index
        encode_i32(&mut response, 0);

        // Committed offset (-1 means no committed offset, consumer should use auto.offset.reset)
        // Return -1 to indicate no committed offset for new consumer groups
        encode_i64(&mut response, -1);

        // Leader epoch (-1 means no epoch)
        encode_i32(&mut response, -1);

        // Metadata (empty)
        encode_string(&mut response, None)?;

        // Error code (0 = no error)
        encode_i16(&mut response, 0);

        // Global error code (v8+, but include for compatibility)
        encode_i16(&mut response, 0);

        debug!(
            "Sending OffsetFetch response: {} bytes, correlation_id: {}, telling consumer no committed offset (-1)",
            response.len(),
            correlation_id
        );

        self.send_response(response).await
    }

    /// Send list offsets response
    async fn send_list_offsets_response(&mut self, correlation_id: i32) -> anyhow::Result<()> {
        let mut response = BytesMut::new();

        // Response header
        let header = ResponseHeader { correlation_id };
        header.encode(&mut response)?;

        // Throttle time
        encode_i32(&mut response, 0);

        // Topics array (1 topic)
        encode_i32(&mut response, 1);

        // Topic name - use the first subscribed topic or default
        let default_topic = "integration-test-topic".to_string();
        let topic_name = self.subscribed_topics.first().unwrap_or(&default_topic);
        encode_string(&mut response, Some(topic_name))?;

        // Partitions array (1 partition)
        encode_i32(&mut response, 1);

        // Partition index
        encode_i32(&mut response, 0);

        // Error code (0 = no error)
        encode_i16(&mut response, 0);

        // Timestamp (-1 for unknown)
        encode_i64(&mut response, -1);

        // Offset (high water mark - next offset to write)
        // If we have 1 message at offset 0, high water mark is 1
        encode_i64(&mut response, 1);

        debug!(
            "Sending ListOffsets response: {} bytes, correlation_id: {}, high water mark: 1",
            response.len(),
            correlation_id
        );

        self.send_response(response).await
    }

    /// Send LIST_OFFSETS response for a specific topic with accurate offset information
    async fn send_list_offsets_response_for_topic(
        &mut self,
        correlation_id: i32,
        topic: &str,
        message_count: u64,
        timestamp: i64,
    ) -> anyhow::Result<()> {
        let mut response = BytesMut::new();

        // Response header
        let header = ResponseHeader { correlation_id };
        header.encode(&mut response)?;

        // Throttle time
        encode_i32(&mut response, 0);

        // Topics array (1 topic)
        encode_i32(&mut response, 1);

        // Topic name
        encode_string(&mut response, Some(topic))?;

        // Partitions array (1 partition)
        encode_i32(&mut response, 1);

        // Partition index
        encode_i32(&mut response, 0);

        // Error code (0 = no error)
        encode_i16(&mut response, 0);

        // Timestamp (-1 for unknown)
        encode_i64(&mut response, -1);

        // Offset based on timestamp request:
        // -2 (earliest): return 0
        // -1 (latest): return high water mark (message_count)
        // other: return high water mark for simplicity
        let offset = if timestamp == -2 {
            0i64 // Start from beginning
        } else {
            message_count as i64 // High water mark (latest)
        };

        encode_i64(&mut response, offset);

        debug!(
            "Sending LIST_OFFSETS response: {} bytes, correlation_id: {}, topic: {}, timestamp: {}, offset: {}, message_count: {}",
            response.len(),
            correlation_id,
            topic,
            timestamp,
            offset,
            message_count
        );

        self.send_response(response).await
    }

    /// Send API versions response
    async fn send_api_versions_response(&mut self, correlation_id: i32) -> anyhow::Result<()> {
        let mut response = BytesMut::new();

        // Response header
        let header = ResponseHeader { correlation_id };
        header.encode(&mut response)?;

        // Create and encode API versions response
        let api_versions_response = ApiVersionsResponse::new();
        api_versions_response.encode(&mut response);

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
