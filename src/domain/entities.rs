use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::value_objects::*;

/// A topic represents a named channel for organizing messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub name: TopicName,
    pub partitions: Vec<Partition>,
    pub created_at: DateTime<Utc>,
    pub message_count: u64,
}

impl Topic {
    /// Create a new topic with a single partition
    pub fn new(name: TopicName) -> Self {
        Self {
            name,
            partitions: vec![Partition::new(PartitionId(0))],
            created_at: Utc::now(),
            message_count: 0,
        }
    }

    /// Add a message to the topic (goes to partition 0 in this simplified implementation)
    pub fn add_message(&mut self, mut message: Message) -> crate::domain::errors::Result<Offset> {
        if let Some(partition) = self.partitions.get_mut(0) {
            let offset = partition.append_message(message);
            self.message_count += 1;
            Ok(offset)
        } else {
            Err(crate::domain::errors::DomainError::PartitionNotFound(0))
        }
    }

    /// Get messages from the specified offset
    pub fn get_messages(&self, from_offset: Offset, limit: usize) -> Vec<&Message> {
        if let Some(partition) = self.partitions.get(0) {
            partition.get_messages(from_offset, limit)
        } else {
            Vec::new()
        }
    }

    /// Get a specific partition by ID
    pub fn get_partition(&self, partition_id: u32) -> Option<&Partition> {
        self.partitions.iter().find(|p| p.id.0 == partition_id)
    }

    /// Get the high watermark (highest offset) for partition 0
    pub fn get_high_watermark(&self) -> Offset {
        self.partitions
            .get(0)
            .map(|p| p.get_high_watermark())
            .unwrap_or(Offset::new(0))
    }
}

/// A partition is an ordered, immutable sequence of messages within a topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    pub id: PartitionId,
    pub messages: Vec<Message>,
    pub high_watermark: Offset,
    pub created_at: DateTime<Utc>,
}

impl Partition {
    /// Create a new empty partition
    pub fn new(id: PartitionId) -> Self {
        Self {
            id,
            messages: Vec::new(),
            high_watermark: Offset::new(0),
            created_at: Utc::now(),
        }
    }

    /// Append a message to the partition and return its offset
    pub fn append_message(&mut self, mut message: Message) -> Offset {
        let offset = self.high_watermark;
        message.offset = Some(offset);
        self.messages.push(message);
        self.high_watermark = offset.next();
        offset
    }

    /// Get messages starting from the specified offset
    pub fn get_messages(&self, from_offset: Offset, limit: usize) -> Vec<&Message> {
        let start_idx = from_offset.value() as usize;
        self.messages.iter().skip(start_idx).take(limit).collect()
    }

    /// Get the highest committed offset
    pub fn get_high_watermark(&self) -> Offset {
        self.high_watermark
    }

    /// Get the number of messages in this partition
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

/// A message is the fundamental unit of data that flows through the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub key: Option<String>,
    pub value: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub offset: Option<Offset>,
    pub headers: HashMap<String, String>,
}

impl Message {
    /// Create a new message
    pub fn new(key: Option<String>, value: Vec<u8>) -> Self {
        Self {
            id: MessageId::new(),
            key,
            value,
            timestamp: Utc::now(),
            offset: None,
            headers: HashMap::new(),
        }
    }

    /// Add headers to the message
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    /// Get the size of the message in bytes
    pub fn size(&self) -> usize {
        self.value.len()
            + self.key.as_ref().map(|k| k.len()).unwrap_or(0)
            + self
                .headers
                .iter()
                .map(|(k, v)| k.len() + v.len())
                .sum::<usize>()
    }
}

/// A consumer represents a client that reads messages from topics
#[derive(Debug, Clone)]
pub struct Consumer {
    pub id: ConsumerId,
    pub group_id: Option<String>,
    pub subscriptions: std::collections::HashSet<TopicName>,
    pub offsets: HashMap<TopicPartition, Offset>,
    pub last_heartbeat: DateTime<Utc>,
}

impl Consumer {
    /// Create a new consumer
    pub fn new(id: ConsumerId, group_id: Option<String>) -> Self {
        Self {
            id,
            group_id,
            subscriptions: std::collections::HashSet::new(),
            offsets: HashMap::new(),
            last_heartbeat: Utc::now(),
        }
    }

    /// Subscribe to topics
    pub fn subscribe(&mut self, topics: Vec<TopicName>) {
        for topic in topics {
            self.subscriptions.insert(topic);
        }
    }

    /// Get the current offset for a topic-partition
    pub fn get_current_offset(&self, topic_partition: &TopicPartition) -> Offset {
        self.offsets
            .get(topic_partition)
            .copied()
            .unwrap_or(Offset::new(0))
    }

    /// Commit an offset for a topic-partition
    pub fn commit_offset(&mut self, topic_partition: TopicPartition, offset: Offset) {
        self.offsets.insert(topic_partition, offset);
    }

    /// Update heartbeat timestamp
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
    }
}

/// A producer represents a client that sends messages to topics
#[derive(Debug, Clone)]
pub struct Producer {
    pub id: ProducerId,
    pub client_id: String,
    pub transaction_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub message_count: u64,
}

impl Producer {
    /// Create a new producer
    pub fn new(id: ProducerId, client_id: String) -> Self {
        Self {
            id,
            client_id,
            transaction_id: None,
            created_at: Utc::now(),
            message_count: 0,
        }
    }

    /// Increment message count (called when a message is sent)
    pub fn increment_message_count(&mut self) {
        self.message_count += 1;
    }
}
