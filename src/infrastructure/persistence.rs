use std::sync::Arc;
use dashmap::DashMap;
use async_trait::async_trait;

use crate::domain::{
    entities::Topic,
    value_objects::*,
    repositories::*,
    errors::RepositoryError,
};

/// In-memory implementation of the TopicRepository
pub struct InMemoryTopicRepository {
    topics: Arc<DashMap<TopicName, Topic>>,
}

impl InMemoryTopicRepository {
    pub fn new() -> Self {
        Self {
            topics: Arc::new(DashMap::new()),
        }
    }
}

impl Default for InMemoryTopicRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TopicRepository for InMemoryTopicRepository {
    async fn save(&self, topic: &Topic) -> Result<(), RepositoryError> {
        self.topics.insert(topic.name.clone(), topic.clone());
        Ok(())
    }

    async fn find_by_name(&self, name: &TopicName) -> Result<Option<Topic>, RepositoryError> {
        Ok(self.topics.get(name).map(|entry| entry.value().clone()))
    }

    async fn list_all(&self) -> Result<Vec<Topic>, RepositoryError> {
        Ok(self.topics.iter().map(|entry| entry.value().clone()).collect())
    }

    async fn delete(&self, name: &TopicName) -> Result<(), RepositoryError> {
        self.topics.remove(name);
        Ok(())
    }

    async fn exists(&self, name: &TopicName) -> Result<bool, RepositoryError> {
        Ok(self.topics.contains_key(name))
    }
}

/// In-memory implementation of the OffsetRepository
pub struct InMemoryOffsetRepository {
    offsets: Arc<DashMap<(ConsumerId, TopicPartition), Offset>>,
}

impl InMemoryOffsetRepository {
    pub fn new() -> Self {
        Self {
            offsets: Arc::new(DashMap::new()),
        }
    }
}

impl Default for InMemoryOffsetRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OffsetRepository for InMemoryOffsetRepository {
    async fn save_offset(
        &self,
        consumer: &ConsumerId,
        topic_partition: &TopicPartition,
        offset: Offset,
    ) -> Result<(), RepositoryError> {
        let key = (consumer.clone(), topic_partition.clone());
        self.offsets.insert(key, offset);
        Ok(())
    }

    async fn load_offset(
        &self,
        consumer: &ConsumerId,
        topic_partition: &TopicPartition,
    ) -> Result<Option<Offset>, RepositoryError> {
        let key = (consumer.clone(), topic_partition.clone());
        Ok(self.offsets.get(&key).map(|entry| *entry.value()))
    }

    async fn delete_consumer_offsets(&self, consumer: &ConsumerId) -> Result<(), RepositoryError> {
        // Remove all offsets for this consumer
        self.offsets.retain(|(c, _), _| c != consumer);
        Ok(())
    }
}
