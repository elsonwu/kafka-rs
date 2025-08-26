use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::value_objects::*;

/// Event emitted when a message is produced to a topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageProduced {
    pub topic: TopicName,
    pub partition: PartitionId,
    pub offset: Offset,
    pub message_id: MessageId,
    pub producer_id: ProducerId,
    pub timestamp: DateTime<Utc>,
}

impl MessageProduced {
    pub fn new(
        topic: TopicName,
        partition: PartitionId,
        offset: Offset,
        message_id: MessageId,
        producer_id: ProducerId,
    ) -> Self {
        Self {
            topic,
            partition,
            offset,
            message_id,
            producer_id,
            timestamp: Utc::now(),
        }
    }
}

/// Event emitted when a message is consumed from a topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageConsumed {
    pub topic: TopicName,
    pub partition: PartitionId,
    pub offset: Offset,
    pub message_id: MessageId,
    pub consumer_id: ConsumerId,
    pub timestamp: DateTime<Utc>,
}

impl MessageConsumed {
    pub fn new(
        topic: TopicName,
        partition: PartitionId,
        offset: Offset,
        message_id: MessageId,
        consumer_id: ConsumerId,
    ) -> Self {
        Self {
            topic,
            partition,
            offset,
            message_id,
            consumer_id,
            timestamp: Utc::now(),
        }
    }
}

/// Event emitted when a new topic is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCreated {
    pub topic: TopicName,
    pub partition_count: u32,
    pub timestamp: DateTime<Utc>,
}

impl TopicCreated {
    pub fn new(topic: TopicName, partition_count: u32) -> Self {
        Self {
            topic,
            partition_count,
            timestamp: Utc::now(),
        }
    }
}

/// Event emitted when a consumer subscribes to topics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerSubscribed {
    pub consumer_id: ConsumerId,
    pub topics: Vec<TopicName>,
    pub timestamp: DateTime<Utc>,
}

impl ConsumerSubscribed {
    pub fn new(consumer_id: ConsumerId, topics: Vec<TopicName>) -> Self {
        Self {
            consumer_id,
            topics,
            timestamp: Utc::now(),
        }
    }
}

/// Event emitted when an offset is committed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffsetCommitted {
    pub consumer_id: ConsumerId,
    pub topic_partition: TopicPartition,
    pub offset: Offset,
    pub timestamp: DateTime<Utc>,
}

impl OffsetCommitted {
    pub fn new(
        consumer_id: ConsumerId,
        topic_partition: TopicPartition,
        offset: Offset,
    ) -> Self {
        Self {
            consumer_id,
            topic_partition,
            offset,
            timestamp: Utc::now(),
        }
    }
}
