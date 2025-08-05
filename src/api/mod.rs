//! HTTP REST API: exposes core, wallet, and network functionality via Axum endpoints for external interaction.

use std::collections::HashMap;

pub mod dux_coin;
pub mod state;
pub mod handlers;
pub mod routes;
pub mod marketplace;
pub mod middleware;

use state::ApiState;
use routes::create_router;
use tracing::info;

pub async fn start_api_server(port: u16, node: crate::core::DuxNetNode) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting API server on port {}", port);
    
    // Initialize API keys (in production, load from secure storage)
    let mut api_keys = HashMap::new();
    api_keys.insert("demo-api-key-123".to_string(), "did:duxnet:demo-user".to_string());
    api_keys.insert("admin-api-key-456".to_string(), "did:duxnet:admin".to_string());
    api_keys.insert("service-api-key-789".to_string(), "did:duxnet:service-provider".to_string());
    
    let state = ApiState::new(std::sync::Arc::new(node), std::sync::Arc::new(api_keys));
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("API server listening on port {}", port);
    info!("🔑 Demo API Keys: demo-api-key-123, admin-api-key-456, service-api-key-789");
    info!("📊 Analytics available at: http://localhost:{}/api/analytics/usage", port);
    info!("👨‍💻 Developer portal at: http://localhost:{}/api/developer/dashboard", port);
    axum::serve(listener, app).await?;
    Ok(())
} 