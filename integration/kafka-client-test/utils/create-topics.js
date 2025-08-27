/**
 * Topic Creation Utilities
 * 
 * This module provides utilities for creating and managing topics
 * in our Kafka-RS server. It's designed for learning how topic
 * management works in Kafka.
 */

const { logger } = require('./logger');

/**
 * Create test topics for integration testing
 * 
 * @param {import('kafkajs').Admin} admin - Kafka admin client
 * @param {string[]} topicNames - Array of topic names to create
 * @param {Object} options - Topic creation options
 */
async function createTestTopics(admin, topicNames, options = {}) {
  const defaultOptions = {
    numPartitions: 1,
    replicationFactor: 1,
    configEntries: [
      {
        name: 'cleanup.policy',
        value: 'delete'
      },
      {
        name: 'retention.ms',
        value: '86400000' // 24 hours
      }
    ],
    ...options
  };

  logger.info(`🏗️  Creating ${topicNames.length} test topics...`);

  try {
    // Check which topics already exist
    const existingTopics = await admin.listTopics();
    const topicsToCreate = topicNames.filter(name => !existingTopics.includes(name));

    if (topicsToCreate.length === 0) {
      logger.info('ℹ️  All topics already exist, skipping creation');
      return;
    }

    // Prepare topic configurations
    const topicConfigurations = topicsToCreate.map(topic => ({
      topic,
      numPartitions: defaultOptions.numPartitions,
      replicationFactor: defaultOptions.replicationFactor,
      configEntries: defaultOptions.configEntries
    }));

    // Create topics
    logger.info(`📝 Creating topics: ${topicsToCreate.join(', ')}`);
    await admin.createTopics({
      topics: topicConfigurations,
      waitForLeaders: true,
      timeout: 10000
    });

    logger.success(`✅ Successfully created ${topicsToCreate.length} topics`);
    
    // Log topic details for learning
    topicsToCreate.forEach(topic => {
      logger.info(`   📁 ${topic}:`);
      logger.info(`      Partitions: ${defaultOptions.numPartitions}`);
      logger.info(`      Replication Factor: ${defaultOptions.replicationFactor}`);
      logger.info(`      Cleanup Policy: ${defaultOptions.configEntries[0]?.value || 'default'}`);
    });

  } catch (error) {
    if (error.message.includes('Topic already exists')) {
      logger.warning('⚠️  Some topics already exist, continuing...');
    } else {
      logger.error('❌ Failed to create topics:', error.message);
      throw error;
    }
  }
}

/**
 * Delete test topics (cleanup utility)
 * 
 * @param {import('kafkajs').Admin} admin - Kafka admin client
 * @param {string[]} topicNames - Array of topic names to delete
 */
async function deleteTestTopics(admin, topicNames) {
  logger.info(`🗑️  Deleting ${topicNames.length} test topics...`);

  try {
    // Check which topics exist
    const existingTopics = await admin.listTopics();
    const topicsToDelete = topicNames.filter(name => existingTopics.includes(name));

    if (topicsToDelete.length === 0) {
      logger.info('ℹ️  No topics to delete');
      return;
    }

    // Delete topics
    await admin.deleteTopics({
      topics: topicsToDelete,
      timeout: 10000
    });

    logger.success(`✅ Successfully deleted ${topicsToDelete.length} topics`);
  } catch (error) {
    logger.error('❌ Failed to delete topics:', error.message);
    throw error;
  }
}

/**
 * List all topics with detailed information
 * 
 * @param {import('kafkajs').Admin} admin - Kafka admin client
 */
async function listTopicsDetailed(admin) {
  logger.info('📋 Listing all topics...');

  try {
    const topics = await admin.listTopics();
    const metadata = await admin.fetchTopicMetadata({ topics });

    logger.info(`📊 Found ${topics.length} topics:`);
    
    metadata.topics.forEach(topic => {
      logger.info(`   📁 ${topic.name}:`);
      logger.info(`      Partitions: ${topic.partitions.length}`);
      topic.partitions.forEach(partition => {
        logger.info(`         Partition ${partition.partitionId}: leader=${partition.leader}, replicas=[${partition.replicas.join(', ')}]`);
      });
    });

    return metadata;
  } catch (error) {
    logger.error('❌ Failed to list topics:', error.message);
    throw error;
  }
}

module.exports = {
  createTestTopics,
  deleteTestTopics,
  listTopicsDetailed
};
