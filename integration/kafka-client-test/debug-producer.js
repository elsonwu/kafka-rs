const { Kafka } = require('kafkajs');

const client = new Kafka({
  brokers: ['localhost:9092'],
  logLevel: 'info'
});

async function sendMessages() {
  const producer = client.producer();

  console.log('🚀 Connecting producer...');
  await producer.connect();

  console.log('📤 Sending messages to debug-topic...');
  for (let i = 1; i <= 3; i++) {
    await producer.send({
      topic: 'debug-topic',
      messages: [
        {
          key: `key-${i}`,
          value: `Debug message ${i} - Hello from producer!`,
        },
      ],
    });
    console.log(`✅ Sent message ${i}`);
  }

  console.log('🔌 Disconnecting producer...');
  await producer.disconnect();
  console.log('✨ Producer finished successfully');
}

sendMessages().catch(console.error);
