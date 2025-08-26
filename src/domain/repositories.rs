use async_trait::async_trait;

use super::{
    entities::*,
    value_objects::*,
    errors::RepositoryError,
};

/// Repository for managing topics
#[async_trait]
pub trait TopicRepository: Send + Sync {
    async fn save(&self, topic: &Topic) -> Result<(), RepositoryError>;
    async fn find_by_name(&self, name: &TopicName) -> Result<Option<Topic>, RepositoryError>;
    async fn list_all(&self) -> Result<Vec<Topic>, RepositoryError>;
    async fn delete(&self, name: &TopicName) -> Result<(), RepositoryError>;
    async fn exists(&self, name: &TopicName) -> Result<bool, RepositoryError>;
}

/// Repository for managing message offsets
#[async_trait]
pub trait OffsetRepository: Send + Sync {
    async fn save_offset(
        &self,
        consumer: &ConsumerId,
        topic_partition: &TopicPartition,
        offset: Offset,
    ) -> Result<(), RepositoryError>;

    async fn load_offset(
        &self,
        consumer: &ConsumerId,
        topic_partition: &TopicPartition,
    ) -> Result<Option<Offset>, RepositoryError>;

    async fn delete_consumer_offsets(&self, consumer: &ConsumerId) -> Result<(), RepositoryError>;
}
