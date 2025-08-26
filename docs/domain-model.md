# Domain Model

## Overview

The domain model represents the core business concepts of our Kafka-compatible system. Following Domain-Driven Design principles, these entities capture the essential behavior and invariants of message streaming.

## Core Entities

### Topic

**Purpose**: A named channel that organizes and categorizes messages.

**Attributes**:
- `name: TopicName` - Unique identifier for the topic
- `partitions: Vec<Partition>` - List of partitions (simplified to 1)
- `created_at: DateTime` - When the topic was created
- `message_count: u64` - Total messages ever sent to this topic

**Invariants**:
- Topic name must be non-empty and valid
- Must have at least one partition
- Names are case-sensitive and unique

**Behavior**:
```rust
impl Topic {
    pub fn new(name: TopicName) -> Self;
    pub fn add_message(&mut self, message: Message) -> Result<Offset>;
    pub fn get_messages(&self, from_offset: Offset, limit: usize) -> Vec<Message>;
    pub fn get_partition(&self, partition_id: u32) -> Option<&Partition>;
}
```

### Message

**Purpose**: The fundamental unit of data that flows through the system.

**Attributes**:
- `id: MessageId` - Unique identifier (UUID)
- `key: Option<String>` - Optional message key for partitioning
- `value: Vec<u8>` - The actual message payload
- `timestamp: DateTime` - When the message was created
- `offset: Option<Offset>` - Position in partition (set when stored)
- `headers: HashMap<String, String>` - Optional message metadata

**Invariants**:
- Message ID must be unique
- Value can be empty but not null
- Timestamp must be valid
- Offset is immutable once set

**Behavior**:
```rust
impl Message {
    pub fn new(key: Option<String>, value: Vec<u8>) -> Self;
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self;
    pub fn assign_offset(&mut self, offset: Offset);
    pub fn size(&self) -> usize;
}
```

### Partition

**Purpose**: An ordered, immutable sequence of messages within a topic.

**Attributes**:
- `id: PartitionId` - Identifier within the topic
- `messages: Vec<Message>` - Ordered list of messages
- `high_watermark: Offset` - Highest committed offset
- `created_at: DateTime` - When partition was created

**Invariants**:
- Messages are stored in offset order
- Offsets are sequential starting from 0
- Messages cannot be deleted or modified
- High watermark only increases

**Behavior**:
```rust
impl Partition {
    pub fn new(id: PartitionId) -> Self;
    pub fn append_message(&mut self, message: Message) -> Offset;
    pub fn get_messages(&self, from: Offset, limit: usize) -> &[Message];
    pub fn get_high_watermark(&self) -> Offset;
    pub fn message_count(&self) -> usize;
}
```

### Consumer

**Purpose**: Represents a client that reads messages from topics.

**Attributes**:
- `id: ConsumerId` - Unique consumer identifier
- `group_id: Option<String>` - Consumer group (simplified)
- `subscriptions: HashSet<TopicName>` - Subscribed topics
- `offsets: HashMap<TopicPartition, Offset>` - Current positions
- `last_heartbeat: DateTime` - For connection management

**Invariants**:
- Consumer ID must be unique within a session
- Cannot consume from unsubscribed topics
- Offsets cannot go backwards
- Must maintain heartbeat to stay active

**Behavior**:
```rust
impl Consumer {
    pub fn new(id: ConsumerId, group_id: Option<String>) -> Self;
    pub fn subscribe(&mut self, topics: Vec<TopicName>);
    pub fn fetch_messages(&self, topic: &TopicName, limit: usize) -> Vec<Message>;
    pub fn commit_offset(&mut self, topic_partition: TopicPartition, offset: Offset);
    pub fn get_current_offset(&self, topic_partition: &TopicPartition) -> Offset;
}
```

### Producer

**Purpose**: Represents a client that sends messages to topics.

**Attributes**:
- `id: ProducerId` - Unique producer identifier
- `client_id: String` - Human-readable client name
- `transaction_id: Option<String>` - For transactional producers
- `created_at: DateTime` - When producer was created
- `message_count: u64` - Total messages sent

**Invariants**:
- Producer ID must be unique within a session
- Client ID should be descriptive
- Message count only increases

**Behavior**:
```rust
impl Producer {
    pub fn new(id: ProducerId, client_id: String) -> Self;
    pub fn send_message(&mut self, topic: TopicName, message: Message) -> Result<Offset>;
    pub fn send_batch(&mut self, topic: TopicName, messages: Vec<Message>) -> Result<Vec<Offset>>;
}
```

## Value Objects

### TopicName
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicName(String);

impl TopicName {
    pub fn new(name: String) -> Result<Self, ValidationError> {
        if name.is_empty() || name.len() > 255 {
            return Err(ValidationError::InvalidTopicName);
        }
        // Additional validation rules...
        Ok(TopicName(name))
    }
}
```

### Offset
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Offset(u64);

impl Offset {
    pub fn new(value: u64) -> Self { Offset(value) }
    pub fn next(&self) -> Self { Offset(self.0 + 1) }
    pub fn value(&self) -> u64 { self.0 }
}
```

### MessageId
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageId(Uuid);

impl MessageId {
    pub fn new() -> Self { MessageId(Uuid::new_v4()) }
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(MessageId(Uuid::parse_str(s)?))
    }
}
```

## Domain Services

### MessageRoutingService

**Purpose**: Determines which partition a message should go to.

```rust
pub struct MessageRoutingService;

impl MessageRoutingService {
    pub fn route_message(
        &self, 
        message: &Message, 
        topic: &Topic
    ) -> PartitionId {
        // Simplified: always use partition 0
        PartitionId(0)
    }
}
```

### OffsetManagementService

**Purpose**: Manages consumer offset tracking and commits.

```rust
pub struct OffsetManagementService {
    offsets: Arc<DashMap<(ConsumerId, TopicPartition), Offset>>,
}

impl OffsetManagementService {
    pub fn get_offset(&self, consumer: ConsumerId, tp: TopicPartition) -> Offset;
    pub fn commit_offset(&self, consumer: ConsumerId, tp: TopicPartition, offset: Offset);
    pub fn reset_offset(&self, consumer: ConsumerId, tp: TopicPartition, offset: Offset);
}
```

### TopicManagementService

**Purpose**: Handles topic lifecycle and metadata.

```rust
pub struct TopicManagementService {
    topics: Arc<DashMap<TopicName, Topic>>,
}

impl TopicManagementService {
    pub fn create_topic(&self, name: TopicName) -> Result<(), DomainError>;
    pub fn get_topic(&self, name: &TopicName) -> Option<Topic>;
    pub fn list_topics(&self) -> Vec<TopicName>;
    pub fn topic_exists(&self, name: &TopicName) -> bool;
}
```

## Repository Interfaces

### TopicRepository
```rust
#[async_trait]
pub trait TopicRepository: Send + Sync {
    async fn save(&self, topic: &Topic) -> Result<(), RepositoryError>;
    async fn find_by_name(&self, name: &TopicName) -> Result<Option<Topic>, RepositoryError>;
    async fn list_all(&self) -> Result<Vec<Topic>, RepositoryError>;
    async fn delete(&self, name: &TopicName) -> Result<(), RepositoryError>;
}
```

### MessageRepository
```rust
#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn store_message(
        &self, 
        topic: &TopicName, 
        partition: PartitionId, 
        message: Message
    ) -> Result<Offset, RepositoryError>;
    
    async fn fetch_messages(
        &self,
        topic: &TopicName,
        partition: PartitionId,
        from_offset: Offset,
        limit: usize,
    ) -> Result<Vec<Message>, RepositoryError>;
}
```

### OffsetRepository
```rust
#[async_trait]
pub trait OffsetRepository: Send + Sync {
    async fn save_offset(
        &self,
        consumer: ConsumerId,
        topic_partition: TopicPartition,
        offset: Offset,
    ) -> Result<(), RepositoryError>;
    
    async fn load_offset(
        &self,
        consumer: ConsumerId,
        topic_partition: &TopicPartition,
    ) -> Result<Option<Offset>, RepositoryError>;
}
```

## Domain Events

### MessageProduced
```rust
#[derive(Debug, Clone)]
pub struct MessageProduced {
    pub topic: TopicName,
    pub partition: PartitionId,
    pub offset: Offset,
    pub message_id: MessageId,
    pub producer_id: ProducerId,
    pub timestamp: DateTime<Utc>,
}
```

### MessageConsumed
```rust
#[derive(Debug, Clone)]
pub struct MessageConsumed {
    pub topic: TopicName,
    pub partition: PartitionId,
    pub offset: Offset,
    pub message_id: MessageId,
    pub consumer_id: ConsumerId,
    pub timestamp: DateTime<Utc>,
}
```

### TopicCreated
```rust
#[derive(Debug, Clone)]
pub struct TopicCreated {
    pub topic: TopicName,
    pub partition_count: u32,
    pub timestamp: DateTime<Utc>,
}
```

## Error Types

### DomainError
```rust
#[derive(Debug, thiserror::Error)]
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
}
```

## Business Rules

### Message Ordering
- Messages within a partition are strictly ordered by offset
- Offsets are sequential integers starting from 0
- Consumers read messages in offset order

### Topic Auto-Creation
- Topics are automatically created on first produce request
- Default configuration: 1 partition, no replication
- Topic names must follow naming conventions

### Consumer Semantics
- Each consumer tracks its own offset per topic-partition
- Consumers can read from any valid offset
- At-least-once delivery guarantee (simplified)

### Producer Semantics  
- Messages are immediately available after successful write
- No batching in this simplified implementation
- Synchronous acknowledgment after storage

## Aggregate Boundaries

### Topic Aggregate
- **Root**: Topic entity
- **Contains**: Partitions and their messages
- **Invariants**: Partition consistency, message ordering

### Consumer Aggregate  
- **Root**: Consumer entity
- **Contains**: Offset tracking, subscriptions
- **Invariants**: Offset progression, subscription validity

### Producer Aggregate
- **Root**: Producer entity  
- **Contains**: Message sending history
- **Invariants**: Producer identity, message counting

This domain model provides a solid foundation for implementing Kafka-like behavior while remaining educational and understandable. The simplified design focuses on core concepts without overwhelming complexity.
