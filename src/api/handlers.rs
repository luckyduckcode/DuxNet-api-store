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

/// Authentication middleware for API key validation
pub async fn auth_middleware<B>(
    State(state): State<ApiState>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Allow unauthenticated access to public endpoints
    let path = req.uri().path();
    if path == "/api/status" || path == "/" || path == "/index.html" {
        return Ok(next.run(req).await);
    }
    // Check Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(api_key) = auth_str.strip_prefix("Bearer ") {
                if state.api_keys.contains_key(api_key) {
                    return Ok(next.run(req).await);
                }
            }
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

/// Handles GET /api/status
pub async fn get_status(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    // Calculate real uptime and service count
    let reputation = node.reputation_system.get_reputation(&node.did_manager.did.id).await;
    let peers = node.network.get_peers().await;
    let uptime_seconds = node.get_uptime_seconds(); // Assumes this method exists or add TODO
    let services_count = node.services.read().await.len(); // Assumes this field exists or add TODO
    let status = crate::core::data_structures::NodeStatus {
        node_id: node.node_id.0.clone(),
        did: node.did_manager.did.id.clone(),
        is_online: true,
        uptime_seconds,
        services_count,
        reputation_score: reputation,
        peers_count: peers.len(),
    };
    Json(status)
}

// --- Placeholder for authentication middleware to be added next --- 