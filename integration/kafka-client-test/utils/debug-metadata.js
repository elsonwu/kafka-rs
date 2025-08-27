/**
 * Metadata Debug Utilities
 * 
 * This module provides utilities for inspecting and debugging
 * Kafka metadata. It's designed to help learners understand
 * how Kafka organizes brokers, topics, and partitions.
 */

const { logger } = require('./logger');

/**
 * Debug and display comprehensive metadata information
 * 
 * @param {import('kafkajs').Kafka} kafka - Kafka instance
 */
async function debugMetadata(kafka) {
  logger.info('🔍 Debugging metadata information...');

  const admin = kafka.admin();
  
  try {
    await admin.connect();

    // Get cluster metadata
    const metadata = await admin.fetchTopicMetadata();
    
    logger.info('\n📊 CLUSTER METADATA ANALYSIS');
    logger.info('=' .repeat(50));
    
    // Broker information
    logger.info(`🖥️  Brokers (${metadata.brokers.length}):`);
    metadata.brokers.forEach(broker => {
      logger.info(`   Broker ${broker.nodeId}:`);
      logger.info(`     Host: ${broker.host}:${broker.port}`);
      logger.info(`     Rack: ${broker.rack || 'none'}`);
    });

    // Topics information
    logger.info(`\n📁 Topics (${metadata.topics.length}):`);
    if (metadata.topics.length === 0) {
      logger.warning('   ⚠️  No topics found - create some topics to see more interesting data!');
    } else {
      metadata.topics.forEach(topic => {
        logger.info(`   📂 ${topic.name}:`);
        logger.info(`      Error Code: ${topic.errorCode} (0 = success)`);
        logger.info(`      Partitions: ${topic.partitions.length}`);
        
        topic.partitions.forEach(partition => {
          logger.info(`         Partition ${partition.partitionId}:`);
          logger.info(`           Leader: Broker ${partition.leader}`);
          logger.info(`           Replicas: [${partition.replicas.join(', ')}]`);
          logger.info(`           ISR: [${partition.isr.join(', ')}]`);
          logger.info(`           Error Code: ${partition.errorCode}`);
        });
      });
    }

    // Learning insights
    logger.info('\n💡 LEARNING INSIGHTS:');
    logger.info('   - Brokers: Physical servers that store and serve data');
    logger.info('   - Topics: Logical channels for organizing messages');
    logger.info('   - Partitions: Allow parallel processing and scaling');
    logger.info('   - Leader: The broker responsible for reads/writes to a partition');
    logger.info('   - Replicas: Brokers that have copies of the partition data');
    logger.info('   - ISR: In-Sync Replicas (currently up-to-date replicas)');
    
    return metadata;

  } catch (error) {
    logger.error('❌ Failed to fetch metadata:', error.message);
    
    // Common troubleshooting hints for learners
    logger.info('\n🔧 TROUBLESHOOTING HINTS:');
    logger.info('   - Is the Kafka-RS server running?');
    logger.info('   - Check if the server is listening on the correct port');
    logger.info('   - Verify ApiVersions support is implemented');
    logger.info('   - Look at server logs for error messages');
    
    throw error;
  } finally {
    await admin.disconnect();
  }
}

/**
 * Compare metadata between calls to detect changes
 * 
 * @param {import('kafkajs').Kafka} kafka - Kafka instance
 * @param {number} interval - Time between checks in milliseconds
 * @param {number} duration - How long to monitor in milliseconds
 */
async function monitorMetadataChanges(kafka, interval = 5000, duration = 30000) {
  logger.info(`🔄 Monitoring metadata changes every ${interval/1000}s for ${duration/1000}s...`);

  const admin = kafka.admin();
  let previousMetadata = null;
  let changeCount = 0;

  try {
    await admin.connect();

    const endTime = Date.now() + duration;
    
    while (Date.now() < endTime) {
      const currentMetadata = await admin.fetchTopicMetadata();
      
      if (previousMetadata) {
        const changes = detectMetadataChanges(previousMetadata, currentMetadata);
        if (changes.length > 0) {
          changeCount++;
          logger.info(`\n🔄 Change ${changeCount} detected:`);
          changes.forEach(change => logger.info(`   ${change}`));
        }
      } else {
        logger.info('📸 Initial metadata snapshot captured');
      }
      
      previousMetadata = currentMetadata;
      await sleep(interval);
    }

    logger.info(`\n📊 Monitoring complete: ${changeCount} changes detected`);

  } catch (error) {
    logger.error('❌ Metadata monitoring failed:', error.message);
    throw error;
  } finally {
    await admin.disconnect();
  }
}

/**
 * Detect changes between two metadata snapshots
 * 
 * @param {Object} prev - Previous metadata
 * @param {Object} curr - Current metadata
 * @returns {string[]} - Array of change descriptions
 */
function detectMetadataChanges(prev, curr) {
  const changes = [];

  // Check broker changes
  if (prev.brokers.length !== curr.brokers.length) {
    changes.push(`Broker count changed: ${prev.brokers.length} → ${curr.brokers.length}`);
  }

  // Check topic changes
  if (prev.topics.length !== curr.topics.length) {
    changes.push(`Topic count changed: ${prev.topics.length} → ${curr.topics.length}`);
  }

  const prevTopicNames = new Set(prev.topics.map(t => t.name));
  const currTopicNames = new Set(curr.topics.map(t => t.name));

  // New topics
  currTopicNames.forEach(name => {
    if (!prevTopicNames.has(name)) {
      changes.push(`New topic created: ${name}`);
    }
  });

  // Deleted topics
  prevTopicNames.forEach(name => {
    if (!currTopicNames.has(name)) {
      changes.push(`Topic deleted: ${name}`);
    }
  });

  return changes;
}

/**
 * Simple sleep utility
 * 
 * @param {number} ms - Milliseconds to sleep
 */
function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

module.exports = {
  debugMetadata,
  monitorMetadataChanges,
  detectMetadataChanges
};
