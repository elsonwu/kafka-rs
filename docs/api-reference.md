# API Reference

## Overview

Kafka-RS implements a subset of the Apache Kafka wire protocol to maintain compatibility with existing Kafka clients like KafkaJS. This document describes the supported APIs and their behavior.

## Protocol Compatibility

### Supported API Keys

| API Key | Name | Description | Status |
|---------|------|-------------|--------|
| 0 | Produce | Send messages to topics | ✅ Implemented |
| 1 | Fetch | Retrieve messages from topics | ✅ Implemented |
| 3 | Metadata | Get topic and broker information | ✅ Implemented |
| 8 | OffsetCommit | Commit consumer offsets | 🔄 Basic implementation |
| 9 | OffsetFetch | Retrieve consumer offsets | 🔄 Basic implementation |

### Unsupported Features (Educational Scope)

- Consumer Groups (simplified single-consumer model)
- Transactions
- Compression
- Message headers (parsed but not fully utilized)
- Authentication/Authorization
- Replication
- Administrative operations beyond topic auto-creation

## API Details

### Produce API (Key: 0)

**Purpose**: Send messages to a topic

**Request Format**:
```
ProduceRequest => 
  TransactionalId => NULLABLE_STRING
  Acks => INT16
  TimeoutMs => INT32
  TopicData => ARRAY
    Topic => STRING
    PartitionData => ARRAY
      Partition => INT32
      RecordSet => RECORDS
```

**Response Format**:
```
ProduceResponse =>
  ThrottleTimeMs => INT32
  Responses => ARRAY
    Topic => STRING
    PartitionResponses => ARRAY
      Partition => INT32
      ErrorCode => INT16
      BaseOffset => INT64
      LogAppendTimeMs => INT64
      LogStartOffset => INT64
```

**Behavior**:
- Topics are auto-created if they don't exist
- All messages go to partition 0 (simplified)
- Synchronous acknowledgment (acks=1 behavior)
- Returns the offset of each stored message

**Error Codes**:
- 0: No error
- -1: Generic error

### Fetch API (Key: 1)

**Purpose**: Retrieve messages from a topic

**Request Format**:
```
FetchRequest =>
  ReplicaId => INT32
  MaxWaitMs => INT32
  MinBytes => INT32
  MaxBytes => INT32
  IsolationLevel => INT8
  SessionId => INT32
  SessionEpoch => INT32
  Topics => ARRAY
    Topic => STRING
    Partitions => ARRAY
      Partition => INT32
      CurrentLeaderEpoch => INT32
      FetchOffset => INT64
      LogStartOffset => INT64
      PartitionMaxBytes => INT32
```

**Response Format**:
```
FetchResponse =>
  ThrottleTimeMs => INT32
  ErrorCode => INT16
  SessionId => INT32
  Responses => ARRAY
    Topic => STRING
    Partitions => ARRAY
      Partition => INT32
      ErrorCode => INT16
      HighWatermark => INT64
      LastStableOffset => INT64
      LogStartOffset => INT64
      AbortedTransactions => ARRAY
      PreferredReadReplica => INT32
      Records => RECORDS
```

**Behavior**:
- Returns messages starting from requested offset
- Maximum of 100 messages per request (configurable)
- Uses consumer correlation ID for offset tracking
- Returns empty response if no messages available

### Metadata API (Key: 3)

**Purpose**: Get cluster and topic metadata

**Request Format**:
```
MetadataRequest =>
  Topics => NULLABLE_ARRAY
    Topic => STRING
  AllowAutoTopicCreation => BOOLEAN
  IncludeClusterAuthorizedOperations => BOOLEAN
  IncludeTopicAuthorizedOperations => BOOLEAN
```

**Response Format**:
```
MetadataResponse =>
  ThrottleTimeMs => INT32
  Brokers => ARRAY
    NodeId => INT32
    Host => STRING
    Port => INT32
    Rack => NULLABLE_STRING
  ClusterId => NULLABLE_STRING
  ControllerId => INT32
  Topics => ARRAY
    ErrorCode => INT16
    Name => STRING
    IsInternal => BOOLEAN
    Partitions => ARRAY
      ErrorCode => INT16
      PartitionIndex => INT32
      LeaderId => INT32
      LeaderEpoch => INT32
      ReplicaNodes => ARRAY
      IsrNodes => ARRAY
      OfflineReplicas => ARRAY
```

**Behavior**:
- Returns information about all topics if none specified
- Single broker (localhost) configuration
- Each topic has one partition (partition 0)
- Broker ID 0 is leader for all partitions

## Client Integration

### KafkaJS Example

```javascript
import { Kafka } from 'kafkajs';

const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['localhost:9092'],
});

// Producer
const producer = kafka.producer();
await producer.connect();

await producer.send({
  topic: 'my-topic',
  messages: [
    { value: 'Hello World!' },
    { key: 'user-123', value: 'User data' }
  ],
});

await producer.disconnect();

// Consumer
const consumer = kafka.consumer({ groupId: 'my-group' });
await consumer.connect();
await consumer.subscribe({ topic: 'my-topic' });

await consumer.run({
  eachMessage: async ({ topic, partition, message }) => {
    console.log({
      topic,
      partition,
      offset: message.offset,
      value: message.value.toString(),
    });
  },
});
```

### Connection Management

**Connection Lifecycle**:
1. Client connects via TCP to port 9092
2. Client sends API requests with correlation IDs
3. Server processes requests and sends responses
4. Client manages connection state

**Error Handling**:
- Protocol errors result in error responses with correlation IDs
- Network errors cause connection termination
- Invalid requests are logged and responded to with error codes

## Message Format

### Internal Message Structure

```rust
pub struct Message {
    pub id: MessageId,           // UUID
    pub key: Option<String>,     // Optional message key
    pub value: Vec<u8>,         // Message payload
    pub timestamp: DateTime<Utc>, // Creation timestamp
    pub offset: Option<Offset>,  // Set when stored
    pub headers: HashMap<String, String>, // Message headers
}
```

### Wire Format

Messages on the wire follow Kafka's record format:
- Length-prefixed protocol messages
- Big-endian byte order
- Nullable strings represented as length -1
- Arrays prefixed with element count

## Logging and Observability

### Request Logging

Each API request is logged with:
- API type and correlation ID
- Topic and partition information
- Message counts and sizes
- Processing time and outcomes

### Example Log Output

```
[INFO] Starting Kafka-RS server on 127.0.0.1:9092
[INFO] New connection from: 127.0.0.1:52341
[DEBUG] Processing PRODUCE request (correlation_id: 1)
[DEBUG] Produce request for topic: test-topic
[INFO] Creating new topic: test-topic
[DEBUG] Message stored at offset 0 in topic test-topic
[DEBUG] Sent response (45 bytes)
[DEBUG] Processing FETCH request (correlation_id: 2)
[DEBUG] Fetch request for topic: test-topic, offset: 0, max_bytes: 1048576
[DEBUG] Retrieved 1 messages from topic test-topic
[DEBUG] Sent response (89 bytes)
```

## Error Codes

### Common Error Codes

| Code | Name | Description |
|------|------|-------------|
| 0 | NONE | No error |
| -1 | UNKNOWN | Generic error |
| 3 | UNKNOWN_TOPIC_OR_PARTITION | Topic does not exist |
| 6 | NOT_LEADER_FOR_PARTITION | This server is not the leader |

### Error Response Format

Error responses maintain the same correlation ID as the request and include appropriate error codes in the response structure.

## Performance Characteristics

### Educational Limitations

- **Throughput**: Suitable for learning, not production workloads
- **Latency**: Synchronous processing, no batching optimizations
- **Memory Usage**: All data stored in memory, lost on restart
- **Concurrency**: Async I/O but simplified concurrency model

### Scaling Considerations

This implementation is intentionally simplified for educational purposes:
- Single-threaded message processing
- No persistent storage
- No clustering or replication
- Basic consumer offset management

## Future Extensions

### Potential Improvements

1. **Persistence**: Add file-based or database storage
2. **Partitioning**: Support multiple partitions per topic
3. **Consumer Groups**: Full consumer group coordination
4. **Compression**: Add compression support
5. **Security**: Authentication and authorization
6. **Admin API**: Topic management operations
7. **Metrics**: Detailed operational metrics

### Implementation Guide

To extend this implementation:
1. Start with domain model extensions
2. Update repository interfaces
3. Implement new infrastructure components
4. Add corresponding protocol handlers
5. Update documentation and examples
