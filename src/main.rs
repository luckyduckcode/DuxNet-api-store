mod core;
mod api;
mod wallet;
mod frontend;

use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting DuxNet API Store");
    
    // Create and start the DuxNet node
    let node = core::DuxNetNode::new().await?;
    
    // Start the web API server
    let api_handle = tokio::spawn(async move {
        if let Err(e) = api::start_api_server(8081, node).await {
            error!("API server error: {}", e);
        }
    });
    
    info!("DuxNet API Store started successfully!");
    info!("Web API available at: http://localhost:8081");
    
    // Wait for the API server
    api_handle.await?;
    
    Ok(())
} 