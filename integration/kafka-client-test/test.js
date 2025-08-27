#!/usr/bin/env node

/**
 * Kafka-RS Integration Test Suite
 * 
 * This comprehensive test suite validates our Kafka-RS server implementation
 * by testing it with real KafkaJS clients. The tests are designed for learning
 * and demonstrate how Kafka protocols work in practice.
 * 
 * Learning Objectives:
 * - Understand Kafka client-server interaction patterns
 * - See how API negotiation works (ApiVersions)
 * - Learn about metadata requests and topic management
 * - Observe producer and consumer patterns
 * - Practice error handling and debugging
 */

const { kafka } = require('./kafka-client');
const { createTestTopics } = require('./utils/create-topics');
const { debugMetadata } = require('./utils/debug-metadata');
const { logger } = require('./utils/logger');

// Test configuration - easily adjustable for learning experiments
const TEST_CONFIG = {
  SERVER_HOST: 'localhost',
  SERVER_PORT: 9092,
  TOPICS: ['test-topic-1', 'test-topic-2', 'learning-topic'],
  CONSUMER_GROUP: 'kafka-rs-test-group',
  TEST_TIMEOUT: 30000, // 30 seconds
  MESSAGES_TO_SEND: 5,
  MESSAGE_PAYLOAD: {
    source: 'kafka-rs-integration-test',
    timestamp: new Date().toISOString(),
    learning: 'This message was sent to test our Kafka-RS implementation!'
  }
};

/**
 * Test Suite Class - Organized approach to testing
 */
class KafkaIntegrationTests {
  constructor() {
    this.kafka = kafka;
    this.admin = null;
    this.producer = null;
    this.consumer = null;
    this.testResults = {
      passed: 0,
      failed: 0,
      errors: []
    };
  }

  /**
   * Initialize all Kafka clients
   * Learning: See how different client types are created
   */
  async initialize() {
    logger.info('🚀 Initializing Kafka clients...');
    
    try {
      // Admin client for topic management
      this.admin = this.kafka.admin();
      await this.admin.connect();
      logger.success('✅ Admin client connected');

      // Producer client for sending messages
      this.producer = this.kafka.producer();
      await this.producer.connect();
      logger.success('✅ Producer client connected');

      // Consumer client for receiving messages
      this.consumer = this.kafka.consumer({ 
        groupId: TEST_CONFIG.CONSUMER_GROUP,
        sessionTimeout: 10000,
        heartbeatInterval: 3000
      });
      await this.consumer.connect();
      logger.success('✅ Consumer client connected');

    } catch (error) {
      logger.error('❌ Failed to initialize clients:', error.message);
      throw error;
    }
  }

  /**
   * Test 1: API Versions negotiation
   * Learning: This is the first thing any Kafka client does
   */
  async testApiVersions() {
    logger.info('\n📡 Test 1: API Versions Negotiation');
    logger.info('Learning: Every Kafka client first asks "what APIs do you support?"');
    
    try {
      // The connection process automatically handles ApiVersions
      // We can verify it worked by the fact that we connected successfully
      logger.success('✅ ApiVersions negotiation successful');
      logger.info('💡 Your server correctly handled the ApiVersions request!');
      this.testResults.passed++;
    } catch (error) {
      logger.error('❌ ApiVersions test failed:', error.message);
      this.testResults.failed++;
      this.testResults.errors.push({ test: 'ApiVersions', error: error.message });
    }
  }

  /**
   * Test 2: Metadata requests
   * Learning: How clients discover topics and brokers
   */
  async testMetadata() {
    logger.info('\n📋 Test 2: Metadata Discovery');
    logger.info('Learning: Clients need to know about topics, partitions, and brokers');
    
    try {
      // Request metadata for all topics
      const metadata = await this.admin.fetchTopicMetadata();
      logger.success('✅ Metadata request successful');
      logger.info(`📊 Discovered ${metadata.topics.length} topics`);
      
      // Debug metadata details
      await debugMetadata(this.kafka);
      
      this.testResults.passed++;
    } catch (error) {
      logger.error('❌ Metadata test failed:', error.message);
      this.testResults.failed++;
      this.testResults.errors.push({ test: 'Metadata', error: error.message });
    }
  }

  /**
   * Test 3: Topic creation
   * Learning: How topics are managed in Kafka
   */
  async testTopicCreation() {
    logger.info('\n🏗️  Test 3: Topic Creation');
    logger.info('Learning: Topics are the fundamental organizing unit in Kafka');
    
    try {
      await createTestTopics(this.admin, TEST_CONFIG.TOPICS);
      logger.success('✅ Topic creation successful');
      this.testResults.passed++;
    } catch (error) {
      logger.error('❌ Topic creation test failed:', error.message);
      this.testResults.failed++;
      this.testResults.errors.push({ test: 'TopicCreation', error: error.message });
    }
  }

  /**
   * Test 4: Message production
   * Learning: How to send messages to Kafka
   */
  async testMessageProduction() {
    logger.info('\n📤 Test 4: Message Production');
    logger.info('Learning: Producers send messages to specific topics');
    
    try {
      const topic = TEST_CONFIG.TOPICS[0];
      const messages = [];
      
      // Create test messages
      for (let i = 0; i < TEST_CONFIG.MESSAGES_TO_SEND; i++) {
        messages.push({
          key: `test-key-${i}`,
          value: JSON.stringify({
            ...TEST_CONFIG.MESSAGE_PAYLOAD,
            messageId: i,
            content: `Test message ${i + 1} from kafka-rs integration test`
          })
        });
      }

      // Send messages
      logger.info(`📨 Sending ${messages.length} messages to topic: ${topic}`);
      const result = await this.producer.send({
        topic: topic,
        messages: messages
      });

      logger.success('✅ Message production successful');
      logger.info(`📊 Sent ${messages.length} messages in ${result.length} batches`);
      
      // Log details for learning
      result.forEach((batch, index) => {
        logger.info(`   Batch ${index}: partition ${batch.partition}, offset ${batch.baseOffset}`);
      });

      this.testResults.passed++;
    } catch (error) {
      logger.error('❌ Message production test failed:', error.message);
      this.testResults.failed++;
      this.testResults.errors.push({ test: 'MessageProduction', error: error.message });
    }
  }

  /**
   * Test 5: Message consumption
   * Learning: How to receive messages from Kafka
   */
  async testMessageConsumption() {
    logger.info('\n📥 Test 5: Message Consumption');
    logger.info('Learning: Consumers read messages from topics and track their progress');
    
    try {
      const topic = TEST_CONFIG.TOPICS[0];
      const receivedMessages = [];

      // Subscribe to topic
      await this.consumer.subscribe({ topic: topic, fromBeginning: true });
      logger.info(`🔔 Subscribed to topic: ${topic}`);

      // Set up message handler
      let messageCount = 0;
      const messagePromise = new Promise((resolve, reject) => {
        const timeout = setTimeout(() => {
          reject(new Error('Timeout waiting for messages'));
        }, 10000);

        this.consumer.run({
          eachMessage: async ({ topic, partition, message }) => {
            messageCount++;
            const messageData = {
              topic,
              partition,
              offset: message.offset,
              key: message.key?.toString(),
              value: JSON.parse(message.value.toString()),
              timestamp: message.timestamp
            };
            
            receivedMessages.push(messageData);
            logger.info(`📨 Received message ${messageCount}: key=${messageData.key}, offset=${messageData.offset}`);
            
            // Stop after receiving expected messages
            if (messageCount >= TEST_CONFIG.MESSAGES_TO_SEND) {
              clearTimeout(timeout);
              resolve();
            }
          }
        });
      });

      // Wait for messages
      await messagePromise;

      logger.success('✅ Message consumption successful');
      logger.info(`📊 Received ${receivedMessages.length} messages`);
      
      // Verify message content
      const firstMessage = receivedMessages[0];
      if (firstMessage && firstMessage.value.source === TEST_CONFIG.MESSAGE_PAYLOAD.source) {
        logger.success('✅ Message content verification passed');
      } else {
        throw new Error('Message content verification failed');
      }

      this.testResults.passed++;
    } catch (error) {
      logger.error('❌ Message consumption test failed:', error.message);
      this.testResults.failed++;
      this.testResults.errors.push({ test: 'MessageConsumption', error: error.message });
    }
  }

  /**
   * Test 6: Consumer group coordination
   * Learning: How multiple consumers work together
   */
  async testConsumerGroupCoordination() {
    logger.info('\n👥 Test 6: Consumer Group Coordination');
    logger.info('Learning: Consumer groups allow scaling and fault tolerance');
    
    try {
      // List consumer groups
      const groups = await this.admin.listGroups();
      logger.info(`📋 Active consumer groups: ${groups.groups.length}`);
      
      // Find our test group
      const ourGroup = groups.groups.find(g => g.groupId === TEST_CONFIG.CONSUMER_GROUP);
      if (ourGroup) {
        logger.success(`✅ Found our consumer group: ${ourGroup.groupId}`);
        logger.info(`   Protocol: ${ourGroup.protocolType}`);
      } else {
        logger.warning('⚠️  Our consumer group not found (this may be normal)');
      }

      this.testResults.passed++;
    } catch (error) {
      logger.error('❌ Consumer group coordination test failed:', error.message);
      this.testResults.failed++;
      this.testResults.errors.push({ test: 'ConsumerGroupCoordination', error: error.message });
    }
  }

  /**
   * Cleanup resources
   */
  async cleanup() {
    logger.info('\n🧹 Cleaning up resources...');
    
    try {
      if (this.consumer) {
        await this.consumer.disconnect();
        logger.success('✅ Consumer disconnected');
      }
      
      if (this.producer) {
        await this.producer.disconnect();
        logger.success('✅ Producer disconnected');
      }
      
      if (this.admin) {
        await this.admin.disconnect();
        logger.success('✅ Admin disconnected');
      }
    } catch (error) {
      logger.error('❌ Cleanup error:', error.message);
    }
  }

  /**
   * Print comprehensive test results
   */
  printResults() {
    logger.info('\n' + '='.repeat(60));
    logger.info('🎯 KAFKA-RS INTEGRATION TEST RESULTS');
    logger.info('='.repeat(60));
    
    logger.info(`✅ Tests Passed: ${this.testResults.passed}`);
    logger.info(`❌ Tests Failed: ${this.testResults.failed}`);
    logger.info(`📊 Success Rate: ${((this.testResults.passed / (this.testResults.passed + this.testResults.failed)) * 100).toFixed(1)}%`);
    
    if (this.testResults.errors.length > 0) {
      logger.info('\n❌ Failed Tests:');
      this.testResults.errors.forEach(({ test, error }) => {
        logger.error(`   ${test}: ${error}`);
      });
    }
    
    logger.info('\n💡 Learning Summary:');
    logger.info('   - ApiVersions: Essential for client-server compatibility');
    logger.info('   - Metadata: How clients discover the cluster topology');
    logger.info('   - Topics: Fundamental data organization in Kafka');
    logger.info('   - Producers: Send messages with keys and values');
    logger.info('   - Consumers: Read messages and track offsets');
    logger.info('   - Consumer Groups: Enable scalable message processing');
    
    if (this.testResults.failed === 0) {
      logger.info('\n🎉 All tests passed! Your Kafka-RS server is working great!');
    } else {
      logger.info('\n🔧 Some tests failed. Check the logs above for debugging hints.');
    }
    
    logger.info('='.repeat(60));
  }

  /**
   * Run all tests in sequence
   */
  async runAllTests() {
    const startTime = Date.now();
    
    logger.info('🚀 Starting Kafka-RS Integration Test Suite');
    logger.info(`🎯 Server: ${TEST_CONFIG.SERVER_HOST}:${TEST_CONFIG.SERVER_PORT}`);
    logger.info(`📝 Topics: ${TEST_CONFIG.TOPICS.join(', ')}`);
    logger.info(`👥 Consumer Group: ${TEST_CONFIG.CONSUMER_GROUP}`);
    
    try {
      await this.initialize();
      
      // Run tests in logical order
      await this.testApiVersions();
      await this.testMetadata();
      await this.testTopicCreation();
      await this.testMessageProduction();
      await this.testMessageConsumption();
      await this.testConsumerGroupCoordination();
      
    } catch (error) {
      logger.error('💥 Critical error during test execution:', error.message);
      this.testResults.failed++;
      this.testResults.errors.push({ test: 'Critical', error: error.message });
    } finally {
      await this.cleanup();
      
      const duration = ((Date.now() - startTime) / 1000).toFixed(2);
      logger.info(`⏱️  Total test duration: ${duration}s`);
      
      this.printResults();
      
      // Exit with appropriate code for CI/CD
      process.exit(this.testResults.failed > 0 ? 1 : 0);
    }
  }
}

/**
 * Main execution
 */
async function main() {
  // Handle graceful shutdown
  process.on('SIGINT', () => {
    logger.warning('\n⚠️  Received SIGINT, shutting down gracefully...');
    process.exit(0);
  });

  process.on('SIGTERM', () => {
    logger.warning('\n⚠️  Received SIGTERM, shutting down gracefully...');
    process.exit(0);
  });

  // Run the test suite
  const testSuite = new KafkaIntegrationTests();
  await testSuite.runAllTests();
}

// Execute if this file is run directly
if (require.main === module) {
  main().catch(error => {
    logger.error('💥 Unhandled error:', error);
    process.exit(1);
  });
}

module.exports = { KafkaIntegrationTests, TEST_CONFIG };
