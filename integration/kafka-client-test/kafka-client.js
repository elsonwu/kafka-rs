/**
 * Kafka Client Configuration
 * 
 * Centralized configuration for all Kafka clients used in testing.
 * This demonstrates how to properly configure KafkaJS for connecting
 * to our Kafka-RS server.
 */

const { Kafka, logLevel } = require('kafkajs');

// Create the main Kafka instance with appropriate configuration
const kafka = new Kafka({
  clientId: 'kafka-rs-integration-test',
  brokers: [`${process.env.KAFKA_HOST || 'localhost'}:${process.env.KAFKA_PORT || 9092}`],
  
  // Connection settings optimized for local testing
  connectionTimeout: 10000,
  requestTimeout: 10000,
  
  // Retry configuration for robust testing
  retry: {
    initialRetryTime: 100,
    retries: 5
  },
  
  // Logging configuration for learning
  logLevel: process.env.NODE_ENV === 'development' ? logLevel.DEBUG : logLevel.INFO,
  logCreator: () => ({ namespace, level, label, log }) => {
    // Custom log formatting for better readability during learning
    const { timestamp, message, ...extra } = log;
    const formattedMessage = `[KafkaJS/${namespace}] ${message}`;
    
    if (Object.keys(extra).length > 0) {
      console.log(`${formattedMessage}`, extra);
    } else {
      console.log(formattedMessage);
    }
  }
});

module.exports = { kafka };
