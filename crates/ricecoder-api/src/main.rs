//! RiceCoder API Server
//!
//! RESTful API server for RiceCoder providing session management,
//! MCP tool execution, and enterprise features.

use ricecoder_api::{ApiServer, AppState};
use ricecoder_di::DIContainer;
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting RiceCoder API Server...");

    // Initialize dependency injection container
    let container = DIContainer::new();

    // Create application state
    let state = AppState::new(container).await?;

    // Create and start server
    let server = ApiServer::new(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!("API server listening on {}", addr);

    server.run(addr).await?;

    Ok(())
}
