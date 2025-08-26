use serde::{Deserialize, Serialize};

/// Data Transfer Object for produce requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProduceRequest {
    pub topic: String,
    pub messages: Vec<MessageData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData {
    pub key: Option<String>,
    pub value: Vec<u8>,
    pub headers: Option<std::collections::HashMap<String, String>>,
}

/// Data Transfer Object for produce responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProduceResponse {
    pub topic: String,
    pub partition: u32,
    pub offsets: Vec<u64>,
}

/// Data Transfer Object for fetch requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchRequest {
    pub consumer_id: String,
    pub topic: String,
    pub partition: u32,
    pub offset: u64,
    pub max_bytes: u32,
}

/// Data Transfer Object for fetch responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResponse {
    pub topic: String,
    pub partition: u32,
    pub messages: Vec<FetchedMessage>,
    pub high_watermark: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedMessage {
    pub offset: u64,
    pub key: Option<String>,
    pub value: Vec<u8>,
    pub timestamp: i64,
    pub headers: std::collections::HashMap<String, String>,
}

/// Data Transfer Object for offset commit requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffsetCommitRequest {
    pub consumer_id: String,
    pub topics: Vec<TopicOffsetCommit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicOffsetCommit {
    pub topic: String,
    pub partitions: Vec<PartitionOffsetCommit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionOffsetCommit {
    pub partition: u32,
    pub offset: u64,
}

/// Data Transfer Object for metadata requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataRequest {
    pub topics: Option<Vec<String>>,
}

/// Data Transfer Object for metadata responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataResponse {
    pub brokers: Vec<BrokerMetadata>,
    pub topics: Vec<TopicMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerMetadata {
    pub node_id: i32,
    pub host: String,
    pub port: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicMetadata {
    pub name: String,
    pub partitions: Vec<PartitionMetadata>,
    pub error_code: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionMetadata {
    pub partition_id: u32,
    pub leader: i32,
    pub replicas: Vec<i32>,
    pub isr: Vec<i32>, // In-sync replicas
    pub error_code: i16,
}
