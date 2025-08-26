# Examples

This document provides practical examples of using Kafka-RS with various clients and scenarios.

## Basic Producer-Consumer Example

### 1. Start Kafka-RS Server

```bash
# Terminal 1: Start the server
cargo run

# Expected output:
# [INFO] Starting Kafka-RS server on 127.0.0.1:9092
# [INFO] This is an educational implementation of Kafka in Rust
# [INFO] Compatible with KafkaJS and other Kafka clients
```

### 2. Simple Producer (Node.js)

Create `examples/simple-producer.js`:

```javascript
import { Kafka } from 'kafkajs';

const kafka = new Kafka({
  clientId: 'simple-producer',
  brokers: ['localhost:9092'],
  logLevel: 'INFO', // See Kafka-RS logs
});

const producer = kafka.producer();

async function sendMessages() {
  console.log('🚀 Connecting producer...');
  await producer.connect();
  console.log('✅ Producer connected');

  for (let i = 0; i < 10; i++) {
    const message = {
      key: `user-${i}`,
      value: JSON.stringify({
        userId: i,
        message: `Hello from user ${i}`,
        timestamp: new Date().toISOString(),
      }),
    };

    console.log(`📤 Sending message ${i + 1}/10`);
    
    await producer.send({
      topic: 'user-events',
      messages: [message],
    });

    // Small delay to see individual messages in logs
    await new Promise(resolve => setTimeout(resolve, 1000));
  }

  console.log('✅ All messages sent');
  await producer.disconnect();
  console.log('👋 Producer disconnected');
}

sendMessages().catch(console.error);
```

### 3. Simple Consumer (Node.js)

Create `examples/simple-consumer.js`:

```javascript
import { Kafka } from 'kafkajs';

const kafka = new Kafka({
  clientId: 'simple-consumer',
  brokers: ['localhost:9092'],
  logLevel: 'INFO',
});

const consumer = kafka.consumer({ 
  groupId: 'user-events-group',
  sessionTimeout: 30000,
  heartbeatInterval: 3000,
});

async function consumeMessages() {
  console.log('🚀 Connecting consumer...');
  await consumer.connect();
  console.log('✅ Consumer connected');

  console.log('📝 Subscribing to user-events topic...');
  await consumer.subscribe({ topic: 'user-events', fromBeginning: true });

  console.log('👂 Listening for messages...');
  await consumer.run({
    eachMessage: async ({ topic, partition, message }) => {
      const value = JSON.parse(message.value.toString());
      
      console.log('📨 Received message:', {
        topic,
        partition,
        offset: message.offset,
        key: message.key?.toString(),
        value,
        timestamp: new Date(message.timestamp),
      });

      // Simulate message processing
      console.log(`   Processing user ${value.userId}...`);
      await new Promise(resolve => setTimeout(resolve, 500));
      console.log(`   ✅ Processed user ${value.userId}`);
    },
  });
}

// Graceful shutdown
process.on('SIGINT', async () => {
  console.log('\n🛑 Shutting down consumer...');
  await consumer.disconnect();
  console.log('👋 Consumer disconnected');
  process.exit(0);
});

consumeMessages().catch(console.error);
```

### 4. Running the Example

```bash
# Terminal 1: Start Kafka-RS server
cargo run

# Terminal 2: Start consumer (run first to catch all messages)
cd examples && node simple-consumer.js

# Terminal 3: Start producer
cd examples && node simple-producer.js
```

Expected output shows message flow from producer through Kafka-RS to consumer!

## Batch Processing Example

### High-Volume Producer

Create `examples/batch-producer.js`:

```javascript
import { Kafka } from 'kafkajs';

const kafka = new Kafka({
  clientId: 'batch-producer',
  brokers: ['localhost:9092'],
});

const producer = kafka.producer({
  maxInFlightRequests: 1,
  idempotent: false,
  transactionTimeout: 30000,
});

async function sendBatchMessages() {
  await producer.connect();
  
  const batchSize = 50;
  const totalMessages = 200;
  
  console.log(`📦 Sending ${totalMessages} messages in batches of ${batchSize}`);
  
  for (let batch = 0; batch < totalMessages / batchSize; batch++) {
    const messages = [];
    
    for (let i = 0; i < batchSize; i++) {
      const messageId = batch * batchSize + i;
      messages.push({
        key: `batch-${batch}-msg-${i}`,
        value: JSON.stringify({
          batchId: batch,
          messageId: messageId,
          data: `Message content ${messageId}`,
          generatedAt: new Date().toISOString(),
        }),
      });
    }

    console.log(`📤 Sending batch ${batch + 1}/${totalMessages / batchSize}`);
    const startTime = Date.now();
    
    await producer.send({
      topic: 'batch-processing',
      messages,
    });
    
    const duration = Date.now() - startTime;
    console.log(`   ✅ Batch sent in ${duration}ms`);
  }

  console.log('🎉 All batches sent successfully');
  await producer.disconnect();
}

sendBatchMessages().catch(console.error);
```

### Batch Consumer with Processing

Create `examples/batch-consumer.js`:

```javascript
import { Kafka } from 'kafkajs';

const kafka = new Kafka({
  clientId: 'batch-consumer',
  brokers: ['localhost:9092'],
});

const consumer = kafka.consumer({ 
  groupId: 'batch-processing-group',
  maxWaitTimeInMs: 1000,
  minBytes: 1024, // Wait for at least 1KB
  maxBytes: 1024 * 1024, // Max 1MB per fetch
});

let messageCount = 0;
let startTime = null;

async function consumeBatch() {
  await consumer.connect();
  await consumer.subscribe({ topic: 'batch-processing', fromBeginning: true });

  console.log('📦 Starting batch consumer...');
  startTime = Date.now();

  await consumer.run({
    eachBatch: async ({ batch, resolveOffset, heartbeat }) => {
      console.log(`📨 Processing batch: ${batch.messages.length} messages`);
      
      for (const message of batch.messages) {
        const value = JSON.parse(message.value.toString());
        
        // Simulate processing
        await processMessage(value);
        
        messageCount++;
        
        // Resolve offset to commit progress
        resolveOffset(message.offset);
      }

      // Send heartbeat to maintain consumer group membership
      await heartbeat();
      
      const elapsed = (Date.now() - startTime) / 1000;
      const rate = messageCount / elapsed;
      
      console.log(`   📊 Processed: ${messageCount} messages, Rate: ${rate.toFixed(2)} msg/sec`);
    },
  });
}

async function processMessage(data) {
  // Simulate some processing work
  if (data.messageId % 100 === 0) {
    console.log(`   🔄 Processing message ${data.messageId} from batch ${data.batchId}`);
  }
  
  // Simulate variable processing time
  await new Promise(resolve => setTimeout(resolve, Math.random() * 10));
}

process.on('SIGINT', async () => {
  console.log('\n📈 Final Statistics:');
  const totalTime = (Date.now() - startTime) / 1000;
  console.log(`   Total Messages: ${messageCount}`);
  console.log(`   Total Time: ${totalTime.toFixed(2)}s`);
  console.log(`   Average Rate: ${(messageCount / totalTime).toFixed(2)} msg/sec`);
  
  await consumer.disconnect();
  process.exit(0);
});

consumeBatch().catch(console.error);
```

## Multi-Topic Example

### Topic Manager Service

Create `examples/topic-manager.js`:

```javascript
import { Kafka } from 'kafkajs';

const kafka = new Kafka({
  clientId: 'topic-manager',
  brokers: ['localhost:9092'],
});

const admin = kafka.admin();

async function manageTopics() {
  await admin.connect();
  
  // Get cluster metadata
  console.log('🔍 Fetching cluster metadata...');
  const metadata = await admin.fetchTopicMetadata();
  
  console.log('📋 Cluster Information:');
  console.log(`   Brokers: ${metadata.brokers.length}`);
  metadata.brokers.forEach(broker => {
    console.log(`     - ${broker.nodeId}: ${broker.host}:${broker.port}`);
  });
  
  console.log(`   Topics: ${metadata.topics.length}`);
  metadata.topics.forEach(topic => {
    console.log(`     - ${topic.name} (${topic.partitions.length} partitions)`);
    topic.partitions.forEach(partition => {
      console.log(`       Partition ${partition.partitionId}: Leader ${partition.leader}`);
    });
  });

  await admin.disconnect();
}

manageTopics().catch(console.error);
```

### Multi-Topic Producer

Create `examples/multi-topic-producer.js`:

```javascript
import { Kafka } from 'kafkajs';

const kafka = new Kafka({
  clientId: 'multi-topic-producer',
  brokers: ['localhost:9092'],
});

const producer = kafka.producer();

const topics = {
  'user-events': ['signup', 'login', 'logout', 'profile_update'],
  'order-events': ['created', 'paid', 'shipped', 'delivered'],
  'system-events': ['startup', 'shutdown', 'error', 'warning'],
};

async function simulateEvents() {
  await producer.connect();
  
  console.log('🎭 Starting multi-topic event simulation...');
  
  // Send events continuously
  const interval = setInterval(async () => {
    const topicNames = Object.keys(topics);
    const randomTopic = topicNames[Math.floor(Math.random() * topicNames.length)];
    const eventTypes = topics[randomTopic];
    const randomEvent = eventTypes[Math.floor(Math.random() * eventTypes.length)];
    
    const event = {
      eventType: randomEvent,
      userId: Math.floor(Math.random() * 1000),
      sessionId: `session-${Math.floor(Math.random() * 100)}`,
      timestamp: new Date().toISOString(),
      data: generateEventData(randomTopic, randomEvent),
    };

    try {
      await producer.send({
        topic: randomTopic,
        messages: [{
          key: event.sessionId,
          value: JSON.stringify(event),
        }],
      });
      
      console.log(`📤 ${randomTopic}: ${randomEvent} (user: ${event.userId})`);
    } catch (error) {
      console.error('❌ Failed to send event:', error);
    }
  }, 2000);

  // Run for 30 seconds
  setTimeout(async () => {
    clearInterval(interval);
    console.log('⏰ Simulation complete');
    await producer.disconnect();
  }, 30000);
}

function generateEventData(topic, eventType) {
  const baseData = { source: topic, type: eventType };
  
  switch (topic) {
    case 'user-events':
      return { ...baseData, userAgent: 'Mozilla/5.0...', ip: '192.168.1.1' };
    case 'order-events':
      return { ...baseData, orderId: `order-${Math.floor(Math.random() * 10000)}`, amount: Math.floor(Math.random() * 500) + 10 };
    case 'system-events':
      return { ...baseData, component: 'kafka-rs', severity: eventType === 'error' ? 'high' : 'low' };
    default:
      return baseData;
  }
}

simulateEvents().catch(console.error);
```

### Multi-Topic Consumer

Create `examples/multi-topic-consumer.js`:

```javascript
import { Kafka } from 'kafkajs';

const kafka = new Kafka({
  clientId: 'multi-topic-consumer',
  brokers: ['localhost:9092'],
});

const consumer = kafka.consumer({ groupId: 'analytics-group' });

const eventProcessors = {
  'user-events': processUserEvent,
  'order-events': processOrderEvent,
  'system-events': processSystemEvent,
};

async function consumeAllEvents() {
  await consumer.connect();
  
  // Subscribe to all topics
  const topics = Object.keys(eventProcessors);
  console.log(`📝 Subscribing to topics: ${topics.join(', ')}`);
  
  for (const topic of topics) {
    await consumer.subscribe({ topic, fromBeginning: true });
  }

  await consumer.run({
    eachMessage: async ({ topic, partition, message }) => {
      const event = JSON.parse(message.value.toString());
      
      console.log(`📨 ${topic}:${partition}@${message.offset} - ${event.eventType}`);
      
      // Process based on topic
      const processor = eventProcessors[topic];
      if (processor) {
        await processor(event, { topic, partition, offset: message.offset });
      } else {
        console.log(`❓ No processor for topic: ${topic}`);
      }
    },
  });
}

async function processUserEvent(event, metadata) {
  console.log(`   👤 User ${event.userId}: ${event.eventType}`);
  
  // Simulate user analytics
  if (event.eventType === 'signup') {
    console.log(`   📈 New user registered: ${event.userId}`);
  }
}

async function processOrderEvent(event, metadata) {
  console.log(`   🛒 Order event: ${event.eventType} ($${event.data.amount || 0})`);
  
  // Simulate order tracking
  if (event.eventType === 'paid') {
    console.log(`   💰 Payment processed for order: ${event.data.orderId}`);
  }
}

async function processSystemEvent(event, metadata) {
  console.log(`   🖥️  System: ${event.eventType} (${event.data.severity} severity)`);
  
  // Simulate monitoring
  if (event.eventType === 'error') {
    console.log(`   🚨 ALERT: System error in ${event.data.component}`);
  }
}

// Graceful shutdown
process.on('SIGINT', async () => {
  console.log('\n🛑 Shutting down multi-topic consumer...');
  await consumer.disconnect();
  console.log('👋 Consumer disconnected');
  process.exit(0);
});

consumeAllEvents().catch(console.error);
```

## Error Handling Example

Create `examples/error-handling.js`:

```javascript
import { Kafka } from 'kafkajs';

const kafka = new Kafka({
  clientId: 'error-handling-example',
  brokers: ['localhost:9092'],
  retry: {
    initialRetryTime: 100,
    retries: 3,
  },
});

async function demonstrateErrorHandling() {
  const producer = kafka.producer();
  const consumer = kafka.consumer({ groupId: 'error-group' });

  try {
    // Test connection errors
    console.log('🔄 Testing connection...');
    await producer.connect();
    console.log('✅ Producer connected successfully');

    // Test topic auto-creation
    console.log('📝 Testing topic auto-creation...');
    await producer.send({
      topic: 'error-test-topic',
      messages: [{ value: 'test message' }],
    });
    console.log('✅ Topic created and message sent');

    // Test consumer subscription
    await consumer.connect();
    await consumer.subscribe({ topic: 'error-test-topic' });
    
    console.log('👂 Testing message consumption...');
    let messageReceived = false;
    
    await consumer.run({
      eachMessage: async ({ topic, message }) => {
        console.log(`📨 Received: ${message.value.toString()}`);
        messageReceived = true;
        
        // Stop consumer after first message
        await consumer.stop();
      },
    });

    // Wait a bit for message processing
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    if (messageReceived) {
      console.log('✅ Error handling test completed successfully');
    } else {
      console.log('❌ No message received');
    }

  } catch (error) {
    console.error('❌ Error during test:', error);
    
    // Handle specific error types
    if (error.type === 'UNKNOWN_TOPIC_OR_PARTITION') {
      console.log('   Topic does not exist and auto-creation failed');
    } else if (error.type === 'NETWORK_EXCEPTION') {
      console.log('   Network connection failed');
    } else if (error.type === 'REQUEST_TIMED_OUT') {
      console.log('   Request timed out');
    } else {
      console.log(`   Unexpected error type: ${error.type}`);
    }
  } finally {
    try {
      await producer.disconnect();
      await consumer.disconnect();
      console.log('👋 Cleanup completed');
    } catch (cleanupError) {
      console.error('❌ Cleanup error:', cleanupError);
    }
  }
}

demonstrateErrorHandling().catch(console.error);
```

## Performance Testing Example

Create `examples/performance-test.js`:

```javascript
import { Kafka } from 'kafkajs';

const kafka = new Kafka({
  clientId: 'performance-test',
  brokers: ['localhost:9092'],
});

async function performanceTest() {
  const producer = kafka.producer();
  const consumer = kafka.consumer({ groupId: 'perf-test-group' });
  
  await producer.connect();
  await consumer.connect();
  await consumer.subscribe({ topic: 'performance-test' });

  // Performance metrics
  let messagesSent = 0;
  let messagesReceived = 0;
  let startTime = Date.now();
  let sendStartTime = null;
  let receiveStartTime = null;

  // Start consumer
  const consumerPromise = consumer.run({
    eachMessage: async ({ message }) => {
      if (messagesReceived === 0) {
        receiveStartTime = Date.now();
      }
      
      messagesReceived++;
      
      if (messagesReceived % 100 === 0) {
        const elapsed = (Date.now() - receiveStartTime) / 1000;
        const rate = messagesReceived / elapsed;
        console.log(`📨 Received: ${messagesReceived}, Rate: ${rate.toFixed(2)} msg/sec`);
      }
    },
  });

  // Wait a bit for consumer to be ready
  await new Promise(resolve => setTimeout(resolve, 1000));

  // Send messages
  console.log('📤 Starting message production...');
  sendStartTime = Date.now();
  
  const totalMessages = 1000;
  const batchSize = 10;
  
  for (let i = 0; i < totalMessages; i += batchSize) {
    const messages = [];
    
    for (let j = 0; j < batchSize && i + j < totalMessages; j++) {
      messages.push({
        key: `perf-test-${i + j}`,
        value: JSON.stringify({
          id: i + j,
          timestamp: Date.now(),
          data: 'x'.repeat(100), // 100 byte payload
        }),
      });
    }

    await producer.send({
      topic: 'performance-test',
      messages,
    });

    messagesSent += messages.length;
    
    if (messagesSent % 100 === 0) {
      const elapsed = (Date.now() - sendStartTime) / 1000;
      const rate = messagesSent / elapsed;
      console.log(`📤 Sent: ${messagesSent}/${totalMessages}, Rate: ${rate.toFixed(2)} msg/sec`);
    }
  }

  console.log(`📊 Production complete: ${messagesSent} messages sent`);

  // Wait for all messages to be consumed
  const timeout = 30000; // 30 seconds
  const checkInterval = 1000; // 1 second
  let waited = 0;
  
  while (messagesReceived < messagesSent && waited < timeout) {
    await new Promise(resolve => setTimeout(resolve, checkInterval));
    waited += checkInterval;
  }

  // Final statistics
  const totalTime = (Date.now() - startTime) / 1000;
  const sendTime = (Date.now() - sendStartTime) / 1000;
  const receiveTime = messagesReceived > 0 ? (Date.now() - receiveStartTime) / 1000 : 0;

  console.log('\n📈 Performance Test Results:');
  console.log(`   Messages Sent: ${messagesSent}`);
  console.log(`   Messages Received: ${messagesReceived}`);
  console.log(`   Total Time: ${totalTime.toFixed(2)}s`);
  console.log(`   Send Rate: ${(messagesSent / sendTime).toFixed(2)} msg/sec`);
  console.log(`   Receive Rate: ${receiveTime > 0 ? (messagesReceived / receiveTime).toFixed(2) : 0} msg/sec`);
  console.log(`   End-to-End Rate: ${(messagesReceived / totalTime).toFixed(2)} msg/sec`);

  await producer.disconnect();
  await consumer.disconnect();
}

performanceTest().catch(console.error);
```

## Package.json for Examples

Create `examples/package.json`:

```json
{
  "name": "kafka-rs-examples",
  "version": "1.0.0",
  "description": "Examples for Kafka-RS educational implementation",
  "type": "module",
  "dependencies": {
    "kafkajs": "^2.2.4"
  },
  "scripts": {
    "simple": "node simple-producer.js",
    "consumer": "node simple-consumer.js",
    "batch": "node batch-producer.js",
    "batch-consumer": "node batch-consumer.js",
    "multi": "node multi-topic-producer.js",
    "multi-consumer": "node multi-topic-consumer.js",
    "topics": "node topic-manager.js",
    "error-test": "node error-handling.js",
    "perf-test": "node performance-test.js"
  }
}
```

## Running All Examples

```bash
# Setup
mkdir examples
cd examples
npm init -y
npm install kafkajs

# Copy example files above into examples/ directory

# Terminal 1: Start Kafka-RS
cd .. && cargo run

# Terminal 2: Run any example
cd examples
npm run simple      # Basic producer
npm run consumer     # Basic consumer
npm run batch        # Batch processing
npm run multi        # Multi-topic simulation
npm run topics       # Topic management
npm run error-test   # Error handling
npm run perf-test    # Performance testing
```

These examples demonstrate the educational value of Kafka-RS by showing:

- **Basic Concepts**: Producer-consumer patterns
- **Batch Processing**: High-volume message handling
- **Multi-Topic**: Complex event-driven architectures
- **Error Handling**: Robust client implementations
- **Performance**: Understanding system limitations and characteristics

Each example includes detailed logging that correlates with Kafka-RS server logs, making it easy to understand the internal message flow and system behavior.
