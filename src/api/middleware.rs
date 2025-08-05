use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::time::Duration;
use tokio::time::timeout;
use tracing::warn;

/// Timeout middleware to prevent requests from hanging
pub async fn timeout_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let uri = request.uri().clone();
    
    // Set different timeouts based on endpoint
    let timeout_duration = if uri.path().starts_with("/api/dux/") {
        Duration::from_secs(45) // DuxCoin operations may take longer
    } else if uri.path().starts_with("/api/messaging/") {
        Duration::from_secs(10) // Messaging should be fast
    } else if uri.path().starts_with("/api/wallet/balances") {
        Duration::from_secs(30) // Balance checks may need external calls
    } else {
        Duration::from_secs(15) // Default timeout for other endpoints
    };
    
    match timeout(timeout_duration, next.run(request)).await {
        Ok(response) => Ok(response),
        Err(_) => {
            warn!("Request timeout for {}", uri.path());
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    // Extract API key from headers
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("anonymous");
    
    // For now, just log the request - real rate limiting would check a store
    tracing::debug!("API request from key: {}", &api_key[..8.min(api_key.len())]);
    
    Ok(next.run(request).await)
}
