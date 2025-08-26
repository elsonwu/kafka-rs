//! # Kafka-RS
//!
//! An educational implementation of Apache Kafka in Rust using Domain-Driven Design.
//!
//! This library provides a compatible subset of the Kafka protocol for learning purposes,
//! including support for producers, consumers, and basic topic management.
//!
//! ## Architecture
//!
//! The codebase follows Domain-Driven Design principles with clear separation between:
//! - **Domain Layer**: Core business logic and entities
//! - **Application Layer**: Use cases and orchestration
//! - **Infrastructure Layer**: Technical concerns like networking and persistence
//!
//! ## Usage
//!
//! ```rust
//! use kafka_rs::domain::services::MessageService;
//! use kafka_rs::domain::value_objects::TopicName;
//! use kafka_rs::domain::entities::Message;
//! use kafka_rs::infrastructure::persistence::InMemoryTopicRepository;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Set up repositories
//!     let topic_repo = Arc::new(InMemoryTopicRepository::new());
//!     
//!     // Create message service
//!     let message_service = MessageService::new(topic_repo);
//!     
//!     // Send a message
//!     let topic_name = TopicName::new("my-topic".to_string())?;
//!     let message = Message::new(None, b"Hello, Kafka!".to_vec());
//!     let offset = message_service
//!         .send_message(topic_name.clone(), message)
//!         .await?;
//!     
//!     println!("Message sent with offset: {}", offset.value());
//!     Ok(())
//! }
//! ```

pub mod domain;
pub mod application;
pub mod infrastructure;

// Re-export commonly used types for convenience
pub use domain::entities::{Message, Topic, Partition};
pub use domain::value_objects::{TopicName, Offset, MessageId};
pub use domain::services::{MessageService, OffsetManagementService};
pub use infrastructure::persistence::{InMemoryTopicRepository, InMemoryOffsetRepository};
pub use infrastructure::server::KafkaServer;
