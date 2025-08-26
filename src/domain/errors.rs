use thiserror::Error;

use super::value_objects::*;

pub type Result<T> = std::result::Result<T, DomainError>;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Topic '{0}' not found")]
    TopicNotFound(TopicName),

    #[error("Invalid topic name: {0}")]
    InvalidTopicName(String),

    #[error("Message offset {0} not found")]
    OffsetNotFound(Offset),

    #[error("Consumer '{0}' not subscribed to topic '{1}'")]
    ConsumerNotSubscribed(ConsumerId, TopicName),

    #[error("Partition {0} does not exist")]
    PartitionNotFound(u32),

    #[error("Producer '{0}' not found")]
    ProducerNotFound(ProducerId),

    #[error("Consumer '{0}' not found")]
    ConsumerNotFound(ConsumerId),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Topic '{0}' already exists")]
    TopicAlreadyExists(TopicName),
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),
}

impl From<serde_json::Error> for RepositoryError {
    fn from(err: serde_json::Error) -> Self {
        RepositoryError::Serialization(err.to_string())
    }
}
