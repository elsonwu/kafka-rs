# Architecture Overview

## System Design Philosophy

Kafka-RS follows **Domain-Driven Design (DDD)** principles to create a clear separation between business logic and technical implementation. This makes the codebase educational and easier to understand.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Client Layer                          │
│  (KafkaJS, kafka-node, or any Kafka client)           │
└─────────────────────────┬───────────────────────────────┘
                          │ TCP/Kafka Protocol
┌─────────────────────────▼───────────────────────────────┐
│                Infrastructure Layer                     │
│  ┌─────────────────┐  ┌─────────────────┐             │
│  │   TCP Server    │  │   Protocol      │             │
│  │   (Tokio)       │  │   Handler       │             │
│  └─────────────────┘  └─────────────────┘             │
└─────────────────────────┬───────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────┐
│                Application Layer                        │
│  ┌─────────────────┐  ┌─────────────────┐             │
│  │  Producer       │  │  Consumer       │             │
│  │  Service        │  │  Service        │             │
│  └─────────────────┘  └─────────────────┘             │
└─────────────────────────┬───────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────┐
│                  Domain Layer                           │
│  ┌──────────┐ ┌──────────┐ ┌─────────────┐            │
│  │  Topic   │ │ Message  │ │ Partition   │            │
│  │ Entity   │ │ Entity   │ │ Entity      │            │
│  └──────────┘ └──────────┘ └─────────────┘            │
│  ┌─────────────────┐  ┌─────────────────┐             │
│  │   Domain        │  │  Repository     │             │
│  │   Services      │  │  Interfaces     │             │
│  └─────────────────┘  └─────────────────┘             │
└─────────────────────────┬───────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────┐
│              Infrastructure Layer (Storage)             │
│  ┌─────────────────┐  ┌─────────────────┐             │
│  │  In-Memory      │  │  Offset         │             │
│  │  Topic Store    │  │  Manager        │             │
│  └─────────────────┘  └─────────────────┘             │
└─────────────────────────────────────────────────────────┘
```

## Layer Responsibilities

### 1. Domain Layer (`src/domain/`)
**Purpose**: Contains the core business logic and rules

**Components**:
- **Entities**: `Topic`, `Message`, `Partition`, `Consumer`, `Producer`
- **Value Objects**: `TopicName`, `MessageId`, `Offset`
- **Domain Services**: Message routing, partition assignment
- **Repository Interfaces**: Abstract storage contracts

**Key Principles**:
- No external dependencies
- Pure business logic
- Framework-agnostic
- Rich domain model

### 2. Application Layer (`src/application/`)
**Purpose**: Orchestrates use cases and coordinates between layers

**Components**:
- **Use Cases**: `SendMessage`, `ConsumeMessages`, `CreateTopic`
- **Application Services**: Coordinate domain operations
- **DTOs**: Data transfer objects for cross-layer communication

**Responsibilities**:
- Transaction management
- Use case orchestration
- Security enforcement
- Input validation

### 3. Infrastructure Layer (`src/infrastructure/`)
**Purpose**: Handles external concerns and technical implementation

**Components**:
- **Network**: TCP server, protocol handlers
- **Persistence**: In-memory storage implementations
- **Logging**: Structured logging
- **Configuration**: System configuration

**External Dependencies**:
- Tokio for async runtime
- TCP sockets for network communication
- Logging frameworks

## Domain Model Core Concepts

### Topic
A named channel for organizing messages. In Kafka-RS, topics are:
- Created automatically on first use
- Contain one partition (simplified)
- Have a unique name
- Store messages in order

### Message
The fundamental unit of data with:
- Unique identifier
- Key-value payload
- Timestamp
- Offset within partition

### Partition
A subset of a topic that:
- Maintains message ordering
- Has sequential offsets
- Allows parallel consumption
- One per topic (simplified)

### Consumer
An entity that:
- Subscribes to topics
- Tracks consumption offset
- Requests messages in batches
- Manages its position

### Producer
An entity that:
- Sends messages to topics
- Can specify message keys
- Receives acknowledgments
- Handles routing

## Message Flow

### Producer Flow
1. Client sends produce request
2. Protocol layer parses request
3. Application service validates input
4. Domain service routes to partition
5. Message stored with offset
6. Acknowledgment sent to client

### Consumer Flow
1. Client sends fetch request
2. Protocol layer parses request
3. Application service retrieves messages
4. Domain service applies offset logic
5. Messages serialized and sent
6. Consumer updates offset

## Concurrency Model

### Thread Safety
- **Shared State**: Protected by `DashMap` and `RwLock`
- **Message Queues**: Lockless where possible
- **Consumer Groups**: Simplified single consumer per topic

### Async Design
- **Tokio Runtime**: For async I/O operations
- **Non-blocking**: Network and storage operations
- **Backpressure**: Natural flow control through async

## Storage Strategy

### In-Memory Storage
- **Topics**: `DashMap<TopicName, Topic>`
- **Messages**: `Vec<Message>` per partition
- **Offsets**: `HashMap<ConsumerId, Offset>`

### Trade-offs
- **Pros**: Fast, simple, educational
- **Cons**: No persistence, memory limited

## Protocol Compatibility

### Kafka Wire Protocol
We implement a subset of Kafka's binary protocol:
- **Produce API**: Send messages
- **Fetch API**: Retrieve messages
- **Metadata API**: Topic information

### Simplified Implementation
- Single partition per topic
- No compression
- Basic error handling
- Essential message formats

## Observability

### Educational Logging
Every operation logs detailed information:
```rust
log::info!("Received produce request for topic: {}", topic_name);
log::debug!("Storing message with offset: {}", offset);
log::info!("Consumer {} fetched {} messages", consumer_id, count);
```

### Metrics (Future)
- Message throughput
- Consumer lag
- Topic sizes
- Connection counts

## Design Patterns Used

### Repository Pattern
Abstract storage behind interfaces:
```rust
trait TopicRepository {
    async fn save_topic(&self, topic: Topic) -> Result<()>;
    async fn find_by_name(&self, name: &TopicName) -> Option<Topic>;
}
```

### Service Layer Pattern
Encapsulate business operations:
```rust
struct MessageService {
    topic_repo: Arc<dyn TopicRepository>,
}
```

### Builder Pattern
Complex object construction:
```rust
Message::builder()
    .key("user-123")
    .value("user data")
    .timestamp(now())
    .build()
```

## Testing Strategy

### Unit Tests
- Domain logic testing
- Repository implementations
- Service behaviors

### Integration Tests
- Full message flow
- Client compatibility
- Error scenarios

### Educational Focus
Tests serve as documentation and examples of expected behavior.

## Future Extensions

This architecture supports adding:
- Multiple partitions per topic
- Consumer groups
- Message persistence
- Replication
- Administrative APIs
- Schema registry

The DDD structure makes these extensions clear about where new functionality belongs.
