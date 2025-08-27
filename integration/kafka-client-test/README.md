# Kafka-RS Integration Test Suite 🚀

## Overview

This comprehensive integ## 📊 Test Suite Components

### 🔧 Test 1: API Versions Negotiation

- **What it does**: Verifies that clients can discover supported APIs
- **Learning focus**: Understanding how Kafka clients establish compatibility  
- **Key insight**: This is the first thing ANY Kafka client does when connecting

### 📋 Test 2: Metadata Discovery

- **What it does**: Requests cluster topology information
- **Learning focus**: How clients discover brokers, topics, and partitions
- **Key insight**: Metadata drives all client routing decisions

### 🏗️ Test 3: Topic Creation

- **What it does**: Creates test topics with specific configurations
- **Learning focus**: Topic management and configuration options
- **Key insight**: Topics are the fundamental organizing unit in Kafka

### 📤 Test 4: Message Production

- **What it does**: Sends structured messages to topics
- **Learning focus**: How producers work and message formatting
- **Key insight**: Messages have keys, values, and metadata

### 📥 Test 5: Message Consumption

- **What it does**: Reads and processes messages from topics
- **Learning focus**: Consumer mechanics and offset management
- **Key insight**: Consumers track their progress with offsets

### 👥 Test 6: Consumer Group Coordination

- **What it does**: Tests consumer group membership and coordination
- **Learning focus**: How multiple consumers work together
- **Key insight**: Consumer groups enable scalability and fault toleranceon test suite validates our Kafka-RS server implementation by testing it with real **KafkaJS** clients. The tests are specifically designed for **learning by doing** - each test demonstrates key Kafka concepts while verifying that our server works correctly.

## 🎯 Learning Objectives

By running and studying these tests, you'll learn:

- **Kafka Client-Server Communication**: How clients negotiate APIs and establish connections
- **Topic Management**: Creating, configuring, and organizing data streams
- **Message Production**: Sending structured data to Kafka topics
- **Message Consumption**: Reading and processing messages with offset tracking
- **Consumer Groups**: Coordinating multiple consumers for scalability
- **Metadata System**: How Kafka organizes brokers, topics, and partitions
- **Error Handling**: Proper debugging and troubleshooting techniques

## 📁 Project Structure

```text
integration/kafka-client-test/
├── test.js                 # Main test suite with comprehensive scenarios
├── kafka-client.js         # Centralized KafkaJS client configuration
├── package.json           # Dependencies and test scripts
├── utils/
│   ├── create-topics.js   # Topic creation and management utilities
│   ├── debug-metadata.js  # Metadata inspection and debugging tools
│   └── logger.js          # Enhanced logging for better learning experience
└── README.md              # This comprehensive guide
```

## 🛠️ Prerequisites

Before running the integration tests, ensure you have:

1. **Node.js** (v16 or later)
2. **Your Kafka-RS server** running on `localhost:9092`
3. **Network connectivity** between the test client and server

## 🚀 Quick Start

### 1. Install Dependencies

```bash
cd integration/kafka-client-test
npm install
```

### 2. Start Your Kafka-RS Server

Make sure your server is running and listening on port 9092:

```bash
# In the project root
cargo run --release
```

### 3. Run the Integration Tests

```bash
# Run the complete test suite
npm test

# Or run directly with Node.js
node test.js
```

## 📊 Test Suite Components

### 🔧 Test 1: API Versions Negotiation
- **What it does**: Verifies that clients can discover supported APIs
- **Learning focus**: Understanding how Kafka clients establish compatibility
- **Key insight**: This is the first thing ANY Kafka client does when connecting

### � Test 2: Metadata Discovery
- **What it does**: Requests cluster topology information
- **Learning focus**: How clients discover brokers, topics, and partitions
- **Key insight**: Metadata drives all client routing decisions

### 🏗️ Test 3: Topic Creation
- **What it does**: Creates test topics with specific configurations
- **Learning focus**: Topic management and configuration options
- **Key insight**: Topics are the fundamental organizing unit in Kafka

### � Test 4: Message Production
- **What it does**: Sends structured messages to topics
- **Learning focus**: How producers work and message formatting
- **Key insight**: Messages have keys, values, and metadata

### 📥 Test 5: Message Consumption
- **What it does**: Reads and processes messages from topics
- **Learning focus**: Consumer mechanics and offset management
- **Key insight**: Consumers track their progress with offsets

### 👥 Test 6: Consumer Group Coordination
- **What it does**: Tests consumer group membership and coordination
- **Learning focus**: How multiple consumers work together
- **Key insight**: Consumer groups enable scalability and fault tolerance

## 🔍 Understanding the Output

The test suite provides rich, color-coded output designed for learning:

```bash
🚀 Starting Kafka-RS Integration Test Suite
🎯 Server: localhost:9092
📝 Topics: test-topic-1, test-topic-2, learning-topic
👥 Consumer Group: kafka-rs-test-group

📡 Test 1: API Versions Negotiation
Learning: Every Kafka client first asks "what APIs do you support?"
✅ ApiVersions negotiation successful
💡 Your server correctly handled the ApiVersions request!

📋 Test 2: Metadata Discovery
Learning: Clients need to know about topics, partitions, and brokers
✅ Metadata request successful
📊 Discovered 3 topics
```

## 🛠️ Debugging and Troubleshooting

### Common Issues and Solutions

#### 1. Connection Refused

**Symptom**: `ECONNREFUSED localhost:9092`

**Solution**:

- Verify your Kafka-RS server is running
- Check that it's listening on port 9092
- Look for server startup logs

#### 2. Unknown API Key Errors

**Symptom**: `Unknown API key: 18`

**Solution**:

- Ensure ApiVersions API is implemented in your server
- Check that the server properly handles API key 18

#### 3. Metadata Request Failures

**Symptom**: Metadata requests timeout or fail

**Solution**:

- Verify Metadata API implementation
- Check topic creation functionality
- Review server logs for protocol errors

### Debug Mode

Enable detailed debugging:

```bash
# Enable debug logging
DEBUG=1 node test.js

# Or with development environment
NODE_ENV=development node test.js
```

## 📚 Educational Deep Dives

### Understanding KafkaJS Configuration

The test suite uses carefully chosen KafkaJS configuration options:

```javascript
const kafka = new Kafka({
  clientId: 'kafka-rs-integration-test',    // Identifies our client
  brokers: ['localhost:9092'],              // Where to connect
  connectionTimeout: 10000,                 // How long to wait for connection
  requestTimeout: 10000,                    // How long to wait for responses
  retry: {                                  // Retry policy for reliability
    initialRetryTime: 100,
    retries: 5
  }
});
```

### Message Structure Deep Dive

Messages in Kafka have a specific structure that our tests demonstrate:

```javascript
{
  key: 'test-key-1',                        // Optional message key for partitioning
  value: JSON.stringify({                   // Message payload (can be any format)
    source: 'kafka-rs-integration-test',
    timestamp: '2025-08-27T10:30:00Z',
    messageId: 1,
    content: 'Learning message content'
  })
}
```

### Consumer Group Mechanics

Consumer groups are a powerful Kafka feature our tests explore:

- **Group Membership**: Multiple consumers can join the same group
- **Partition Assignment**: Kafka automatically distributes partitions among consumers
- **Offset Tracking**: Each consumer group tracks its progress independently
- **Rebalancing**: When consumers join/leave, Kafka redistributes work

## 🎛️ Customization Options

### Environment Variables

```bash
# Server configuration
KAFKA_HOST=localhost        # Default: localhost
KAFKA_PORT=9092            # Default: 9092

# Logging
NODE_ENV=development       # Enables debug logging
DEBUG=1                    # Enables detailed debug output
```

### Test Configuration

Modify `TEST_CONFIG` in `test.js` to experiment:

```javascript
const TEST_CONFIG = {
  TOPICS: ['my-topic-1', 'my-topic-2'],     // Custom topic names
  CONSUMER_GROUP: 'my-test-group',          // Custom group name
  MESSAGES_TO_SEND: 10,                     // Number of test messages
  TEST_TIMEOUT: 60000,                      // Test timeout (ms)
};
```

## 📈 Extending the Tests

### Adding New Test Scenarios

1. **Create a new test method** in the `KafkaIntegrationTests` class
2. **Follow the naming convention**: `test[FeatureName]()`
3. **Include learning comments** to explain what the test demonstrates
4. **Add the test to `runAllTests()`** method

Example:

```javascript
async testCustomFeature() {
  logger.info('🧪 Test X: Custom Feature');
  logger.info('Learning: What this test teaches...');
  
  try {
    // Your test logic here
    this.testResults.passed++;
  } catch (error) {
    logger.error('❌ Custom feature test failed:', error.message);
    this.testResults.failed++;
    this.testResults.errors.push({ test: 'CustomFeature', error: error.message });
  }
}
```

## 🎓 Learning Resources

### Recommended Reading Order

1. **Start with the main `test.js`** - Read the comments and understand the flow
2. **Explore `kafka-client.js`** - See how KafkaJS is configured
3. **Study `utils/logger.js`** - Understand the enhanced logging system
4. **Examine `utils/create-topics.js`** - Learn about topic management
5. **Dive into `utils/debug-metadata.js`** - Understand metadata inspection

### Key Concepts to Master

- **Client-Server Protocol**: How Kafka clients communicate with servers
- **Topic-Partition Model**: How data is organized and distributed
- **Producer Semantics**: Message sending patterns and guarantees
- **Consumer Semantics**: Message reading patterns and offset management
- **Error Handling**: Proper ways to handle and debug issues

## 🤝 Contributing

When adding new tests or features:

1. **Maintain the learning focus** - Add educational comments
2. **Follow the existing patterns** - Consistent code structure
3. **Update documentation** - Keep this README current
4. **Test thoroughly** - Ensure new code works with the server

## 📝 License

This integration test suite is part of the kafka-rs learning project and follows the same license as the main project.

---

**Happy Learning!** 🎉 These tests are designed to make learning Kafka protocols engaging and practical. Run them, break them, modify them, and learn by doing!
