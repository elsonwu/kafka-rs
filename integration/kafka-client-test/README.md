# Kafka Client Integration Tests

This directory contains integration tests that verify the compatibility of our Rust-based Kafka server implementation with real JavaScript Kafka clients.

## Overview

The integration test uses [KafkaJS](https://kafka.js.org/), a popular JavaScript Kafka client library, to:

1. **Producer Test**: Send messages to our Kafka server
2. **Consumer Test**: Receive and verify messages from our Kafka server  
3. **Metadata Test**: Fetch topic metadata (if supported)

## Test Scenarios

### Producer Test
- Connects to the Kafka server running on `localhost:9092`
- Creates a topic called `integration-test-topic`
- Sends 3 test messages with different keys and values
- Verifies successful message delivery

### Consumer Test
- Connects to the same Kafka server
- Subscribes to the test topic from the beginning
- Consumes all messages sent by the producer
- Verifies message integrity (keys and values match)
- Uses consumer group `integration-test-group`

### Metadata Test
- Fetches topic metadata using the admin client
- Verifies server responds to metadata requests correctly

## Running Locally

### Prerequisites
- Node.js 18+ installed
- Kafka-RS server built and ready to run

### Steps

1. **Install dependencies**:
   ```bash
   cd integration/kafka-client-test
   npm install
   ```

2. **Start the Kafka server** (in another terminal):
   ```bash
   cd /path/to/kafka-rs
   cargo run --release -- --port 9092
   ```

3. **Run the integration test**:
   ```bash
   npm test
   ```

### Expected Output

If successful, you should see output like:
```
🎯 Starting Kafka Client Integration Test
📡 Connecting to Kafka broker: localhost:9092
📝 Test topic: integration-test-topic
👥 Consumer group: integration-test-group

🚀 Testing Kafka Producer...
✅ Producer connected successfully
✅ Sent 3 messages: [...]
✅ Producer disconnected successfully

📥 Testing Kafka Consumer...
✅ Consumer connected successfully
✅ Subscribed to topic: integration-test-topic
📩 Received message: {...}
📩 Received message: {...}
📩 Received message: {...}
✅ Received 3 messages (expected 3)
✅ Message 0 verified successfully
✅ Message 1 verified successfully
✅ Message 2 verified successfully
✅ Consumer disconnected successfully

🎉 Integration Test Results:
   ✅ Producer: Successfully sent 3 messages
   ✅ Consumer: Successfully received 3 messages
   ✅ Server compatibility: Verified with real Kafka JavaScript client

🎯 All integration tests passed! Kafka-RS server is compatible with KafkaJS client.
```

## CI/CD Integration

This test is automatically run in GitHub Actions as part of the `kafka-client-integration` job. The CI pipeline:

1. Builds the Kafka server in release mode
2. Starts the server in the background
3. Installs Node.js dependencies
4. Runs the integration test
5. Reports results and cleans up

## Troubleshooting

### Common Issues

**Server Connection Failed**
- Ensure the Kafka server is running on port 9092
- Check that no other process is using port 9092
- Verify the server starts without errors

**Messages Not Received**
- Check server logs for any protocol errors
- Verify the producer successfully sent messages
- Ensure the consumer is subscribing to the correct topic

**Timeout Errors**
- The test waits up to 15 seconds for messages
- If your server is slow to start, increase the wait time
- Check for any blocking operations in the server

### Debug Mode

To run with more verbose logging, modify the `logLevel` in `test.js`:
```javascript
const kafka = new Kafka({
    // ... other config
    logLevel: logLevel.INFO, // or logLevel.DEBUG
});
```

## Dependencies

- **kafkajs**: Modern Apache Kafka client for Node.js
  - Compatible with Apache Kafka 0.10+
  - Supports producers, consumers, and admin operations
  - Well-maintained and widely used in production

## Future Enhancements

Potential improvements to the integration tests:

1. **Multiple Topics**: Test with multiple topics simultaneously
2. **Concurrent Clients**: Test with multiple producers and consumers
3. **Error Scenarios**: Test error handling and edge cases
4. **Performance Tests**: Measure throughput and latency
5. **Schema Registry**: Test with Avro schemas (if implemented)
6. **Transactions**: Test transactional producers (if implemented)
