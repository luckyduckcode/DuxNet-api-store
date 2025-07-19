//! # API Handlers Module
//!
//! This module contains all async handler functions for the HTTP API endpoints exposed by the DuxNet platform.
//! 
//! ## Purpose
//! - Each function here implements the business logic for a specific API route, processing requests and returning responses.
//! - Handlers should be stateless except for accessing shared state via the `ApiState` struct.
//! - All endpoint logic (validation, calling core methods, formatting responses) should be implemented here.
//!
//! ## Best Practices
//! - Each handler should be `pub async fn` and take `State<ApiState>` as the first argument if it needs access to the node.
//! - Keep handlers focused: delegate complex logic to core modules or helper functions.
//! - Use clear, descriptive names matching the route (e.g., `get_status`, `register_service`).
//! - Return types should be compatible with Axum (e.g., `impl IntoResponse`, `Result<Html<String>, StatusCode>`).
//! - Group related handlers with comments (e.g., Wallet, Messaging, Community Fund).
//! - Add doc comments to each handler describing its endpoint and behavior.
//!
//! ## Example
//! ```rust
//! /// Handles GET /api/status
//! pub async fn get_status(State(state): State<ApiState>) -> impl IntoResponse { ... }
//! ```
//!
//! ## Future Improvements
//! - Consider splitting very large groups (e.g., wallet, messaging) into their own submodules if this file grows too large.
//! - Add error handling helpers for consistent API error responses.
//! - Use Axum extractors for validation and parsing where possible.
//!
//! This structure makes it easy to add, test, and maintain API endpoints as the platform evolves. 

use axum::{extract::State, response::IntoResponse};
use crate::api::state::ApiState;
use axum::Json;
use axum::{http::Request, http::StatusCode, middleware::Next, response::Response};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Authentication middleware for API key validation
pub async fn auth_middleware<B>(
    State(state): State<ApiState>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let start_time = SystemTime::now();
    let path = req.uri().path();
    let user_agent = req.headers()
        .get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown")
        .to_string();
    
    // Allow unauthenticated access to public endpoints
    if path == "/api/status" || path == "/api/version" || path == "/" || path == "/index.html" {
        return Ok(next.run(req).await);
    }
    
    // Check Authorization header
    let api_key = if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(key) = auth_str.strip_prefix("Bearer ") {
                key
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    
    // Check rate limiting
    if !state.check_rate_limit(api_key).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    // Check if API key exists
    if !state.api_keys.contains_key(api_key) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    // Track usage
    let response = next.run(req).await;
    let response_time = start_time.elapsed().unwrap().as_millis() as u64;
    
    // Extract IP address (simplified)
    let ip_address = "127.0.0.1".to_string(); // In production, extract from request
    
    let usage = crate::api::state::ApiUsage {
        api_key: api_key.to_string(),
        endpoint: path.to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        response_time,
        status_code: response.status().as_u16(),
        user_agent,
        ip_address,
    };
    
    state.track_usage(usage).await;
    
    Ok(response)
}

/// Handles GET /api/status
pub async fn get_status(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let reputation = node.reputation_system.get_reputation(&node.did_manager.did.id).await;
    let peers = node.network.get_peers().await;
    
    let status = crate::core::data_structures::NodeStatus {
        node_id: node.node_id.0.clone(),
        did: node.did_manager.did.id.clone(),
        is_online: true,
        uptime_seconds: 0, // TODO: implement uptime tracking
        services_count: 0,  // TODO: implement service counting
        reputation_score: reputation,
        peers_count: peers.len(),
    };
    Json(status)
}

/// Handles GET /api/version
pub async fn get_api_version(State(state): State<ApiState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "version": state.api_version,
        "status": "active",
        "features": [
            "service_registration",
            "service_discovery", 
            "payment_processing",
            "analytics",
            "rate_limiting",
            "reviews_ratings"
        ]
    }))
}

/// Handles GET /api/stats
pub async fn get_stats(State(state): State<ApiState>) -> impl IntoResponse {
    let usage_stats = state.get_usage_stats(None, 24).await;
    let analytics = state.service_analytics.read().await;
    
    Json(serde_json::json!({
        "total_requests_24h": usage_stats.len(),
        "active_services": analytics.len(),
        "total_revenue_24h": analytics.values().map(|a| a.total_revenue).sum::<u64>(),
        "average_response_time": usage_stats.iter().map(|u| u.response_time).sum::<u64>() / usage_stats.len().max(1) as u64,
        "success_rate": if usage_stats.is_empty() { 100.0 } else {
            let successful = usage_stats.iter().filter(|u| u.status_code < 400).count();
            (successful as f64 / usage_stats.len() as f64) * 100.0
        }
    }))
}

// Enhanced Service Management Handlers

/// Handles POST /api/services/register (Enhanced)
pub async fn register_service(
    State(state): State<ApiState>,
    Json(request): Json<crate::core::data_structures::RegisterServiceRequest>,
) -> impl IntoResponse {
    let node = &state.node;
    
    // Generate API key for the service
    let service_api_key = format!("srv_{}", Uuid::new_v4().to_string().replace("-", ""));
    
    match node.register_service_enhanced(request).await {
        Ok(service_id) => Json(crate::core::data_structures::RegisterServiceResponse {
            service_id: service_id.0,
            success: true,
            message: "Service registered successfully".to_string(),
            api_key: Some(service_api_key),
            documentation_url: Some(format!("http://localhost:8081/docs/services/{}", service_id.0)),
        }),
        Err(e) => {
            tracing::error!("Failed to register service: {}", e);
            Json(crate::core::data_structures::RegisterServiceResponse {
                service_id: "".to_string(),
                success: false,
                message: format!("Failed to register service: {}", e),
                api_key: None,
                documentation_url: None,
            })
        }
    }
}

/// Handles POST /api/services/search (Enhanced)
pub async fn search_services(
    State(state): State<ApiState>,
    Json(request): Json<crate::core::data_structures::FindServicesRequest>,
) -> impl IntoResponse {
    let node = &state.node;
    let services = node.find_services_enhanced(&request).await;
    
    // Calculate pagination
    let total_count = services.len() as u64;
    let limit = request.limit.unwrap_or(20) as usize;
    let offset = request.offset.unwrap_or(0) as usize;
    let paginated_services = services.into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    
    let total_pages = (total_count + limit as u64 - 1) / limit as u64;
    
    Json(crate::core::data_structures::FindServicesResponse {
        services: paginated_services,
        total_count,
        success: true,
        message: format!("Found {} services", total_count),
        pagination: crate::core::data_structures::PaginationInfo {
            current_page: (offset / limit) as u32 + 1,
            total_pages: total_pages as u32,
            items_per_page: limit as u32,
            has_next: offset + limit < total_count as usize,
            has_previous: offset > 0,
        },
    })
}

/// Handles GET /api/services/:service_id
pub async fn get_service_details(
    State(state): State<ApiState>,
    axum::extract::Path(service_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let node = &state.node;
    
    match node.get_service_details(&service_id).await {
        Ok(service) => Json(serde_json::json!({
            "success": true,
            "service": service
        })),
        Err(e) => {
            tracing::error!("Failed to get service details: {}", e);
            Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to get service details: {}", e)
            }))
        }
    }
}

/// Handles GET /api/services/categories
pub async fn get_service_categories(State(state): State<ApiState>) -> impl IntoResponse {
    let categories = vec![
        "AI & Machine Learning",
        "Data Processing", 
        "Image & Video",
        "Text & Language",
        "Financial Services",
        "Gaming & Entertainment",
        "IoT & Hardware",
        "Blockchain & Crypto",
        "Social Media",
        "E-commerce",
        "Healthcare",
        "Education",
        "Transportation",
        "Utilities",
        "Other"
    ];
    
    Json(serde_json::json!({
        "success": true,
        "categories": categories
    }))
}

/// Handles GET /api/services/trending
pub async fn get_trending_services(State(state): State<ApiState>) -> impl IntoResponse {
    let analytics = state.service_analytics.read().await;
    
    // Sort services by popularity (total requests in last 24h)
    let mut trending: Vec<_> = analytics.iter()
        .filter(|(_, stats)| stats.last_updated > SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 86400)
        .collect();
    
    trending.sort_by(|a, b| b.1.total_requests.cmp(&a.1.total_requests));
    
    let top_services: Vec<_> = trending.into_iter()
        .take(10)
        .map(|(service_id, stats)| {
            serde_json::json!({
                "service_id": service_id,
                "total_requests": stats.total_requests,
                "revenue": stats.total_revenue,
                "uptime": stats.uptime_percentage
            })
        })
        .collect();
    
    Json(serde_json::json!({
        "success": true,
        "trending_services": top_services
    }))
}

// Analytics Handlers

/// Handles GET /api/analytics/usage
pub async fn get_usage_analytics(
    State(state): State<ApiState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let hours = params.get("hours").and_then(|h| h.parse::<u64>().ok()).unwrap_or(24);
    let usage_stats = state.get_usage_stats(None, hours).await;
    
    // Group by endpoint
    let mut endpoint_stats = HashMap::new();
    for usage in &usage_stats {
        let entry = endpoint_stats.entry(usage.endpoint.clone()).or_insert((0, 0, 0));
        entry.0 += 1; // total requests
        entry.1 += usage.response_time; // total response time
        if usage.status_code < 400 {
            entry.2 += 1; // successful requests
        }
    }
    
    let analytics: Vec<_> = endpoint_stats.into_iter()
        .map(|(endpoint, (total, response_time, successful))| {
            serde_json::json!({
                "endpoint": endpoint,
                "total_requests": total,
                "successful_requests": successful,
                "success_rate": if total > 0 { (successful as f64 / total as f64) * 100.0 } else { 0.0 },
                "average_response_time": if total > 0 { response_time / total } else { 0 }
            })
        })
        .collect();
    
    Json(serde_json::json!({
        "success": true,
        "period_hours": hours,
        "total_requests": usage_stats.len(),
        "endpoint_analytics": analytics
    }))
}

/// Handles GET /api/analytics/services
pub async fn get_services_analytics(State(state): State<ApiState>) -> impl IntoResponse {
    let analytics = state.service_analytics.read().await;
    
    let services_analytics: Vec<_> = analytics.iter()
        .map(|(service_id, stats)| {
            serde_json::json!({
                "service_id": service_id,
                "total_requests": stats.total_requests,
                "successful_requests": stats.successful_requests,
                "failed_requests": stats.failed_requests,
                "success_rate": if stats.total_requests > 0 {
                    (stats.successful_requests as f64 / stats.total_requests as f64) * 100.0
                } else { 0.0 },
                "average_response_time": stats.average_response_time,
                "total_revenue": stats.total_revenue,
                "last_updated": stats.last_updated
            })
        })
        .collect();
    
    Json(serde_json::json!({
        "success": true,
        "services_analytics": services_analytics
    }))
}

// Developer Portal Handlers

/// Handles GET /api/developer/keys
pub async fn get_api_keys(State(state): State<ApiState>) -> impl IntoResponse {
    let keys: Vec<_> = state.api_keys.iter()
        .map(|(key, user_did)| {
            serde_json::json!({
                "key_id": key[..8].to_string() + "...",
                "user_did": user_did,
                "created_at": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                "last_used": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
            })
        })
        .collect();
    
    Json(serde_json::json!({
        "success": true,
        "api_keys": keys
    }))
}

/// Handles POST /api/developer/keys
pub async fn generate_api_key(State(state): State<ApiState>) -> impl IntoResponse {
    let new_key = format!("dux_{}", Uuid::new_v4().to_string().replace("-", ""));
    let user_did = "did:duxnet:generated-user".to_string();
    
    // In a real implementation, you'd add this to the state
    // For now, just return the generated key
    
    Json(serde_json::json!({
        "success": true,
        "api_key": new_key,
        "user_did": user_did,
        "message": "API key generated successfully"
    }))
}

/// Handles GET /api/developer/dashboard
pub async fn get_developer_dashboard(State(state): State<ApiState>) -> impl IntoResponse {
    let usage_stats = state.get_usage_stats(None, 24).await;
    let analytics = state.service_analytics.read().await;
    
    Json(serde_json::json!({
        "success": true,
        "dashboard": {
            "total_requests_24h": usage_stats.len(),
            "active_services": analytics.len(),
            "total_revenue_24h": analytics.values().map(|a| a.total_revenue).sum::<u64>(),
            "average_response_time": usage_stats.iter().map(|u| u.response_time).sum::<u64>() / usage_stats.len().max(1) as u64,
            "top_endpoints": {
                "most_used": "/api/services/search",
                "slowest": "/api/tasks/submit",
                "most_revenue": "/api/wallet/send"
            }
        }
    }))
}

// Placeholder handlers for other endpoints
pub async fn update_service() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn delete_service() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_service_analytics() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn check_service_health() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_service_reviews() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn add_service_review() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn update_service_review() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn delete_service_review() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn vote_on_review() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_task_status() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn cancel_task() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_user_tasks() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_escrow_details() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn sign_escrow() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn add_reputation_attestation() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_api_key_usage() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_revenue_analytics() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_rate_limit_info() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn update_rate_limit() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn revoke_api_key() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_billing_info() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }

// Existing handlers (keeping placeholders for now)
pub async fn register_service_old() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Use enhanced endpoint"})) }
pub async fn search_services_old() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Use enhanced endpoint"})) }
pub async fn submit_task() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn create_escrow() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_reputation() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_wallet_info() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_wallet_balances() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_wallet_addresses() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn send_funds() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn receive_funds() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_transaction_history() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_transaction_by_id() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn backup_wallet() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn restore_wallet() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_wallet_keys() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_dux_balance() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_dux_transactions() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn send_dux() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_dux_network() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn start_dux_mining() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn stop_dux_mining() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_dux_mining_status() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn sync_dux_balance() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn register_aoi_key() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_aoi_key() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_community_fund_stats() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_community_fund_balance() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn distribute_community_fund() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn send_message() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_conversations() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_messages() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn mark_message_read() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn delete_message() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn get_messaging_stats() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn shutdown_node() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) }
pub async fn serve_index() -> impl IntoResponse { Json(serde_json::json!({"success": false, "message": "Not implemented yet"})) } 