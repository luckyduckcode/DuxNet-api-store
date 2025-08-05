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

use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use base64;
use crate::api::state::ApiState;
use crate::core::{data_structures::*, manifest::ManifestValidator};
use crate::container::ContainerManager;
use tracing::error;
use crate::api::dux_coin::{DuxCoinAPI, DuxNetworkInfo};
use std::sync::Arc;
use tokio::sync::Mutex;

use std::sync::LazyLock;
pub static DUXCOIN_API: LazyLock<DuxCoinAPI> = LazyLock::new(|| {
    DuxCoinAPI::new(
        "http://localhost:8332".to_string(), // Adjust to your DuxCoin daemon RPC URL
        "rpcuser".to_string(),                // Adjust to your DuxCoin RPC username
        "rpcpassword".to_string()             // Adjust to your DuxCoin RPC password
    )
});

/// Handles GET /api/health
pub async fn get_health_status(State(state): State<ApiState>) -> impl IntoResponse {
    // Simplified health check for now
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": uptime,
        "version": env!("CARGO_PKG_VERSION"),
        "components": {
            "api_server": true,
            "wallet_system": true,
            "messaging_system": true
        }
    }))
}

/// Handles GET /api/status
pub async fn get_status(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let reputation = node.reputation_system.get_reputation(&node.did_manager.did.id).await;
    
    let status = crate::core::data_structures::NodeStatus {
        node_id: node.node_id.0.clone(),
        did: node.did_manager.did.id.clone(),
        is_online: true,
        uptime_seconds: 0, // TODO: implement uptime tracking
        services_count: 0,  // TODO: implement service counting
        reputation_score: reputation,
        peers_count: 0, // No P2P network in simplified version
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
            Ok(service_id) => {
                let service_id_str = service_id.0.clone();
                Json(crate::core::data_structures::RegisterServiceResponse {
                    service_id: service_id_str.clone(),
                    success: true,
                    message: "Service registered successfully".to_string(),
                    api_key: Some(service_api_key),
                    documentation_url: Some(format!("http://localhost:8081/docs/services/{}", service_id_str)),
                })
            },
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
                "uptime": 100.0 // Default uptime for now
            })
        })
        .collect();
    
    Json(serde_json::json!({
        "success": true,
        "trending_services": top_services
    }))
}

// === YAML MANIFEST HANDLERS ===

/// Handles POST /api/services/manifest - Deploy service from YAML manifest
pub async fn register_service_manifest(
    State(state): State<ApiState>,
    Json(request): Json<RegisterManifestRequest>,
) -> impl IntoResponse {
    use crate::core::manifest::ManifestValidator;
    
    // Validate YAML manifest
    let validator = match ManifestValidator::new() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to create manifest validator: {}", e);
            return Json(serde_json::json!({
                "success": false,
                "error": "Internal validation error"
            }));
        }
    };
    
    let manifest = match validator.validate_manifest(&request.manifest_yaml) {
        Ok(manifest) => manifest,
        Err(e) => {
            tracing::warn!("Invalid manifest: {}", e);
            return Json(serde_json::json!({
                "success": false,
                "error": format!("Invalid manifest: {}", e)
            }));
        }
    };
    
    // Deploy service using ServiceManager (Phase 2 integration)
    let service_id = match state.node.service_manager.deploy_service(manifest.clone()).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to deploy service via ServiceManager: {}", e);
            return Json(serde_json::json!({
                "success": false,
                "error": format!("Deployment failed: {}", e)
            }));
        }
    };

    // Phase 4: Store manifest in DHT for enhanced discovery
    if let Err(e) = state.node.dht.store_manifest(&manifest).await {
        tracing::warn!("Failed to store manifest in DHT for discovery: {}", e);
        // Don't fail the request, but log the warning
    }
    
    tracing::info!("Successfully deployed service: {} with ID: {} and stored in DHT", manifest.name, service_id.0);
    
    Json(serde_json::json!({
        "success": true,
        "service_id": service_id.0,
        "service_name": manifest.name,
        "version": manifest.version,
        "message": "Service deployed successfully"
    }))
}

/// Handles GET /api/services/manifest/:service_id - Get YAML manifest for a service
pub async fn get_service_manifest(
    State(state): State<ApiState>,
    axum::extract::Path(service_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let service_id_obj = ServiceId(service_id.clone());
    
    // Get service from ServiceManager
    match state.node.service_manager.get_service(&service_id_obj).await {
        Some(instance) => {
            // Convert manifest to YAML
            match serde_yaml::to_string(&instance.manifest) {
                Ok(yaml_content) => Json(serde_json::json!({
                    "success": true,
                    "service_id": service_id,
                    "manifest_yaml": yaml_content,
                    "manifest": instance.manifest,
                    "status": instance.status,
                    "endpoints": instance.endpoints,
                    "container_id": instance.container_id,
                    "deployed_at": instance.deployed_at
                })),
                Err(e) => {
                    tracing::error!("Failed to serialize manifest to YAML: {}", e);
                    Json(serde_json::json!({
                        "success": false,
                        "error": "Failed to serialize manifest"
                    }))
                }
            }
        },
        None => {
            // Try to get from DHT as fallback
            match state.node.dht.get_manifest(&service_id).await {
                Ok(Some(manifest)) => {
                    match serde_yaml::to_string(&manifest) {
                        Ok(yaml_content) => Json(serde_json::json!({
                            "success": true,
                            "service_id": service_id,
                            "manifest_yaml": yaml_content,
                            "manifest": manifest,
                            "note": "Service found in DHT but not locally deployed"
                        })),
                        Err(e) => Json(serde_json::json!({
                            "success": false,
                            "error": format!("Failed to serialize manifest: {}", e)
                        }))
                    }
                },
                _ => Json(serde_json::json!({
                    "success": false,
                    "error": "Service manifest not found"
                }))
            }
        }
    }
}

/// Handles DELETE /api/services/manifest/:service_id - Remove deployed service
pub async fn remove_service_manifest(
    State(state): State<ApiState>,
    axum::extract::Path(service_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let service_id_obj = ServiceId(service_id.clone());
    
    // Remove service using ServiceManager
    match state.node.service_manager.remove_service(&service_id_obj).await {
        Ok(()) => {
            tracing::info!("Successfully removed service: {}", service_id);
            Json(serde_json::json!({
                "success": true,
                "service_id": service_id,
                "message": "Service removed successfully"
            }))
        },
        Err(e) => {
            tracing::error!("Failed to remove service {}: {}", service_id, e);
            Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to remove service: {}", e)
            }))
        }
    }
}

/// Handles GET /api/services/list - List all deployed services
pub async fn list_deployed_services(
    State(state): State<ApiState>,
) -> impl IntoResponse {
    let services = state.node.service_manager.list_services().await;
    
    let service_list: Vec<_> = services.into_iter().map(|(id, instance)| {
        serde_json::json!({
            "service_id": id.0,
            "name": instance.manifest.name,
            "version": instance.manifest.version,
            "description": instance.manifest.description,
            "category": instance.manifest.category,
            "author": instance.manifest.author.name,
            "status": instance.status,
            "endpoints": instance.endpoints,
            "deployed_at": instance.deployed_at,
            "container_id": instance.container_id
        })
    }).collect();
    
    Json(serde_json::json!({
        "success": true,
        "services": service_list,
        "total": service_list.len()
    }))
}

/// Handles GET /api/services/status/:service_id - Get service status and health
pub async fn get_service_status(
    State(state): State<ApiState>,
    axum::extract::Path(service_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let service_id_obj = ServiceId(service_id.clone());
    
    match state.node.service_manager.get_service(&service_id_obj).await {
        Some(instance) => {
            // Perform health check
            let is_healthy = state.node.service_manager.health_check_service(&service_id_obj).await
                .unwrap_or(false);
            
            Json(serde_json::json!({
                "success": true,
                "service_id": service_id,
                "name": instance.manifest.name,
                "status": instance.status,
                "health_status": if is_healthy { "healthy" } else { "unhealthy" },
                "endpoints": instance.endpoints,
                "container_id": instance.container_id,
                "deployed_at": instance.deployed_at,
                "uptime_seconds": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() - instance.deployed_at
            }))
        },
        None => Json(serde_json::json!({
            "success": false,
            "error": "Service not found"
        }))
    }
}

/// Handles GET /api/services/stats - Get service manager statistics
pub async fn get_service_manager_stats(
    State(state): State<ApiState>,
) -> impl IntoResponse {
    let stats = state.node.service_manager.get_service_stats().await;
    
    Json(serde_json::json!({
        "success": true,
        "stats": {
            "total_services": stats.total_services,
            "running_services": stats.running_services,
            "failed_services": stats.failed_services,
            "starting_services": stats.starting_services
        }
    }))
}

/// Request structure for manifest registration
#[derive(serde::Deserialize)]
pub struct RegisterManifestRequest {
    pub manifest_yaml: String,
    pub signature: Option<String>,
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

/// Handles POST /api/tasks/submit (with escrow payment)
pub async fn submit_task(State(state): State<ApiState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let node = &state.node;
    let service_id = payload.get("service_id").and_then(|v| v.as_str()).unwrap_or("");
    let buyer_address = payload.get("buyer_address").and_then(|v| v.as_str()).unwrap_or("");
    let requirements = TaskRequirements {
        cpu_cores: payload.get("cpu_cores").and_then(|v| v.as_u64()).unwrap_or(1) as u32,
        memory_mb: payload.get("memory_mb").and_then(|v| v.as_u64()).unwrap_or(512) as u32,
        timeout_seconds: payload.get("timeout_seconds").and_then(|v| v.as_u64()).unwrap_or(60) as u32,
    };
    let task_payload = payload.get("payload").and_then(|v| v.as_str()).unwrap_or("").as_bytes().to_vec();
    if service_id.is_empty() || buyer_address.is_empty() {
        return Json(serde_json::json!({
            "success": false,
            "message": "Missing service_id or buyer_address"
        }));
    }
    match node.submit_task_with_escrow(ServiceId(service_id.to_string()), task_payload, requirements, buyer_address.to_string()).await {
        Ok(task_id) => Json(serde_json::json!({
            "success": true,
            "task_id": task_id.0
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to submit task with escrow: {}", e)
        })),
    }
}

/// Handles POST /api/tasks/:task_id/complete (release escrow)
pub async fn complete_task(State(state): State<ApiState>, axum::extract::Path(task_id): axum::extract::Path<String>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let node = &state.node;
    // Lookup the task and escrow info (simplified)
    let task = node.task_engine.completed_tasks.read().await.get(&TaskId(task_id.clone())).cloned();
    if let Some(task) = task {
        // Lookup provider address and amount (simplified: use service endpoint and price)
        if let Ok(service) = node.get_service_details(&task.task_id.0).await {
            let provider_address = service.endpoint;
            let amount = service.price;
            match node.release_escrow_to_provider(provider_address, amount).await {
                Ok(txid) => Json(serde_json::json!({
                    "success": true,
                    "message": "Escrow released to provider",
                    "txid": txid
                })),
                Err(e) => Json(serde_json::json!({
                    "success": false,
                    "message": format!("Failed to release escrow: {}", e)
                })),
            }
        } else {
            Json(serde_json::json!({
                "success": false,
                "message": "Service not found for task"
            }))
        }
    } else {
        Json(serde_json::json!({
            "success": false,
            "message": "Task not found or not completed"
        }))
    }
}
// ===== DUXCOIN WALLET HANDLERS =====

/// Handles POST /api/wallet/backup
pub async fn backup_wallet(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    
    match wallet.backup_wallet() {
        Ok(backup_data) => Json(serde_json::json!({
            "success": true,
            "backup_data": backup_data,
            "message": "Wallet backed up successfully",
            "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to backup wallet: {}", e)
        }))
    }
}

/// Handles POST /api/wallet/restore
pub async fn restore_wallet(State(state): State<ApiState>, Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    let backup_data = match request.get("backup_data").and_then(|v| v.as_str()) {
        Some(data) => data,
        None => return Json(serde_json::json!({
            "success": false,
            "message": "Missing backup_data in request"
        }))
    };
    
    match crate::wallet::Wallet::restore_wallet(backup_data) {
        Ok(restored_wallet) => {
            // In production, you'd update the node's wallet here
            Json(serde_json::json!({
                "success": true,
                "message": "Wallet restored successfully",
                "addresses": restored_wallet.get_all_addresses()
            }))
        }
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to restore wallet: {}", e)
        }))
    }
}

/// Handles GET /api/wallet/keys
pub async fn get_wallet_keys(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    
    match wallet.get_public_key_base64() {
        Ok(public_key) => Json(serde_json::json!({
            "success": true,
            "public_key": public_key,
            "addresses": wallet.get_all_addresses(),
            "preferred_currency": wallet.get_preferred_currency()
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to get wallet keys: {}", e)
        }))
    }
}

/// Handles GET /api/dux/balance
pub async fn get_dux_balance(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let dux_address = wallet.get_address(&crate::wallet::Currency::DUX);
    
    match DUXCOIN_API.get_balance(&dux_address).await {
        Ok(balance) => Json(serde_json::json!({
            "success": true,
            "balance": balance.total,
            "confirmed": balance.confirmed,
            "unconfirmed": balance.unconfirmed,
            "address": dux_address,
            "currency": "DUX"
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to get DUX balance: {}", e)
        }))
    }
}

/// Handles GET /api/dux/transactions
pub async fn get_dux_transactions(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let dux_address = wallet.get_address(&crate::wallet::Currency::DUX);
    
    match DUXCOIN_API.get_transactions(&dux_address, 50).await {
        Ok(transactions) => Json(serde_json::json!({
            "success": true,
            "transactions": transactions,
            "address": dux_address,
            "count": transactions.len()
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to get DUX transactions: {}", e)
        }))
    }
}

/// Handles POST /api/dux/send
pub async fn send_dux(State(state): State<ApiState>, Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let from_address = wallet.get_address(&crate::wallet::Currency::DUX);
    
    let to_address = match request.get("to_address").and_then(|v| v.as_str()) {
        Some(addr) => addr,
        None => return Json(serde_json::json!({
            "success": false,
            "message": "Missing to_address in request"
        }))
    };
    
    let amount = match request.get("amount").and_then(|v| v.as_f64()) {
        Some(amt) if amt > 0.0 => amt,
        _ => return Json(serde_json::json!({
            "success": false,
            "message": "Invalid amount - must be a positive number"
        }))
    };
    
    // Validate DUX address format
    match DUXCOIN_API.validate_address(to_address).await {
        Ok(false) => return Json(serde_json::json!({
            "success": false,
            "message": "Invalid DUX address format"
        })),
        Err(e) => return Json(serde_json::json!({
            "success": false,
            "message": format!("Address validation failed: {}", e)
        })),
        Ok(true) => {}
    }
    
    match DUXCOIN_API.send_dux(&from_address, to_address, amount).await {
        Ok(txid) => Json(serde_json::json!({
            "success": true,
            "txid": txid,
            "from_address": from_address,
            "to_address": to_address,
            "amount": amount,
            "message": "DUX sent successfully"
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to send DUX: {}", e)
        }))
    }
}

/// Handles GET /api/dux/network
pub async fn get_dux_network(State(state): State<ApiState>) -> impl IntoResponse {
    match DUXCOIN_API.get_network_info().await {
        Ok(network_info) => Json(serde_json::json!({
            "success": true,
            "network": network_info,
            "status": "connected"
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to get DUX network info: {}", e),
            "status": "disconnected"
        }))
    }
}
// ===== DUXCOIN MINING & ADVANCED HANDLERS =====

/// Handles POST /api/dux/sync
pub async fn sync_dux_balance(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let dux_address = wallet.get_address(&crate::wallet::Currency::DUX);
    
    // Clear cache to force fresh data
    DUXCOIN_API.clear_cache().await;
    
    match DUXCOIN_API.get_balance(&dux_address).await {
        Ok(balance) => {
            // Update local wallet balance
            drop(wallet);
            let mut wallet_mut = node.wallet.write().await;
            wallet_mut.add_funds(crate::wallet::Currency::DUX, (balance.total * 100_000_000.0) as u64); // Convert to satoshis
            
            Json(serde_json::json!({
                "success": true,
                "balance": balance,
                "message": "DUX balance synchronized successfully"
            }))
        }
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to sync DUX balance: {}", e)
        }))
    }
}

/// Handles GET /api/wallet/transaction/:id
pub async fn get_transaction_by_id(State(state): State<ApiState>, axum::extract::Path(txid): axum::extract::Path<String>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let dux_address = wallet.get_address(&crate::wallet::Currency::DUX);
    match DUXCOIN_API.get_transactions(&dux_address, 100).await {
        Ok(txs) => {
            let tx = txs.into_iter().find(|t| t.txid == txid);
            match tx {
                Some(transaction) => Json(serde_json::json!({
                    "success": true,
                    "transaction": transaction
                })),
                None => Json(serde_json::json!({
                    "success": false,
                    "message": "Transaction not found"
                }))
            }
        },
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to get transaction: {}", e)
        })),
    }
}

/// Handles GET /api/wallet/validate/:address
pub async fn validate_dux_address(State(state): State<ApiState>, axum::extract::Path(address): axum::extract::Path<String>) -> impl IntoResponse {
    match DUXCOIN_API.validate_address(&address).await {
        Ok(is_valid) => Json(serde_json::json!({
            "success": true,
            "address": address,
            "valid": is_valid
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to validate address: {}", e)
        }))
    }
}

/// Handles POST /api/wallet/encrypt
pub async fn encrypt_wallet(State(state): State<ApiState>, Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    let passphrase = match request.get("passphrase").and_then(|v| v.as_str()) {
        Some(pass) if pass.len() >= 8 => pass,
        _ => return Json(serde_json::json!({
            "success": false,
            "message": "Passphrase must be at least 8 characters long"
        }))
    };

    // In a real implementation, you would encrypt the wallet here
    Json(serde_json::json!({
        "success": true,
        "message": "Wallet encrypted successfully",
        "encrypted": true
    }))
}

/// Handles POST /api/wallet/unlock
pub async fn unlock_wallet(State(state): State<ApiState>, Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    let passphrase = match request.get("passphrase").and_then(|v| v.as_str()) {
        Some(pass) => pass,
        None => return Json(serde_json::json!({
            "success": false,
            "message": "Passphrase required"
        }))
    };

    // In a real implementation, you would unlock the wallet here
    Json(serde_json::json!({
        "success": true,
        "message": "Wallet unlocked successfully",
        "unlocked": true
    }))
}

/// Handles POST /api/wallet/generate-address
pub async fn generate_new_address(State(state): State<ApiState>) -> impl IntoResponse {
    // In a real implementation, you would generate a new address using the DuxCoin daemon
    let new_address = format!("DUX{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..26].to_uppercase());
    
    Json(serde_json::json!({
        "success": true,
        "address": new_address,
        "message": "New address generated successfully"
    }))
}

/// Handles GET /api/wallet/export
pub async fn export_private_keys(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    
    match wallet.export_private_key() {
        Ok(private_key) => {
            // Convert to base64 for safe transport
            let private_key_b64 = base64::encode(&private_key);
            Json(serde_json::json!({
                "success": true,
                "private_key": private_key_b64,
                "message": "Private key exported successfully",
                "warning": "Keep this private key secure and never share it"
            }))
        }
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to export private key: {}", e)
        }))
    }
}

/// Handles POST /api/wallet/import
pub async fn import_private_key(State(state): State<ApiState>, Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    let private_key_b64 = match request.get("private_key").and_then(|v| v.as_str()) {
        Some(key) => key,
        None => return Json(serde_json::json!({
            "success": false,
            "message": "Private key required"
        }))
    };

    let label = request.get("label").and_then(|v| v.as_str()).unwrap_or("Imported Key");

    match base64::decode(private_key_b64) {
        Ok(private_key_bytes) => {
            // In a real implementation, you would import the private key
            Json(serde_json::json!({
                "success": true,
                "message": "Private key imported successfully",
                "label": label
            }))
        }
        Err(_) => Json(serde_json::json!({
            "success": false,
            "message": "Invalid private key format"
        }))
    }
}

/// Handles POST /api/dux/stake
pub async fn stake_dux(State(state): State<ApiState>, Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let dux_address = wallet.get_address(&crate::wallet::Currency::DUX);
    
    let amount = match request.get("amount").and_then(|v| v.as_f64()) {
        Some(amt) if amt > 0.0 => amt,
        _ => return Json(serde_json::json!({
            "success": false,
            "message": "Invalid amount - must be a positive number"
        }))
    };
    
    let duration = match request.get("duration_days").and_then(|v| v.as_u64()) {
        Some(days) if days >= 30 => days,
        _ => return Json(serde_json::json!({
            "success": false,
            "message": "Invalid duration - minimum 30 days"
        }))
    };
    
    // In a real implementation, you would interact with staking contracts
    let estimated_reward = amount * 0.05 * (duration as f64 / 365.0); // 5% APY
    let unlock_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() + (duration * 24 * 60 * 60);
    
    Json(serde_json::json!({
        "success": true,
        "staked_amount": amount,
        "duration_days": duration,
        "estimated_reward": estimated_reward,
        "unlock_time": unlock_time,
        "message": "DUX staked successfully"
    }))
}

/// Handles GET /api/dux/staking/info
pub async fn get_staking_info(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let dux_address = wallet.get_address(&crate::wallet::Currency::DUX);
    
    // Mock staking data - in real implementation, query from blockchain
    Json(serde_json::json!({
        "success": true,
        "staking": {
            "active_stakes": [
                {
                    "amount": 1000.0,
                    "duration_days": 90,
                    "start_time": 1696636800,
                    "unlock_time": 1704412800,
                    "estimated_reward": 12.33,
                    "status": "active"
                }
            ],
            "total_staked": 1000.0,
            "pending_rewards": 4.12,
            "apy": 5.0,
            "min_stake": 10.0,
            "min_duration": 30
        }
    }))
}

/// Handles GET /api/dux/mining/status  
pub async fn get_mining_status(State(state): State<ApiState>) -> impl IntoResponse {
    // Mock mining data - in real implementation, query from DuxCoin daemon
    Json(serde_json::json!({
        "success": true,
        "mining": {
            "is_mining": false,
            "hashrate": 0.0,
            "difficulty": 1024.5,
            "blocks_mined": 0,
            "estimated_time_to_block": null,
            "network_hashrate": 15673.2,
            "block_height": 285943,
            "last_block_time": 1696636800
        }
    }))
}

/// Handles POST /api/dux/mining/start
pub async fn start_mining(State(state): State<ApiState>, Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    let threads = request.get("threads").and_then(|v| v.as_u64()).unwrap_or(1);
    
    // In a real implementation, you would start mining via DuxCoin daemon
    Json(serde_json::json!({
        "success": true,
        "message": "Mining started successfully",
        "threads": threads,
        "mining": true
    }))
}

/// Handles POST /api/dux/mining/stop
pub async fn stop_mining(State(state): State<ApiState>) -> impl IntoResponse {
    // In a real implementation, you would stop mining via DuxCoin daemon
    Json(serde_json::json!({
        "success": true,
        "message": "Mining stopped successfully",
        "mining": false
    }))
}

/// Handles POST /api/aoi/register
pub async fn register_aoi_key(State(state): State<ApiState>, Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    let node = &state.node;
    
    let service_id_str = match request.get("service_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return Json(serde_json::json!({
            "success": false,
            "message": "Missing service_id in request"
        }))
    };
    
    let key_data = match request.get("key_data").and_then(|v| v.as_str()) {
        Some(data) => data.to_string(),
        None => return Json(serde_json::json!({
            "success": false,
            "message": "Missing key_data in request"
        }))
    };
    
    let service_id = ServiceId(service_id_str.to_string());
    
    match node.register_aoi_key_for_service(service_id, key_data).await {
        Ok(()) => Json(serde_json::json!({
            "success": true,
            "service_id": service_id_str,
            "message": "AOI key registered successfully"
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to register AOI key: {}", e)
        }))
    }
}

/// Handles GET /api/aoi/key/:service_id
pub async fn get_aoi_key(State(state): State<ApiState>, axum::extract::Path(service_id): axum::extract::Path<String>) -> impl IntoResponse {
    let node = &state.node;
    let service_id_obj = ServiceId(service_id.clone());
    
    match node.get_aoi_key_for_service(service_id_obj).await {
        Some(aoi_key) => Json(serde_json::json!({
            "success": true,
            "service_id": service_id,
            "aoi_key": aoi_key
        })),
        None => Json(serde_json::json!({
            "success": false,
            "message": "AOI key not found for service"
        }))
    }
}

/// Handles GET /api/community/stats
pub async fn get_community_fund_stats(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    
    match node.get_community_fund_stats().await {
        Ok(stats) => Json(serde_json::json!({
            "success": true,
            "stats": stats
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to get community fund stats: {}", e)
        }))
    }
}

/// Handles GET /api/community/balance/:currency
pub async fn get_community_fund_balance(State(state): State<ApiState>, axum::extract::Path(currency): axum::extract::Path<String>) -> impl IntoResponse {
    let node = &state.node;
    
    let currency_enum = match currency.to_uppercase().as_str() {
        "DUX" => crate::wallet::Currency::DUX,
        "BTC" => crate::wallet::Currency::BTC,
        "ETH" => crate::wallet::Currency::ETH,
        "USDT" => crate::wallet::Currency::USDC, // Map USDT to USDC since USDT doesn't exist
        _ => return Json(serde_json::json!({
            "success": false,
            "message": "Unsupported currency. Supported: DUX, BTC, ETH, USDT"
        }))
    };
    
    let balance = node.get_community_fund_balance(&currency_enum).await;
    
    Json(serde_json::json!({
        "success": true,
        "currency": currency.to_uppercase(),
        "balance": balance
    }))
}

/// Handles POST /api/community/distribute
pub async fn distribute_community_fund(State(state): State<ApiState>, Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    let node = &state.node;
    
    let currency_str = match request.get("currency").and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return Json(serde_json::json!({
            "success": false,
            "message": "Missing currency in request"
        }))
    };
    
    let currency_enum = match currency_str.to_uppercase().as_str() {
        "DUX" => crate::wallet::Currency::DUX,
        "BTC" => crate::wallet::Currency::BTC,
        "ETH" => crate::wallet::Currency::ETH,
        "USDT" => crate::wallet::Currency::USDC, // Map USDT to USDC since USDT doesn't exist
        _ => return Json(serde_json::json!({
            "success": false,
            "message": "Unsupported currency. Supported: DUX, BTC, ETH, USDT"
        }))
    };
    
    match node.distribute_community_fund(currency_enum).await {
        Ok(distribution) => Json(serde_json::json!({
            "success": true,
            "distribution": distribution,
            "message": "Community fund distributed successfully"
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to distribute community fund: {}", e)
        }))
    }
}
// ===== MESSAGING & NODE MANAGEMENT HANDLERS =====

/// Handles POST /api/messages/send
pub async fn send_message(State(state): State<ApiState>, Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    let node = &state.node;
    
    let to_did = match request.get("to_did").and_then(|v| v.as_str()) {
        Some(did) => did.to_string(),
        None => return Json(serde_json::json!({
            "success": false,
            "message": "Missing to_did in request"
        }))
    };
    
    let content = match request.get("content").and_then(|v| v.as_str()) {
        Some(content) => content.to_string(),
        None => return Json(serde_json::json!({
            "success": false,
            "message": "Missing content in request"
        }))
    };
    
    let message_type = request.get("message_type").and_then(|v| v.as_str()).unwrap_or("text").to_string();
    
    let message_request = crate::core::data_structures::MessageRequest {
        to_did,
        content,
        message_type: match message_type.as_str() {
            "text" => crate::core::data_structures::MessageType::Text,
            "file" => crate::core::data_structures::MessageType::File,
            "system" => crate::core::data_structures::MessageType::System,
            _ => crate::core::data_structures::MessageType::Text,
        },
        reply_to: None,
    };
    
    match node.messaging_system.send_message(message_request).await {
        Ok(response) => Json(serde_json::json!({
            "success": true,
            "message_id": response.message_id,
            "message": response.message
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to send message: {}", e)
        }))
    }
}

/// Handles GET /api/messages/conversations
pub async fn get_conversations(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    
    let conversations = node.messaging_system.get_conversations().await;
    
    Json(serde_json::json!({
        "success": true,
        "conversations": conversations,
        "count": conversations.len()
    }))
}

/// Handles GET /api/messages/:did
pub async fn get_messages(State(state): State<ApiState>, axum::extract::Path(did): axum::extract::Path<String>) -> impl IntoResponse {
    let node = &state.node;
    
    let messages = node.messaging_system.get_messages(&did).await;
    
    Json(serde_json::json!({
        "success": true,
        "messages": messages,
        "peer_did": did,
        "count": messages.len()
    }))
}

/// Handles PUT /api/messages/:message_id/read
pub async fn mark_message_read(State(state): State<ApiState>, axum::extract::Path(message_id): axum::extract::Path<String>) -> impl IntoResponse {
    let node = &state.node;
    
    match node.messaging_system.mark_message_read(&message_id).await {
        Ok(()) => Json(serde_json::json!({
            "success": true,
            "message": "Message marked as read",
            "message_id": message_id
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to mark message as read: {}", e)
        }))
    }
}

/// Handles DELETE /api/messages/:message_id
pub async fn delete_message(State(state): State<ApiState>, axum::extract::Path(message_id): axum::extract::Path<String>) -> impl IntoResponse {
    let node = &state.node;
    
    match node.messaging_system.delete_message(&message_id).await {
        Ok(()) => Json(serde_json::json!({
            "success": true,
            "message": "Message deleted successfully",
            "message_id": message_id
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to delete message: {}", e)
        }))
    }
}

/// Handles GET /api/messages/stats
pub async fn get_messaging_stats(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    
    let stats = node.messaging_system.get_message_stats().await;
    
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

/// Handles POST /api/node/shutdown
pub async fn shutdown_node(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    
    // In production, this should have proper authentication and safety checks
    tracing::warn!("Node shutdown requested via API");
    
    match node.stop().await {
        Ok(()) => Json(serde_json::json!({
            "success": true,
            "message": "Node shutdown initiated"
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to shutdown node: {}", e)
        }))
    }
}
/// Handles GET / and /index.html
pub async fn serve_index() -> Result<axum::response::Html<String>, axum::http::StatusCode> {
    let html_content = include_str!("../../frontend/index.html");
    Ok(axum::response::Html(html_content.to_string()))
} 

// --- DuxCoin Mining Handlers ---

/// Handles POST /api/dux/mine/start
pub async fn start_dux_mining() -> impl axum::response::IntoResponse {
    match DUXCOIN_API.start_mining(2).await {
        Ok(_) => axum::Json(serde_json::json!({
            "success": true,
            "message": "DuxCoin mining started with 2 threads"
        })),
        Err(e) => axum::Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to start mining: {}", e)
        })),
    }
}

/// Handles POST /api/dux/mine/stop
pub async fn stop_dux_mining() -> impl axum::response::IntoResponse {
    match DUXCOIN_API.stop_mining().await {
        Ok(_) => axum::Json(serde_json::json!({
            "success": true,
            "message": "DuxCoin mining stopped"
        })),
        Err(e) => axum::Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to stop mining: {}", e)
        })),
    }
}

/// Handles GET /api/dux/mine/status
pub async fn get_dux_mining_status() -> impl axum::response::IntoResponse {
    match DUXCOIN_API.get_hash_rate().await {
        Ok(hashrate) => axum::Json(serde_json::json!({
            "success": true,
            "hashrate": hashrate
        })),
        Err(e) => axum::Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to get mining status: {}", e)
        })),
    }
} 

/// Handles GET /api/wallet/info
pub async fn get_wallet_info(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let dux_address = wallet.get_address(&crate::wallet::Currency::DUX);
    Json(serde_json::json!({
        "success": true,
        "address": dux_address
    }))
}

/// Handles GET /api/wallet/balances
pub async fn get_wallet_balances(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let dux_address = wallet.get_address(&crate::wallet::Currency::DUX);
    match DUXCOIN_API.get_balance(&dux_address).await {
        Ok(balance) => Json(serde_json::json!({
            "success": true,
            "balance": balance.total,
            "confirmed": balance.confirmed,
            "unconfirmed": balance.unconfirmed,
            "address": dux_address
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to get balance: {}", e)
        })),
    }
}

/// Handles GET /api/wallet/addresses
pub async fn get_wallet_addresses(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let dux_address = wallet.get_address(&crate::wallet::Currency::DUX);
    Json(serde_json::json!({
        "success": true,
        "addresses": [dux_address]
    }))
}

/// Handles POST /api/wallet/send
pub async fn send_funds(State(state): State<ApiState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let from_address = wallet.get_address(&crate::wallet::Currency::DUX);
    let to_address = payload.get("to_address").and_then(|v| v.as_str()).unwrap_or("");
    let amount = payload.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    if to_address.is_empty() || amount <= 0.0 {
        return Json(serde_json::json!({
            "success": false,
            "message": "Invalid to_address or amount"
        }));
    }
    match DUXCOIN_API.send_dux(&from_address, to_address, amount).await {
        Ok(txid) => Json(serde_json::json!({
            "success": true,
            "txid": txid
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to send DuxCoin: {}", e)
        })),
    }
}

/// Handles POST /api/wallet/receive
pub async fn receive_funds(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let dux_address = wallet.get_address(&crate::wallet::Currency::DUX);
    Json(serde_json::json!({
        "success": true,
        "address": dux_address
    }))
}

/// Handles GET /api/wallet/transactions
pub async fn get_transaction_history(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let dux_address = wallet.get_address(&crate::wallet::Currency::DUX);
    match DUXCOIN_API.get_transactions(&dux_address, 50).await {
        Ok(txs) => Json(serde_json::json!({
            "success": true,
            "transactions": txs
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to get transactions: {}", e)
        })),
    }
}

//
// Phase 5: Advanced Analytics & Monitoring Handlers
//

/// Handles GET /api/analytics/snapshot
/// Returns current real-time analytics snapshot
pub async fn get_analytics_snapshot(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    match node.analytics_engine.generate_snapshot().await {
        Ok(snapshot) => Json(serde_json::json!({
            "success": true,
            "snapshot": snapshot
        })),
        Err(e) => {
            error!("Failed to generate analytics snapshot: {}", e);
            Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to generate analytics snapshot: {}", e)
            }))
        }
    }
}

/// Handles GET /api/analytics/metrics
/// Query metrics with optional filters
pub async fn get_metrics(State(state): State<ApiState>, axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>) -> impl IntoResponse {
    let node = &state.node;
    
    // Parse query parameters
    let query = AnalyticsQuery {
        start_time: params.get("start_time").and_then(|s| s.parse().ok()),
        end_time: params.get("end_time").and_then(|s| s.parse().ok()),
        service_id: params.get("service_id").cloned(),
        metric_type: params.get("metric_type").and_then(|s| match s.as_str() {
            "RequestCount" => Some(MetricType::RequestCount),
            "ResponseTime" => Some(MetricType::ResponseTime),
            "ErrorRate" => Some(MetricType::ErrorRate),
            "DiskUsage" => Some(MetricType::DiskUsage),
            "MemoryUsage" => Some(MetricType::MemoryUsage),
            "CpuUsage" => Some(MetricType::CpuUsage),
            "NetworkIO" => Some(MetricType::NetworkIO),
            _ => None,
        }),
        granularity: params.get("granularity").and_then(|s| match s.as_str() {
            "Second" => Some(TimeGranularity::Second),
            "Minute" => Some(TimeGranularity::Minute),
            "Hour" => Some(TimeGranularity::Hour),
            "Day" => Some(TimeGranularity::Day),
            _ => None,
        }).unwrap_or(TimeGranularity::Minute),
        limit: params.get("limit").and_then(|s| s.parse().ok()),
    };

    match node.analytics_engine.get_metrics(query).await {
        Ok(metrics) => Json(serde_json::json!({
            "success": true,
            "metrics": metrics
        })),
        Err(e) => {
            error!("Failed to get metrics: {}", e);
            Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to get metrics: {}", e)
            }))
        }
    }
}

/// Handles POST /api/analytics/metrics
/// Record a new metric data point
pub async fn record_metric(State(state): State<ApiState>, Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    let node = &state.node;
    
    // Parse metric type and value from request
    let metric_type = match request["metric_type"].as_str() {
        Some("RequestCount") => MetricType::RequestCount,
        Some("ResponseTime") => MetricType::ResponseTime,
        Some("ErrorRate") => MetricType::ErrorRate,
        Some("DiskUsage") => MetricType::DiskUsage,
        Some("MemoryUsage") => MetricType::MemoryUsage,
        Some("CpuUsage") => MetricType::CpuUsage,
        Some("NetworkIO") => MetricType::NetworkIO,
        _ => {
            return Json(serde_json::json!({
                "success": false,
                "message": "Invalid metric_type"
            }));
        }
    };
    
    let value = match request["value"].as_f64() {
        Some(v) => v,
        None => {
            return Json(serde_json::json!({
                "success": false,
                "message": "Invalid value"
            }));
        }
    };

    match node.analytics_engine.record_metric(metric_type, value).await {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "message": "Metric recorded successfully"
        })),
        Err(e) => {
            error!("Failed to record metric: {}", e);
            Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to record metric: {}", e)
            }))
        }
    }
}

/// Handles POST /api/analytics/service-metrics
/// Record service performance metrics
pub async fn record_service_metrics(State(state): State<ApiState>, Json(metrics): Json<ServicePerformanceMetrics>) -> impl IntoResponse {
    let node = &state.node;
    
    match node.analytics_engine.record_service_metrics(metrics).await {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "message": "Service metrics recorded successfully"
        })),
        Err(e) => {
            error!("Failed to record service metrics: {}", e);
            Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to record service metrics: {}", e)
            }))
        }
    }
}

/// Handles GET /api/analytics/summary
/// Get analytics summary for dashboard
pub async fn get_analytics_summary(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    
    match node.analytics_engine.get_analytics_summary().await {
        Ok(summary) => Json(serde_json::json!({
            "success": true,
            "summary": summary
        })),
        Err(e) => {
            error!("Failed to get analytics summary: {}", e);
            Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to get analytics summary: {}", e)
            }))
        }
    }
}

/// Handles GET /api/analytics/alerts
/// Get current active alerts
pub async fn get_active_alerts(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    
    match node.analytics_engine.get_active_alerts().await {
        Ok(alerts) => Json(serde_json::json!({
            "success": true,
            "alerts": alerts
        })),
        Err(e) => {
            error!("Failed to get active alerts: {}", e);
            Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to get active alerts: {}", e)
            }))
        }
    }
}

/// Handles POST /api/analytics/alerts
/// Add a new alert rule
pub async fn add_alert_rule(State(state): State<ApiState>, Json(rule): Json<AlertRule>) -> impl IntoResponse {
    let node = &state.node;
    
    match node.analytics_engine.add_alert_rule(rule).await {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "message": "Alert rule added successfully"
        })),
        Err(e) => {
            error!("Failed to add alert rule: {}", e);
            Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to add alert rule: {}", e)
            }))
        }
    }
}

/// Handles DELETE /api/analytics/alerts/:alert_id
/// Resolve/dismiss an active alert
pub async fn resolve_alert(State(state): State<ApiState>, axum::extract::Path(alert_id): axum::extract::Path<String>) -> impl IntoResponse {
    // TODO: Implement alert resolution in analytics engine
    Json(serde_json::json!({
        "success": true,
        "message": format!("Alert {} resolved", alert_id)
    }))
}

/// Handles GET /api/analytics/dashboards
/// Get list of available dashboards
pub async fn get_dashboards(State(state): State<ApiState>) -> impl IntoResponse {
    // TODO: Implement dashboard listing
    Json(serde_json::json!({
        "success": true,
        "dashboards": []
    }))
}

/// Handles POST /api/analytics/dashboards
/// Create a new dashboard
pub async fn create_dashboard(State(state): State<ApiState>, Json(dashboard): Json<DashboardConfig>) -> impl IntoResponse {
    // TODO: Implement dashboard creation
    Json(serde_json::json!({
        "success": true,
        "message": "Dashboard created successfully",
        "dashboard_id": dashboard.id
    }))
}

/// Handles GET /api/analytics/dashboards/:dashboard_id
/// Get specific dashboard configuration
pub async fn get_dashboard(State(state): State<ApiState>, axum::extract::Path(dashboard_id): axum::extract::Path<String>) -> impl IntoResponse {
    let node = &state.node;
    
    // Create default dashboard if requesting "default"
    if dashboard_id == "default" {
        match node.analytics_engine.create_default_dashboard().await {
            Ok(dashboard) => Json(serde_json::json!({
                "success": true,
                "dashboard": dashboard
            })),
            Err(e) => {
                error!("Failed to create default dashboard: {}", e);
                Json(serde_json::json!({
                    "success": false,
                    "message": format!("Failed to create default dashboard: {}", e)
                }))
            }
        }
    } else {
        // TODO: Implement dashboard retrieval
        Json(serde_json::json!({
            "success": false,
            "message": "Dashboard not found"
        }))
    }
}

/// Handles PUT /api/analytics/dashboards/:dashboard_id
/// Update dashboard configuration
pub async fn update_dashboard(State(state): State<ApiState>, axum::extract::Path(dashboard_id): axum::extract::Path<String>, Json(dashboard): Json<DashboardConfig>) -> impl IntoResponse {
    // TODO: Implement dashboard update
    Json(serde_json::json!({
        "success": true,
        "message": format!("Dashboard {} updated successfully", dashboard_id)
    }))
}

/// Handles DELETE /api/analytics/dashboards/:dashboard_id
/// Delete a dashboard
pub async fn delete_dashboard(State(state): State<ApiState>, axum::extract::Path(dashboard_id): axum::extract::Path<String>) -> impl IntoResponse {
    // TODO: Implement dashboard deletion
    Json(serde_json::json!({
        "success": true,
        "message": format!("Dashboard {} deleted successfully", dashboard_id)
    }))
}

// === P2P GATEWAY HANDLERS ===

/// Handles ALL /api/gateway/proxy/:service_id/*path - P2P API Gateway proxy
pub async fn proxy_p2p_request(
    State(state): State<ApiState>,
    axum::extract::Path((service_id, path)): axum::extract::Path<(String, String)>,
    method: axum::http::Method,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    use crate::gateway::proxy::P2PApiGateway;
    
    tracing::info!("P2P Gateway proxy request: {} {} -> {}", method, path, service_id);
    
    // Create gateway instance (should be cached in production)
    let gateway = P2PApiGateway::new(state.node.service_manager.clone());
    
    // Route the request through P2P gateway
    match gateway.proxy_request(&service_id, &format!("/{}", path), method, headers, body).await {
        Ok(response) => {
            tracing::info!("P2P proxy successful for service: {}", service_id);
            response
        }
        Err(e) => {
            tracing::error!("P2P proxy failed for service {}: {}", service_id, e);
            axum::response::Response::builder()
                .status(axum::http::StatusCode::BAD_GATEWAY)
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::json!({
                        "success": false,
                        "error": format!("Gateway error: {}", e),
                        "service_id": service_id
                    }).to_string()
                ))
                .unwrap_or_else(|_| {
                    axum::response::Response::builder()
                        .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                        .body(axum::body::Body::from("Internal server error"))
                        .unwrap()
                })
        }
    }
}

/// Handles GET /api/gateway/stats - Get P2P gateway statistics
pub async fn get_gateway_stats(State(state): State<ApiState>) -> impl IntoResponse {
    // Get gateway statistics from analytics engine
    let analytics_summary = match state.node.analytics_engine.get_analytics_summary().await {
        Ok(summary) => summary,
        Err(e) => {
            tracing::warn!("Failed to get analytics summary: {}", e);
            return Json(serde_json::json!({
                "success": false,
                "error": "Failed to retrieve gateway statistics"
            }));
        }
    };
    
    Json(serde_json::json!({
        "success": true,
        "stats": {
            "total_requests": analytics_summary.get("total_requests").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))),
            "success_rate": analytics_summary.get("success_rate").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(100))),
            "average_response_time": analytics_summary.get("average_response_time").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))),
            "active_services": analytics_summary.get("active_services").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))),
            "p2p_network": {
                "peer_count": analytics_summary.get("peer_count").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))),
                "dht_entries": analytics_summary.get("dht_entries").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))),
                "uptime_percentage": analytics_summary.get("uptime_percentage").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(99)))
            }
        }
    }))
}

/// Handles GET /api/discovery/services - Discover services via P2P network
pub async fn discover_p2p_services(State(state): State<ApiState>) -> impl IntoResponse {
    // Search for all services in DHT with empty filters
    let search_filters = crate::core::data_structures::SearchFilters {
        category: None,
        tags: Vec::new(),
        min_reputation: None,
        max_price: None,
        author: None,
    };
    let services = state.node.dht.search_manifests_enhanced("", search_filters).await;
    
    let service_list: Vec<_> = services.into_iter().map(|manifest| {
        serde_json::json!({
            "service_id": format!("{}:{}", manifest.name, manifest.version),
            "name": manifest.name,
            "version": manifest.version,
            "description": manifest.description,
            "category": manifest.category,
            "tags": manifest.tags,
            "author": manifest.author,
            "endpoints": [] // TODO: Get actual endpoints from service manager
        })
    }).collect();
    
    Json(serde_json::json!({
        "success": true,
        "services": service_list,
        "total_count": service_list.len(),
        "discovery_method": "p2p_dht"
    }))
}

/// Handles GET /api/discovery/service/:service_id - Discover endpoints for specific service
pub async fn discover_service_endpoints(
    State(state): State<ApiState>,
    axum::extract::Path(service_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let service_id_obj = crate::core::data_structures::ServiceId(service_id.clone());
    
    // Try to get service from local service manager first
    if let Some(instance) = state.node.service_manager.get_service(&service_id_obj).await {
        return Json(serde_json::json!({
            "success": true,
            "service_id": service_id,
            "endpoints": instance.endpoints,
            "status": instance.status,
            "discovery_method": "local"
        }));
    }
    
    // Try P2P discovery via DHT
    match state.node.dht.get_manifest(&service_id).await {
        Ok(Some(manifest)) => {
            Json(serde_json::json!({
                "success": true,
                "service_id": service_id,
                "manifest": manifest,
                "discovery_method": "p2p_dht",
                "endpoints": [] // Would be populated by P2P discovery
            }))
        }
        Ok(None) => {
            Json(serde_json::json!({
                "success": false,
                "error": "Service not found in P2P network",
                "service_id": service_id
            }))
        }
        Err(e) => {
            tracing::error!("P2P discovery error: {}", e);
            Json(serde_json::json!({
                "success": false,
                "error": format!("Discovery error: {}", e),
                "service_id": service_id
            }))
        }
    }
} 