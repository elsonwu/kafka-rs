//! Integration tests for the Kafka-RS server
//! 
//! These tests verify that the core functionality works correctly
//! in an integrated environment.

use kafka_rs::domain::entities::Message;
use kafka_rs::domain::services::MessageService;
use kafka_rs::domain::value_objects::{TopicName};
use kafka_rs::infrastructure::persistence::InMemoryTopicRepository;
use std::sync::Arc;

/// Test that we can create a message service and it compiles correctly
#[tokio::test]
async fn test_service_creation() {
    // Arrange
    let topic_repo = Arc::new(InMemoryTopicRepository::new());
    let message_service = MessageService::new(topic_repo.clone());
    
    let topic_name = TopicName::new("test-topic".to_string()).unwrap();
    let message = Message::new(None, b"Hello, Kafka-RS!".to_vec());
    
    // Act
    let result = message_service
        .send_message(topic_name.clone(), message)
        .await;
    
    // Assert
    assert!(result.is_ok());
    let offset = result.unwrap();
    assert_eq!(offset.value(), 0); // First message should have offset 0
}

/// Test basic message retrieval
#[tokio::test] 
async fn test_message_retrieval() {
    // Arrange
    let topic_repo = Arc::new(InMemoryTopicRepository::new());
    let message_service = MessageService::new(topic_repo.clone());
    
    let topic_name = TopicName::new("retrieve-test".to_string()).unwrap();
    
    // Send a message
    let message = Message::new(None, b"Test message".to_vec());
    let _offset = message_service
        .send_message(topic_name.clone(), message)
        .await
        .unwrap();
    
    // Act - Try to retrieve messages
    let result = message_service
        .get_messages(&topic_name, kafka_rs::domain::value_objects::Offset::new(0), 10)
        .await;
    
    // Assert
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].value, b"Test message");
}

/// Test topic creation functionality
#[tokio::test]
async fn test_topic_creation() {
    // Arrange  
    let topic_repo = Arc::new(InMemoryTopicRepository::new());
    let message_service = MessageService::new(topic_repo.clone());
    
    let topic_name = TopicName::new("create-test".to_string()).unwrap();
    
    // Act
    let result = message_service.create_topic(topic_name.clone()).await;
    
    // Assert
    assert!(result.is_ok());
}
