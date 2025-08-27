const { Kafka } = require('kafkajs');

/**
 * Create Topics Utility
 * 
 * This utility creates test topics for the Kafka integration tests.
 * It's useful for debugging and setting up the test environment.
 */

async function createTopics() {
  const kafka = new Kafka({
    clientId: 'topic-creator',
    brokers: ['localhost:9092'],
  });

  const admin = kafka.admin();
  
  try {
    console.log('📝 Creating topics...');
    await admin.connect();
    
    await admin.createTopics({
      topics: [
        {
          topic: 'integration-test-topic',
          numPartitions: 1,
          replicationFactor: 1,
        },
        {
          topic: 'test-topic-1',
          numPartitions: 1,
          replicationFactor: 1,
        },
        {
          topic: 'test-topic-2',
          numPartitions: 2,
          replicationFactor: 1,
        }
      ]
    });
    
    console.log('✅ Topics created successfully');
    
  } catch (error) {
    console.error('❌ Failed to create topics:', error.message);
    process.exit(1);
  } finally {
    await admin.disconnect();
    console.log('✅ Admin client disconnected');
  }
}

// Run if called directly
if (require.main === module) {
  createTopics().catch(console.error);
}

module.exports = { createTopics };
