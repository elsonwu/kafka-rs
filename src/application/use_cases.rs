use std::sync::Arc;
use log::{info, debug};

use crate::domain::{
    entities::*,
    value_objects::*,
    services::*,
    errors::*,
};

/// Use case for sending messages to topics
pub struct SendMessageUseCase {
    message_service: Arc<MessageService>,
}

impl SendMessageUseCase {
    pub fn new(message_service: Arc<MessageService>) -> Self {
        Self { message_service }
    }

    /// Send a single message to a topic
    pub async fn execute(
        &self,
        topic_name: String,
        key: Option<String>,
        value: Vec<u8>,
    ) -> Result<Offset> {
        let topic_name = TopicName::new(topic_name)?;
        let message = Message::new(key, value);

        info!("Sending message to topic: {}", topic_name);
        debug!("Message size: {} bytes", message.size());

        self.message_service
            .send_message(topic_name, message)
            .await
    }

    /// Send multiple messages to a topic
    pub async fn execute_batch(
        &self,
        topic_name: String,
        messages: Vec<(Option<String>, Vec<u8>)>,
    ) -> Result<Vec<Offset>> {
        let topic_name = TopicName::new(topic_name)?;
        let mut offsets = Vec::new();

        info!("Sending {} messages to topic: {}", messages.len(), topic_name);

        for (key, value) in messages {
            let message = Message::new(key, value);
            let offset = self.message_service
                .send_message(topic_name.clone(), message)
                .await?;
            offsets.push(offset);
        }

        Ok(offsets)
    }
}

/// Use case for consuming messages from topics
pub struct ConsumeMessagesUseCase {
    message_service: Arc<MessageService>,
    offset_service: Arc<OffsetManagementService>,
}

impl ConsumeMessagesUseCase {
    pub fn new(
        message_service: Arc<MessageService>,
        offset_service: Arc<OffsetManagementService>,
    ) -> Self {
        Self {
            message_service,
            offset_service,
        }
    }

    /// Consume messages from a topic starting from the consumer's current offset
    pub async fn execute(
        &self,
        consumer_id: String,
        topic_name: String,
        max_messages: usize,
    ) -> Result<Vec<Message>> {
        let consumer_id = ConsumerId::new(consumer_id);
        let topic_name = TopicName::new(topic_name)?;
        let topic_partition = TopicPartition::new(topic_name.clone(), PartitionId(0));

        // Get current offset for this consumer
        let current_offset = self
            .offset_service
            .get_offset(&consumer_id, &topic_partition)
            .await?;

        info!(
            "Consumer {} requesting {} messages from {} starting at offset {}",
            consumer_id, max_messages, topic_name, current_offset
        );

        // Fetch messages
        let messages = self
            .message_service
            .get_messages(&topic_name, current_offset, max_messages)
            .await?;

        debug!("Retrieved {} messages for consumer {}", messages.len(), consumer_id);

        Ok(messages)
    }

    /// Commit offset for a consumer after processing messages
    pub async fn commit_offset(
        &self,
        consumer_id: String,
        topic_name: String,
        offset: u64,
    ) -> Result<()> {
        let consumer_id = ConsumerId::new(consumer_id);
        let topic_name = TopicName::new(topic_name)?;
        let topic_partition = TopicPartition::new(topic_name, PartitionId(0));
        let offset = Offset::new(offset);

        info!(
            "Consumer {} committing offset {} for {}",
            consumer_id, offset, topic_partition
        );

        self.offset_service
            .commit_offset(&consumer_id, &topic_partition, offset)
            .await
    }
}

/// Use case for managing topics
pub struct TopicManagementUseCase {
    message_service: Arc<MessageService>,
}

impl TopicManagementUseCase {
    pub fn new(message_service: Arc<MessageService>) -> Self {
        Self { message_service }
    }

    /// Create a new topic
    pub async fn create_topic(&self, topic_name: String) -> Result<()> {
        let topic_name = TopicName::new(topic_name)?;
        
        info!("Creating topic: {}", topic_name);
        self.message_service.create_topic(topic_name).await
    }

    /// Get topic metadata
    pub async fn get_topic(&self, topic_name: String) -> Result<Option<Topic>> {
        let topic_name = TopicName::new(topic_name)?;
        self.message_service.get_topic(&topic_name).await
    }

    /// List all topics
    pub async fn list_topics(&self) -> Result<Vec<String>> {
        let topic_names = self.message_service.list_topics().await?;
        Ok(topic_names.into_iter().map(|t| t.0).collect())
    }
}
