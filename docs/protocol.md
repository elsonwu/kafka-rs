# Protocol Implementation

## Overview

Kafka-RS implements a subset of the Apache Kafka wire protocol to maintain compatibility with existing Kafka clients. This document explains how the protocol is implemented and the educational simplifications made.

## Kafka Wire Protocol Basics

### Message Structure

All Kafka protocol messages follow this structure:

```
Message => Size MessageContent
  Size => INT32              // Big-endian message size
  MessageContent => RequestMessage | ResponseMessage
```

### Request Message Format

```
RequestMessage => RequestHeader RequestBody
  RequestHeader => ApiKey ApiVersion CorrelationId ClientId
    ApiKey => INT16          // Identifies the request type
    ApiVersion => INT16      // Protocol version
    CorrelationId => INT32   // Matches request with response
    ClientId => NULLABLE_STRING  // Client identifier
  RequestBody => (varies by API)
```

### Response Message Format

```
ResponseMessage => ResponseHeader ResponseBody
  ResponseHeader => CorrelationId
    CorrelationId => INT32   // Matches the request
  ResponseBody => (varies by API)
```

## Data Type Encoding

### Primitive Types

| Type | Size | Encoding | Description |
|------|------|----------|-------------|
| INT8 | 1 byte | Big-endian | Signed 8-bit integer |
| INT16 | 2 bytes | Big-endian | Signed 16-bit integer |
| INT32 | 4 bytes | Big-endian | Signed 32-bit integer |
| INT64 | 8 bytes | Big-endian | Signed 64-bit integer |

### String Encoding

```rust
// Nullable string encoding
pub fn encode_string(buf: &mut BytesMut, value: Option<&str>) -> io::Result<()> {
    match value {
        Some(s) => {
            let bytes = s.as_bytes();
            encode_i16(buf, bytes.len() as i16);  // Length prefix
            buf.put_slice(bytes);                 // UTF-8 bytes
        }
        None => {
            encode_i16(buf, -1);                  // Null marker
        }
    }
    Ok(())
}
```

### Array Encoding

```rust
// Array with length prefix
encode_i32(buf, array.len() as i32);  // Element count
for item in array {
    // Encode each item
    item.encode(buf)?;
}
```

### Bytes Encoding

```rust
// Nullable bytes encoding
pub fn encode_bytes(buf: &mut BytesMut, value: Option<&[u8]>) -> io::Result<()> {
    match value {
        Some(bytes) => {
            encode_i32(buf, bytes.len() as i32); // Length prefix
            buf.put_slice(bytes);                // Raw bytes
        }
        None => {
            encode_i32(buf, -1);                 // Null marker
        }
    }
    Ok(())
}
```

## Implemented APIs

### 1. Produce API (Key: 0)

**Purpose**: Send messages to topics

**Request Processing**:
```rust
async fn handle_produce_request(&mut self, header: RequestHeader, buf: &mut BytesMut) -> anyhow::Result<()> {
    // 1. Decode the produce request
    let request = ProduceRequest::decode(buf)?;
    
    // 2. Process each message
    for msg in request.messages {
        let key = msg.key.map(|k| String::from_utf8_lossy(&k).to_string());
        let value = msg.value.unwrap_or_default();
        
        // 3. Store the message (creates topic if needed)
        let offset = self.send_message_use_case.execute(
            request.topic.clone(), 
            key, 
            value
        ).await?;
    }
    
    // 4. Send acknowledgment response
    self.send_produce_response(header.correlation_id, &request.topic, offsets).await?;
}
```

**Simplified Request Format**:
```
ProduceRequest => 
  TransactionalId => NULLABLE_STRING    // Ignored (no transactions)
  Acks => INT16                         // Always treated as 1
  TimeoutMs => INT32                    // Ignored (synchronous)
  TopicData => ARRAY
    Topic => STRING
    PartitionData => ARRAY
      Partition => INT32                // Always 0 (single partition)
      RecordSet => RECORDS
```

**Response Format**:
```
ProduceResponse =>
  ThrottleTimeMs => INT32               // Always 0
  Responses => ARRAY
    Topic => STRING
    PartitionResponses => ARRAY
      Partition => INT32                // Always 0
      ErrorCode => INT16                // 0 for success
      BaseOffset => INT64               // First message offset
      LogAppendTimeMs => INT64          // -1 (not used)
      LogStartOffset => INT64           // Always 0
```

### 2. Fetch API (Key: 1)

**Purpose**: Retrieve messages from topics

**Request Processing**:
```rust
async fn handle_fetch_request(&mut self, header: RequestHeader, buf: &mut BytesMut) -> anyhow::Result<()> {
    // 1. Decode fetch request
    let request = FetchRequest::decode(buf)?;
    
    // 2. Generate consumer ID from correlation ID
    let consumer_id = format!("consumer-{}", header.correlation_id);
    
    // 3. Fetch messages from the topic
    let messages = self.consume_messages_use_case.execute(
        consumer_id, 
        request.topic.clone(), 
        100  // Max 100 messages
    ).await?;
    
    // 4. Send fetch response
    self.send_fetch_response(
        header.correlation_id, 
        &request.topic, 
        messages, 
        request.offset as u64
    ).await?;
}
```

**Message Record Format**:
```
Record =>
  Length => INT32
  Attributes => INT8                    // Always 0 (no compression)
  TimestampDelta => VARINT              // Simplified to full timestamp
  OffsetDelta => VARINT                 // Always 0 (absolute offsets)
  KeyLength => VARINT
  Key => BYTES
  ValueLength => VARINT  
  Value => BYTES
  Headers => ARRAY                      // Empty array
```

### 3. Metadata API (Key: 3)

**Purpose**: Get cluster and topic information

**Response Structure**:
```rust
async fn send_metadata_response(&mut self, correlation_id: i32, topics: Vec<String>) -> anyhow::Result<()> {
    // Single broker metadata
    encode_i32(&mut response, 1);           // Broker count
    encode_i32(&mut response, 0);           // Broker ID
    encode_string(&mut response, Some("localhost"))?;
    encode_i32(&mut response, 9092);        // Port
    
    // Topic metadata
    for topic in topics {
        encode_i16(&mut response, 0);       // Error code
        encode_string(&mut response, Some(&topic))?;
        encode_i8(&mut response, 0);        // Not internal
        
        // Single partition per topic
        encode_i32(&mut response, 1);       // Partition count
        encode_i16(&mut response, 0);       // Error code
        encode_i32(&mut response, 0);       // Partition ID
        encode_i32(&mut response, 0);       // Leader (broker 0)
        
        // Replica and ISR (both contain just broker 0)
        encode_i32(&mut response, 1);       // Replica count
        encode_i32(&mut response, 0);       // Replica broker
        encode_i32(&mut response, 1);       // ISR count  
        encode_i32(&mut response, 0);       // ISR broker
    }
}
```

### 4. Offset APIs (Keys: 8, 9)

**OffsetCommit (Key: 8)** and **OffsetFetch (Key: 9)** are implemented with basic responses:

```rust
// Simple acknowledgment for offset commits
async fn send_offset_commit_response(&mut self, correlation_id: i32) -> anyhow::Result<()> {
    let mut response = BytesMut::new();
    let header = ResponseHeader { correlation_id };
    header.encode(&mut response)?;
    
    encode_i32(&mut response, 0);  // Throttle time
    encode_i32(&mut response, 0);  // Empty topics array
    
    self.send_response(response).await
}
```

## Educational Simplifications

### 1. Single Partition Model

**Real Kafka**: Topics can have multiple partitions for scalability
**Kafka-RS**: Each topic has exactly one partition (ID: 0)

```rust
// Simplified partition assignment
pub fn route_message(&self, _message: &Message, _topic: &Topic) -> PartitionId {
    PartitionId(0)  // Always partition 0
}
```

**Benefits**: 
- Easier to understand message ordering
- Simpler offset management
- Focus on core concepts

### 2. Synchronous Processing

**Real Kafka**: Asynchronous batching with configurable acknowledgments
**Kafka-RS**: Synchronous processing with immediate acknowledgment

```rust
// Immediate acknowledgment after storage
let offset = topic.add_message(message)?;
self.topic_repo.save(&topic).await?;
// Response sent immediately
```

**Benefits**:
- Predictable behavior for learning
- Simpler error handling
- Clear cause-and-effect relationships

### 3. In-Memory Storage

**Real Kafka**: Persistent log files with configurable retention
**Kafka-RS**: In-memory storage with HashMap/DashMap

```rust
pub struct InMemoryTopicRepository {
    topics: Arc<DashMap<TopicName, Topic>>,  // Lost on restart
}
```

**Benefits**:
- Fast startup and operation
- No file system complexity
- Focus on protocol and logic

### 4. Simplified Consumer Groups

**Real Kafka**: Complex coordinator protocol with rebalancing
**Kafka-RS**: Simple offset tracking per consumer ID

```rust
// Basic consumer identification
let consumer_id = format!("consumer-{}", header.correlation_id);
```

**Benefits**:
- Easier to trace message flow
- No rebalancing complexity
- Clear ownership model

## Protocol Extensions for Learning

### Debug Logging

Every protocol operation is logged for educational visibility:

```rust
debug!("Processing {} request (correlation_id: {})", 
       match header.api_key {
           ApiKey::Produce => "PRODUCE",
           ApiKey::Fetch => "FETCH",
           ApiKey::Metadata => "METADATA",
           _ => "UNKNOWN",
       }, 
       header.correlation_id);
```

### Correlation ID Tracking

Each request-response pair is tracked:

```rust
// Request received
[DEBUG] Processing PRODUCE request (correlation_id: 123)

// Message processing  
[INFO] Creating new topic: test-topic
[DEBUG] Stored message at offset 0 in topic test-topic

// Response sent
[DEBUG] Sent response (87 bytes)
```

### Educational Error Handling

Errors are handled gracefully with educational logging:

```rust
match ProduceRequest::decode(buf) {
    Ok(request) => { /* process */ }
    Err(e) => {
        error!("Failed to decode produce request: {}", e);
        self.send_error_response(header.correlation_id, -1).await?;
    }
}
```

## Client Compatibility

### Tested Clients

- **KafkaJS**: Full compatibility with basic operations
- **kafka-node**: Should work (not extensively tested)
- **librdkafka**: Should work with appropriate configuration

### Configuration Requirements

For optimal compatibility with KafkaJS:

```javascript
const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['localhost:9092'],
  
  // Recommended settings for Kafka-RS
  connectionTimeout: 3000,
  requestTimeout: 30000,
  enforceRequestTimeout: false,
  
  // Disable features not implemented
  ssl: false,
  sasl: false,
});
```

### Consumer Configuration

```javascript
const consumer = kafka.consumer({
  groupId: 'my-group',
  
  // Work well with simplified implementation
  sessionTimeout: 30000,
  rebalanceTimeout: 60000,
  heartbeatInterval: 3000,
  
  // Single partition optimization  
  maxWaitTimeInMs: 5000,
  minBytes: 1,
  maxBytes: 1024 * 1024,
});
```

## Performance Characteristics

### Throughput Limitations

1. **Synchronous Processing**: No request pipelining
2. **Single Partition**: No parallel processing
3. **In-Memory Storage**: Limited by RAM
4. **Simple Protocol**: No compression or batching optimizations

### Latency Profile

- **Produce Latency**: ~1-5ms (synchronous storage)
- **Fetch Latency**: ~1-5ms (memory lookup)
- **Metadata Latency**: ~1ms (static response)

### Suitable Workloads

- **Learning and Development**: Perfect
- **Integration Testing**: Good for simple cases
- **Production**: Not suitable (by design)

## Future Protocol Extensions

### Additional APIs

1. **OffsetList (Key: 2)**: List available offsets
2. **LeaderAndIsr (Key: 4)**: Partition leadership info
3. **CreateTopics (Key: 19)**: Explicit topic creation
4. **DeleteTopics (Key: 20)**: Topic deletion
5. **DescribeGroups (Key: 15)**: Consumer group info

### Enhanced Features

1. **Message Headers**: Full header support in wire format
2. **Compression**: GZIP, Snappy, LZ4 support  
3. **Transactions**: Transactional producer protocol
4. **Schema Registry**: Confluent Schema Registry compatibility

### Implementation Pattern

```rust
// Add new API handler
ApiKey::NewApi => {
    self.handle_new_api_request(header, buf).await?;
}

// Implement request decoder
impl KafkaDecodable for NewApiRequest {
    fn decode(buf: &mut BytesMut) -> io::Result<Self> {
        // Protocol-specific decoding
    }
}

// Add response encoder  
async fn send_new_api_response(&mut self, correlation_id: i32) -> anyhow::Result<()> {
    // Build and send response
}
```

This protocol implementation demonstrates how a complex distributed system protocol can be simplified for educational purposes while maintaining compatibility with real clients.
