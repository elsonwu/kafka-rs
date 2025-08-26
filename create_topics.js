const { Kafka } = require('kafkajs');

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
          topic: 'test-topic-1',
          numPartitions: 1,
          replicationFactor: 1,
        },
        {
          topic: 'test-topic-2', 
          numPartitions: 1,
          replicationFactor: 1,
        }
      ],
    });
    
    console.log('✅ Topics created successfully');
    
  } catch (error) {
    console.error('❌ Failed to create topics:', error.message);
  } finally {
    await admin.disconnect();
  }
}

createTopics();
