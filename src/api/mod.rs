//! HTTP REST API: exposes core, wallet, and network functionality via Axum endpoints for external interaction.

use std::collections::HashMap;

pub mod dux_coin;
pub mod state;
pub mod handlers;
pub mod routes;

use state::ApiState;
use routes::create_router;
use tracing::info;

pub async fn start_api_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting API server on port {}", port);
    let node = std::sync::Arc::new(crate::core::DuxNetNode::new(8080).await?);
    // TODO: Load API keys from secure storage or config
    let api_keys = std::sync::Arc::new(HashMap::new());
    let state = ApiState { node, api_keys };
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("API server listening on port {}", port);
    axum::serve(listener, app).await?;
    Ok(())
} 