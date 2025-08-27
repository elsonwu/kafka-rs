const { Kafka, logLevel } = require('kafkajs');

/**
 * Debug Metadata Utility
 * 
 * This utility tests metadata requests against the Kafka server.
 * It's useful for debugging server compatibility issues.
 */

async function testMetadata() {
  const kafka = new Kafka({
    clientId: 'debug-metadata',
    brokers: ['localhost:9092'],
    logLevel: logLevel.DEBUG,
  });

  const admin = kafka.admin();
  
  try {
    console.log('🔍 Testing Metadata Request...');
    await admin.connect();
    
    // Fetch cluster metadata
    const metadata = await admin.fetchTopicMetadata();
    console.log('✅ Metadata received:');
    console.log(JSON.stringify(metadata, null, 2));
    
    // List existing topics
    const topics = await admin.listTopics();
    console.log('✅ Available topics:', topics);
    
  } catch (error) {
    console.error('❌ Metadata test failed:', error.message);
    console.error('Full error:', error);
    process.exit(1);
  } finally {
    await admin.disconnect();
    console.log('✅ Admin client disconnected');
  }
}

// Run if called directly
if (require.main === module) {
  testMetadata().catch(console.error);
}

module.exports = { testMetadata };
