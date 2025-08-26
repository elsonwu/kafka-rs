const { Kafka } = require('kafkajs');

const client = new Kafka({
  brokers: ['localhost:9092'],
  logLevel: 'debug'
});

async function debugConsumer() {
  const consumer = client.consumer({ 
    groupId: 'debug-group',
    maxWaitTimeInMs: 100,
    minBytes: 1,
    maxBytes: 1024 * 1024
  });

  console.log('📥 Connecting consumer...');
  await consumer.connect();

  console.log('📝 Subscribing to topic...');
  await consumer.subscribe({ topic: 'debug-topic' });

  console.log('⏳ Running consumer for 10 seconds...');
  
  let messageCount = 0;
  await consumer.run({
    eachMessage: async ({ topic, partition, message }) => {
      messageCount++;
      console.log(`📨 Message ${messageCount}: ${message.value?.toString()}`);
    },
  });

  // Let it run for 10 seconds then disconnect
  setTimeout(async () => {
    console.log('🔌 Disconnecting consumer...');
    await consumer.disconnect();
    process.exit(0);
  }, 10000);
}

debugConsumer().catch(console.error);
