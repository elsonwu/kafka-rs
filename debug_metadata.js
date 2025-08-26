const { Kafka, logLevel } = require('kafkajs');

async function testMetadata() {
  const kafka = new Kafka({
    clientId: 'debug-test',
    brokers: ['localhost:9092'],
    logLevel: logLevel.DEBUG,
  });

  const admin = kafka.admin();
  
  try {
    console.log('🔍 Testing Metadata Request...');
    await admin.connect();
    
    const metadata = await admin.fetchTopicMetadata();
    console.log('✅ Metadata received:', JSON.stringify(metadata, null, 2));
    
  } catch (error) {
    console.error('❌ Metadata test failed:', error.message);
    console.error('Raw error:', error);
  } finally {
    await admin.disconnect();
  }
}

testMetadata();
