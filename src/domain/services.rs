use std::sync::Arc;
use log::{info, debug};

use super::{
    entities::*,
    value_objects::*,
    repositories::*,
    errors::*,
};

/// Service for managing message routing and topic operations
pub struct MessageService {
    topic_repo: Arc<dyn TopicRepository>,
}

impl MessageService {
    pub fn new(topic_repo: Arc<dyn TopicRepository>) -> Self {
        Self { topic_repo }
    }

    /// Send a message to a topic (creates topic if it doesn't exist)
    pub async fn send_message(
        &self,
        topic_name: TopicName,
        message: Message,
    ) -> Result<Offset> {
        debug!("Sending message to topic: {}", topic_name);

        // Get or create the topic
        let mut topic = match self.topic_repo.find_by_name(&topic_name).await.map_err(|e| {
            DomainError::InvalidMessage(format!("Repository error: {}", e))
        })? {
            Some(topic) => topic,
            None => {
                info!("Creating new topic: {}", topic_name);
                let new_topic = Topic::new(topic_name.clone());
                self.topic_repo.save(&new_topic).await.map_err(|e| {
                    DomainError::InvalidMessage(format!("Failed to create topic: {}", e))
                })?;
                new_topic
            }
        };

        // Add the message to the topic
        let offset = topic.add_message(message)?;

        // Save the updated topic
        self.topic_repo.save(&topic).await.map_err(|e| {
            DomainError::InvalidMessage(format!("Failed to save topic: {}", e))
        })?;

        info!("Message stored at offset {} in topic {}", offset, topic_name);
        Ok(offset)
    }

    /// Get messages from a topic starting from a specific offset
    pub async fn get_messages(
        &self,
        topic_name: &TopicName,
        from_offset: Offset,
        limit: usize,
    ) -> Result<Vec<Message>> {
        debug!(
            "Fetching up to {} messages from topic {} starting at offset {}",
            limit, topic_name, from_offset
        );

        let topic = self
            .topic_repo
            .find_by_name(topic_name)
            .await
            .map_err(|e| DomainError::InvalidMessage(format!("Repository error: {}", e)))?
            .ok_or_else(|| DomainError::TopicNotFound(topic_name.clone()))?;

        let messages: Vec<Message> = topic
            .get_messages(from_offset, limit)
            .into_iter()
            .cloned()
            .collect();

        debug!("Retrieved {} messages from topic {}", messages.len(), topic_name);
        Ok(messages)
    }

    /// Create a new topic
    pub async fn create_topic(&self, topic_name: TopicName) -> Result<()> {
        if self.topic_repo.exists(&topic_name).await.map_err(|e| {
            DomainError::InvalidMessage(format!("Repository error: {}", e))
        })? {
            return Err(DomainError::TopicAlreadyExists(topic_name));
        }

        let topic = Topic::new(topic_name.clone());
        self.topic_repo.save(&topic).await.map_err(|e| {
            DomainError::InvalidMessage(format!("Failed to create topic: {}", e))
        })?;

        info!("Created new topic: {}", topic_name);
        Ok(())
    }

    /// Get topic metadata
    pub async fn get_topic(&self, topic_name: &TopicName) -> Result<Option<Topic>> {
        self.topic_repo
            .find_by_name(topic_name)
            .await
            .map_err(|e| DomainError::InvalidMessage(format!("Repository error: {}", e)))
    }

    /// List all topics
    pub async fn list_topics(&self) -> Result<Vec<TopicName>> {
        let topics = self.topic_repo.list_all().await.map_err(|e| {
            DomainError::InvalidMessage(format!("Repository error: {}", e))
        })?;

        Ok(topics.into_iter().map(|t| t.name).collect())
    }
}

/// Service for managing consumer offsets
pub struct OffsetManagementService {
    offset_repo: Arc<dyn OffsetRepository>,
}

impl OffsetManagementService {
    pub fn new(offset_repo: Arc<dyn OffsetRepository>) -> Self {
        Self { offset_repo }
    }

    /// Get the current offset for a consumer and topic-partition
    pub async fn get_offset(
        &self,
        consumer_id: &ConsumerId,
        topic_partition: &TopicPartition,
    ) -> Result<Offset> {
        let offset = self
            .offset_repo
            .load_offset(consumer_id, topic_partition)
            .await
            .map_err(|e| DomainError::InvalidMessage(format!("Repository error: {}", e)))?
            .unwrap_or(Offset::new(0));

        debug!(
            "Current offset for consumer {} on {}: {}",
            consumer_id, topic_partition, offset
        );
        Ok(offset)
    }

    /// Commit an offset for a consumer and topic-partition
    pub async fn commit_offset(
        &self,
        consumer_id: &ConsumerId,
        topic_partition: &TopicPartition,
        offset: Offset,
    ) -> Result<()> {
        self.offset_repo
            .save_offset(consumer_id, topic_partition, offset)
            .await
            .map_err(|e| DomainError::InvalidMessage(format!("Repository error: {}", e)))?;

        debug!(
            "Committed offset {} for consumer {} on {}",
            offset, consumer_id, topic_partition
        );
        Ok(())
    }

    /// Reset offsets for a consumer (typically on disconnect)
    pub async fn reset_consumer_offsets(&self, consumer_id: &ConsumerId) -> Result<()> {
        self.offset_repo
            .delete_consumer_offsets(consumer_id)
            .await
            .map_err(|e| DomainError::InvalidMessage(format!("Repository error: {}", e)))?;

        info!("Reset all offsets for consumer {}", consumer_id);
        Ok(())
    }
}

/// Service for routing messages to partitions
pub struct MessageRoutingService;

impl MessageRoutingService {
    pub fn new() -> Self {
        Self
    }

    /// Route a message to a partition (simplified: always partition 0)
    pub fn route_message(&self, _message: &Message, _topic: &Topic) -> PartitionId {
        // In a real Kafka implementation, this would:
        // 1. Hash the message key to determine partition
        // 2. Round-robin if no key
        // 3. Consider partition availability
        // 
        // For educational purposes, we always use partition 0
        PartitionId(0)
    }
}

impl Default for MessageRoutingService {
    fn default() -> Self {
        Self::new()
    }
}
