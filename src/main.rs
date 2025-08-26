use clap::Parser;
use log::info;
use std::sync::Arc;

mod domain;
mod infrastructure;
mod application;

use infrastructure::server::KafkaServer;
use infrastructure::persistence::{InMemoryTopicRepository, InMemoryOffsetRepository};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Host to bind the server to
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// Port to bind the server to
    #[arg(short, long, default_value_t = 9092)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();

    let args = Args::parse();

    info!("Starting Kafka-RS server on {}:{}", args.host, args.port);
    info!("This is an educational implementation of Kafka in Rust");
    info!("Compatible with KafkaJS and other Kafka clients");

    // Create shared repositories
    let topic_repo = Arc::new(InMemoryTopicRepository::new());
    let offset_repo = Arc::new(InMemoryOffsetRepository::new());

    // Create and start the server
    let server = KafkaServer::new(
        args.host,
        args.port,
        topic_repo,
        offset_repo,
    );

    server.start().await?;

    Ok(())
}
