//! # API Routes Module
//!
//! This module defines the HTTP route structure for the DuxNet API using Axum's router.
//!
//! ## Purpose
//! - Centralizes all route definitions and their mapping to handler functions.
//! - Sets up middleware (e.g., CORS, static file serving) and attaches shared state.
//! - Provides a single entry point (`create_router`) for constructing the API router.
//!
//! ## Best Practices
//! - Import all handler functions from `handlers.rs` and state from `state.rs`.
//! - Keep route definitions clear and grouped by feature (e.g., wallet, messaging, community fund).
//! - Use descriptive, RESTful route paths and HTTP methods.
//! - Add comments to group related routes for easier navigation.
//! - If the number of routes grows very large, consider splitting into sub-routers by feature.
//!
//! ## Example
//! ```rust
//! pub fn create_router(state: ApiState) -> Router { ... }
//! ```
//!
//! ## Future Improvements
//! - Add versioning support (e.g., `/api/v1/` prefix).
//! - Use Axum layers for authentication, logging, or rate limiting as needed.
//! - Consider extracting sub-routers for large features (e.g., wallet, messaging).
//!
//! This structure makes it easy to extend the API with new endpoints and middleware in a maintainable way.
use axum::{routing::{get, post, put, delete}, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use crate::api::handlers::*;
use crate::api::state::ApiState;
use axum::http::Method;

pub fn create_router(state: ApiState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any);

    Router::new()
        // Core API endpoints
        .route("/api/status", get(get_status))
        .route("/api/stats", get(get_stats))
        .route("/api/version", get(get_api_version))
        
        // Service Management (Enhanced)
        .route("/api/services/register", post(register_service))
        .route("/api/services/search", post(search_services))
        .route("/api/services/:service_id", get(get_service_details))
        .route("/api/services/:service_id", put(update_service))
        .route("/api/services/:service_id", delete(delete_service))
        .route("/api/services/:service_id/analytics", get(get_service_analytics))
        .route("/api/services/:service_id/health", get(check_service_health))
        .route("/api/services/categories", get(get_service_categories))
        .route("/api/services/trending", get(get_trending_services))
        
        // Service Reviews & Ratings
        .route("/api/services/:service_id/reviews", get(get_service_reviews))
        .route("/api/services/:service_id/reviews", post(add_service_review))
        .route("/api/reviews/:review_id", put(update_service_review))
        .route("/api/reviews/:review_id", delete(delete_service_review))
        .route("/api/reviews/:review_id/vote", post(vote_on_review))
        
        // Task Management (Enhanced)
        .route("/api/tasks/submit", post(submit_task))
        .route("/api/tasks/:task_id", get(get_task_status))
        .route("/api/tasks/:task_id", delete(cancel_task))
        .route("/api/tasks", get(get_user_tasks))
        
        // Escrow & Payments
        .route("/api/escrow/create", post(create_escrow))
        .route("/api/escrow/:escrow_id", get(get_escrow_details))
        .route("/api/escrow/:escrow_id/sign", post(sign_escrow))
        
        // Reputation System
        .route("/api/reputation/:did", get(get_reputation))
        .route("/api/reputation/attest", post(add_reputation_attestation))
        
        // Wallet Operations
        .route("/api/wallet/info", get(get_wallet_info))
        .route("/api/wallet/balances", get(get_wallet_balances))
        .route("/api/wallet/addresses", get(get_wallet_addresses))
        .route("/api/wallet/send", post(send_funds))
        .route("/api/wallet/receive", post(receive_funds))
        .route("/api/wallet/transactions", get(get_transaction_history))
        .route("/api/wallet/transaction/:id", get(get_transaction_by_id))
        .route("/api/wallet/backup", get(backup_wallet))
        .route("/api/wallet/restore", post(restore_wallet))
        .route("/api/wallet/keys", get(get_wallet_keys))
        
        // DUX Coin Integration
        .route("/api/dux/balance", get(get_dux_balance))
        .route("/api/dux/transactions", get(get_dux_transactions))
        .route("/api/dux/send", post(send_dux))
        .route("/api/dux/network", get(get_dux_network))
        .route("/api/dux/mine/start", post(start_dux_mining))
        .route("/api/dux/mine/stop", post(stop_dux_mining))
        .route("/api/dux/mine/status", get(get_dux_mining_status))
        .route("/api/dux/sync", post(sync_dux_balance))
        
        // API Management & Analytics
        .route("/api/analytics/usage", get(get_usage_analytics))
        .route("/api/analytics/usage/:api_key", get(get_api_key_usage))
        .route("/api/analytics/services", get(get_services_analytics))
        .route("/api/analytics/revenue", get(get_revenue_analytics))
        .route("/api/rate-limits/:api_key", get(get_rate_limit_info))
        .route("/api/rate-limits/:api_key", put(update_rate_limit))
        
        // Developer Portal
        .route("/api/developer/keys", get(get_api_keys))
        .route("/api/developer/keys", post(generate_api_key))
        .route("/api/developer/keys/:key_id", delete(revoke_api_key))
        .route("/api/developer/dashboard", get(get_developer_dashboard))
        .route("/api/developer/billing", get(get_billing_info))
        
        // Service Discovery & AOI Keys
        .route("/api/services/aoi/register", post(register_aoi_key))
        .route("/api/services/aoi/get", post(get_aoi_key))
        
        // Community Fund
        .route("/api/community_fund/stats", get(get_community_fund_stats))
        .route("/api/community_fund/balance/:currency", get(get_community_fund_balance))
        .route("/api/community_fund/distribute/:currency", post(distribute_community_fund))
        
        // Messaging
        .route("/api/messaging/send", post(send_message))
        .route("/api/messaging/conversations", get(get_conversations))
        .route("/api/messaging/messages/:peer_did", get(get_messages))
        .route("/api/messaging/read/:message_id", post(mark_message_read))
        .route("/api/messaging/delete/:message_id", post(delete_message))
        .route("/api/messaging/stats", get(get_messaging_stats))
        
        // System Management
        .route("/api/shutdown", post(shutdown_node))
        
        // Web Interface
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        .nest_service("/static", ServeDir::new("static"))
        .layer(cors)
        .with_state(state)
} 