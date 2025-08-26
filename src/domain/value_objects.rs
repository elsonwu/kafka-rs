use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Represents a topic name with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TopicName(pub String);

impl TopicName {
    pub fn new(name: String) -> Result<Self, crate::domain::errors::DomainError> {
        if name.is_empty() {
            return Err(crate::domain::errors::DomainError::InvalidTopicName(
                "Topic name cannot be empty".to_string(),
            ));
        }
        if name.len() > 255 {
            return Err(crate::domain::errors::DomainError::InvalidTopicName(
                "Topic name too long (max 255 characters)".to_string(),
            ));
        }
        // Basic validation - could be expanded
        if name.contains(' ') || name.contains('\t') {
            return Err(crate::domain::errors::DomainError::InvalidTopicName(
                "Topic name cannot contain whitespace".to_string(),
            ));
        }
        Ok(TopicName(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TopicName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for TopicName {
    fn from(name: String) -> Self {
        TopicName(name)
    }
}

impl From<&str> for TopicName {
    fn from(name: &str) -> Self {
        TopicName(name.to_string())
    }
}

/// Represents a message offset within a partition
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Offset(pub u64);

impl Offset {
    pub fn new(value: u64) -> Self {
        Offset(value)
    }

    pub fn next(&self) -> Self {
        Offset(self.0 + 1)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for Offset {
    fn from(value: u64) -> Self {
        Offset(value)
    }
}

/// Unique identifier for a message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

impl MessageId {
    pub fn new() -> Self {
        MessageId(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(MessageId(Uuid::parse_str(s)?))
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a partition within a topic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartitionId(pub u32);

impl PartitionId {
    pub fn new(id: u32) -> Self {
        PartitionId(id)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for PartitionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for PartitionId {
    fn from(id: u32) -> Self {
        PartitionId(id)
    }
}

/// Unique identifier for a consumer
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConsumerId(pub String);

impl ConsumerId {
    pub fn new(id: String) -> Self {
        ConsumerId(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ConsumerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ConsumerId {
    fn from(id: String) -> Self {
        ConsumerId(id)
    }
}

impl From<&str> for ConsumerId {
    fn from(id: &str) -> Self {
        ConsumerId(id.to_string())
    }
}

/// Unique identifier for a producer
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProducerId(pub String);

impl ProducerId {
    pub fn new(id: String) -> Self {
        ProducerId(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProducerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ProducerId {
    fn from(id: String) -> Self {
        ProducerId(id)
    }
}

impl From<&str> for ProducerId {
    fn from(id: &str) -> Self {
        ProducerId(id.to_string())
    }
}

/// Represents a topic-partition combination for offset tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TopicPartition {
    pub topic: TopicName,
    pub partition: PartitionId,
}

impl TopicPartition {
    pub fn new(topic: TopicName, partition: PartitionId) -> Self {
        Self { topic, partition }
    }
}

impl fmt::Display for TopicPartition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.topic, self.partition)
    }
}
