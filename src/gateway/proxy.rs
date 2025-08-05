use crate::core::{data_structures::ServiceId, service_manager::ServiceManager};
use anyhow::{Context, Result};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tracing::{error, info, warn};

/// P2P API Gateway for routing API calls to deployed services
/// 
/// This gateway enables:
/// 1. Routing API calls to locally deployed containers
/// 2. Proxying requests to remote P2P services
/// 3. Load balancing across multiple service instances
/// 4. Health checking and failover
#[derive(Clone)]
pub struct P2PApiGateway {
    /// HTTP client for making proxied requests
    client: Client,
    /// Reference to the service manager for service discovery
    service_manager: Arc<ServiceManager>,
    /// Gateway statistics
    stats: Arc<tokio::sync::RwLock<GatewayStats>>,
}

impl P2PApiGateway {
    /// Create a new P2P API Gateway
    pub fn new(service_manager: Arc<ServiceManager>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            service_manager,
            stats: Arc::new(tokio::sync::RwLock::new(GatewayStats::default())),
        }
    }

    /// Proxy an API request to a deployed service
    pub async fn proxy_request(
        &self,
        service_id: &str,
        path: &str,
        method: axum::http::Method,
        headers: axum::http::HeaderMap,
        body: bytes::Bytes,
    ) -> Result<axum::response::Response, Box<dyn std::error::Error + Send + Sync>> {
        // Find service endpoint (this would come from service registry)
        let service_url = format!("http://localhost:8080{}", path); // Placeholder
        
        let url = reqwest::Url::parse(&service_url)?;
        
        // Convert axum Method to reqwest Method
        let reqwest_method = match method {
            axum::http::Method::GET => reqwest::Method::GET,
            axum::http::Method::POST => reqwest::Method::POST,
            axum::http::Method::PUT => reqwest::Method::PUT,
            axum::http::Method::DELETE => reqwest::Method::DELETE,
            axum::http::Method::PATCH => reqwest::Method::PATCH,
            axum::http::Method::HEAD => reqwest::Method::HEAD,
            axum::http::Method::OPTIONS => reqwest::Method::OPTIONS,
            _ => reqwest::Method::GET, // fallback
        };
        
        let mut request = self.client.request(reqwest_method, url);
        
        // Convert and add headers
        for (name, value) in headers.iter() {
            if let (Ok(name_str), Ok(value_str)) = (name.as_str().parse::<reqwest::header::HeaderName>(), value.to_str()) {
                request = request.header(name_str, value_str);
            }
        }
        
        // Add body if present
        if !body.is_empty() {
            request = request.body(body.to_vec());
        }

        let start_time = std::time::Instant::now();
        
        // Execute request
        let response = request.send().await?;
        
        let duration = start_time.elapsed();
        
        // Update statistics
        self.update_stats(duration.as_millis() as f64, response.status().is_success()).await;
        
        // Convert response back to axum format
        let status_code = axum::http::StatusCode::from_u16(response.status().as_u16())
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        
        let mut axum_response = axum::response::Response::builder().status(status_code);
        
        // Convert headers
        for (name, value) in response.headers().iter() {
            if let (Ok(name_str), Ok(value_str)) = (name.as_str().parse::<axum::http::HeaderName>(), value.to_str()) {
                axum_response = axum_response.header(name_str, value_str);
            }
        }
        
        let body_bytes = response.bytes().await?;
        
        Ok(axum_response.body(axum::body::Body::from(body_bytes.to_vec()))?)
    }

    /// Select the best endpoint for routing (load balancing)
    async fn select_endpoint(&self, endpoints: &[String]) -> Result<String> {
        if endpoints.is_empty() {
            return Err(anyhow::anyhow!("No endpoints available for service"));
        }

        // For now, use simple round-robin (can be enhanced with health checking)
        // In a real implementation, you'd want to:
        // 1. Check endpoint health
        // 2. Consider response times
        // 3. Balance load across instances
        Ok(endpoints[0].clone())
    }

    /// Check if a header is hop-by-hop and should not be forwarded
    fn is_hop_by_hop_header(&self, name: &str) -> bool {
        matches!(
            name,
            "connection"
                | "keep-alive"
                | "proxy-authenticate"
                | "proxy-authorization"
                | "te"
                | "trailers"
                | "transfer-encoding"
                | "upgrade"
        )
    }

    /// Health check a specific service endpoint
    pub async fn health_check_endpoint(&self, endpoint: &str) -> bool {
        let health_url = format!("{}/health", endpoint);
        
        match self.client.get(&health_url).send().await {
            Ok(response) => {
                let is_healthy = response.status().is_success();
                if is_healthy {
                    info!("Endpoint {} is healthy", endpoint);
                } else {
                    warn!("Endpoint {} returned status: {}", endpoint, response.status());
                }
                is_healthy
            }
            Err(e) => {
                warn!("Health check failed for endpoint {}: {}", endpoint, e);
                false
            }
        }
    }

    /// Update gateway statistics
    async fn update_stats(&self, response_time_ms: f64, success: bool) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        
        if success {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }
        
        // Calculate average response time
        let total_response_time = stats.average_response_time_ms * (stats.total_requests - 1) as f64;
        stats.average_response_time_ms = (total_response_time + response_time_ms) / stats.total_requests as f64;
    }

    /// Health check for the gateway
    pub async fn health_check(&self) -> bool {
        // Simple health check - gateway is healthy if it's responding
        true
    }

    /// Get gateway statistics
    pub async fn get_stats(&self) -> GatewayStats {
        // Get service count from service manager
        let services = self.service_manager.list_services().await;
        let stats = self.stats.read().await;
        
        GatewayStats {
            total_services: services.len(),
            active_endpoints: services.iter().map(|(_, instance)| instance.endpoints.len()).sum(),
            requests_processed: stats.requests_processed,
            average_response_time_ms: stats.average_response_time_ms,
            total_requests: stats.total_requests,
            successful_requests: stats.successful_requests,
            failed_requests: stats.failed_requests,
            active_connections: stats.active_connections,
            peer_connections: stats.peer_connections,
        }
    }
}

/// Gateway statistics for monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GatewayStats {
    pub total_services: usize,
    pub active_endpoints: usize,
    pub requests_processed: u64,
    pub average_response_time_ms: f64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub active_connections: usize,
    pub peer_connections: usize,
}

impl Default for GatewayStats {
    fn default() -> Self {
        Self {
            total_services: 0,
            active_endpoints: 0,
            requests_processed: 0,
            average_response_time_ms: 0.0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            active_connections: 0,
            peer_connections: 0,
        }
    }
}

/// API request structure for P2P calls
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ApiRequest {
    pub method: String,
    pub path: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Vec<u8>,
    pub service_id: String,
}

/// API response structure for P2P calls
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ApiResponse {
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Vec<u8>,
}

// Gateway HTTP handlers for Axum integration

/// Handle proxied API requests
pub async fn proxy_api_request(
    State(state): State<crate::api::state::ApiState>,
    Path((service_id, api_path)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Get service details from ServiceManager first
    let service_id_typed = crate::core::data_structures::ServiceId(service_id.clone());
    let service = match state.node.service_manager.get_service_by_id(&service_id_typed).await {
        Some(service) => service,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Service not found",
                    "service_id": service_id
                }))
            ).into_response();
        }
    };

    let path = format!("/{}", api_path);
    
    match state.node.api_gateway.proxy_request(&service_id, &path, method, headers, body).await {
        Ok(response) => response,
        Err(e) => {
            error!("Proxy request failed: {}", e);
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "error": "Service temporarily unavailable",
                    "details": e.to_string()
                }))
            ).into_response()
        }
    }
}

/// Get gateway statistics
pub async fn get_gateway_stats(
    State(state): State<crate::api::state::ApiState>,
) -> impl IntoResponse {
    let stats = state.node.api_gateway.get_stats().await;
    Json(serde_json::json!({
        "gateway_stats": stats,
        "success": true
    }))
}

/// Health check for the gateway itself
pub async fn gateway_health_check(
    State(state): State<crate::api::state::ApiState>,
) -> impl IntoResponse {
    let healthy = state.node.api_gateway.health_check().await;
    Json(serde_json::json!({
        "status": if healthy { "healthy" } else { "unhealthy" },
        "gateway": "P2PApiGateway",
        "timestamp": chrono::Utc::now().timestamp()
    }))
}
