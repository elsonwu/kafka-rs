# Getting Started

## Prerequisites

- Rust 1.70+ installed
- Node.js 16+ (for testing with JavaScript client)
- Git

## Installation

1. **Clone the repository:**
```bash
git clone https://github.com/elsonwu/kafka-rs.git
cd kafka-rs
```

2. **Build the project:**
```bash
cargo build
```

3. **Run tests:**
```bash
cargo test
```

## Running the Server

### Basic Usage
```bash
cargo run
```

This starts the Kafka-RS server on `localhost:9092` (default Kafka port).

### With Debug Logging
```bash
RUST_LOG=debug cargo run
```

### Custom Configuration
```bash
cargo run -- --host 0.0.0.0 --port 9093
```

## Testing with JavaScript Client

### Install KafkaJS
```bash
npm install kafkajs
```

### Basic Producer Example
```javascript
// producer.js
import { Kafka } from 'kafkajs';

const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['localhost:9092'],
});

const producer = kafka.producer();

async function run() {
  await producer.connect();
  
  console.log('Sending message...');
  await producer.send({
    topic: 'test-topic',
    messages: [
      {
        key: 'user-1',
        value: JSON.stringify({ 
          userId: 1, 
          action: 'login',
          timestamp: Date.now()
        }),
      },
    ],
  });
  
  console.log('Message sent!');
  await producer.disconnect();
}

run().catch(console.error);
```

### Basic Consumer Example
```javascript
// consumer.js
import { Kafka } from 'kafkajs';

const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['localhost:9092'],
});

const consumer = kafka.consumer({ groupId: 'test-group' });

async function run() {
  await consumer.connect();
  await consumer.subscribe({ topic: 'test-topic' });

  await consumer.run({
    eachMessage: async ({ topic, partition, message }) => {
      console.log({
        topic,
        partition,
        offset: message.offset,
        key: message.key?.toString(),
        value: message.value?.toString(),
      });
    },
  });
}

run().catch(console.error);
```

### Running the Examples

1. **Start the Kafka-RS server:**
```bash
cargo run
```

2. **In another terminal, run the consumer:**
```bash
node consumer.js
```

3. **In a third terminal, run the producer:**
```bash
node producer.js
```

You should see the message flow from producer to consumer!

## Expected Output

### Server Logs
```
[INFO] Starting Kafka-RS server on 127.0.0.1:9092
[INFO] Received connection from 127.0.0.1:52341
[DEBUG] Processing produce request for topic: test-topic
[DEBUG] Created new topic: test-topic with 1 partition
[DEBUG] Stored message with offset 0 in partition 0
[INFO] Producer sent 1 message(s) to test-topic
[DEBUG] Processing fetch request from consumer: test-group
[DEBUG] Fetching messages from offset 0 for topic: test-topic
[INFO] Consumer test-group fetched 1 message(s) from test-topic
```

### Consumer Output
```javascript
{
  topic: 'test-topic',
  partition: 0,
  offset: '0',
  key: 'user-1',
  value: '{"userId":1,"action":"login","timestamp":1703123456789}'
}
```

## Understanding the Flow

### What Happens When You Send a Message

1. **Client Connection**: KafkaJS connects to Kafka-RS via TCP
2. **Topic Creation**: If the topic doesn't exist, it's created automatically
3. **Message Storage**: Message is stored in memory with an incrementing offset
4. **Acknowledgment**: Producer receives confirmation

### What Happens When You Consume Messages

1. **Consumer Registration**: Consumer subscribes to the topic
2. **Offset Management**: Consumer tracks its position in the topic
3. **Message Retrieval**: Messages are fetched from the stored offset
4. **Offset Update**: Consumer position advances

## Automated Integration Testing

For comprehensive testing, Kafka-RS includes automated integration tests using real Kafka JavaScript clients.

### Running the Integration Test

The integration test verifies protocol compatibility with actual Kafka clients:

```bash
# Install Node.js dependencies
cd integration/kafka-client-test
npm install

# Start the server (in another terminal)
cargo run --release -- --port 9092

# Run the automated test
npm test
```

### What the Integration Test Does

The automated test performs a complete producer-consumer cycle:

1. **Producer Test**: 
   - Connects to Kafka-RS server
   - Creates topic `integration-test-topic`
   - Sends 3 test messages with different keys and values
   - Verifies successful delivery

2. **Consumer Test**:
   - Subscribes to the test topic from beginning  
   - Consumes all messages sent by producer
   - Verifies message integrity (keys and values match)
   - Uses consumer group `integration-test-group`

3. **Metadata Test**:
   - Fetches topic metadata using admin client
   - Verifies server responds correctly

### Integration Test Output

When successful, you'll see:

```
🎯 Starting Kafka Client Integration Test
📡 Connecting to Kafka broker: localhost:9092

🚀 Testing Kafka Producer...
✅ Producer connected successfully
✅ Sent 3 messages
✅ Producer disconnected successfully

📥 Testing Kafka Consumer...  
✅ Consumer connected successfully
✅ Subscribed to topic: integration-test-topic
📩 Received message: {"key":"key1","value":"Hello from KafkaJS client!"}
📩 Received message: {"key":"key2","value":"Testing Kafka-RS server compatibility"}
📩 Received message: {"key":"key3","value":"{\"test\":true,\"timestamp\":...}"}
✅ Received 3 messages (expected 3)
✅ Consumer disconnected successfully

🎉 Integration Test Results:
   ✅ Producer: Successfully sent 3 messages
   ✅ Consumer: Successfully received 3 messages
   ✅ Server compatibility: Verified with real Kafka JavaScript client

🎯 All integration tests passed! Kafka-RS server is compatible with KafkaJS client.
```

### CI/CD Integration

This integration test runs automatically in GitHub Actions as part of the `kafka-client-integration` job, ensuring ongoing compatibility with real Kafka clients.

## Troubleshooting

### Connection Refused
```
Error: Connection refused (os error 61)
```
**Solution**: Make sure Kafka-RS server is running on the correct port.

### Topic Not Found
```
Error: Topic 'test-topic' not found
```
**Solution**: Topics are auto-created on first produce. Send a message first.

### Port Already in Use
```
Error: Address already in use (os error 48)
```
**Solution**: 
- Stop any other Kafka instances: `pkill -f kafka`
- Use a different port: `cargo run -- --port 9093`

## Development Workflow

### Making Changes
1. Edit the code
2. Run tests: `cargo test`
3. Test manually with the examples above
4. Check logs for detailed operation info

### Adding Features
1. Start with domain models in `src/domain/`
2. Add application services in `src/application/`
3. Implement infrastructure in `src/infrastructure/`
4. Test with JavaScript client

## Next Steps

- Explore the [Domain Model](domain-model.md) to understand the business logic
- Read [API Reference](api-reference.md) for detailed protocol information
- Check out [Examples](examples.md) for more complex scenarios
- Learn about [Protocol Implementation](protocol.md) details
