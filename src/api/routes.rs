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
use axum::{routing::{get, post}, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use crate::api::handlers::*;
use crate::api::state::ApiState;
use axum::http::Method;
use axum::middleware;
use crate::api::handlers::auth_middleware;

pub fn create_router(state: ApiState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    Router::new()
        .route("/api/status", get(get_status))
        .route("/api/services/register", post(register_service))
        .route("/api/services/search", post(search_services))
        .route("/api/tasks/submit", post(submit_task))
        .route("/api/escrow/create", post(create_escrow))
        .route("/api/reputation/:did", get(get_reputation))
        .route("/api/stats", get(get_stats))
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
        .route("/api/services/aoi/register", post(register_aoi_key))
        .route("/api/services/aoi/get", post(get_aoi_key))
        .route("/api/community_fund/stats", get(get_community_fund_stats))
        .route("/api/community_fund/balance/:currency", get(get_community_fund_balance))
        .route("/api/community_fund/distribute/:currency", post(distribute_community_fund))
        .route("/api/messaging/send", post(send_message))
        .route("/api/messaging/conversations", get(get_conversations))
        .route("/api/messaging/messages/:peer_did", get(get_messages))
        .route("/api/messaging/read/:message_id", post(mark_message_read))
        .route("/api/messaging/delete/:message_id", post(delete_message))
        .route("/api/messaging/stats", get(get_messaging_stats))
        .route("/api/dux/balance", get(get_dux_balance))
        .route("/api/dux/transactions", get(get_dux_transactions))
        .route("/api/dux/send", post(send_dux))
        .route("/api/dux/network", get(get_dux_network))
        .route("/api/dux/mine/start", post(start_dux_mining))
        .route("/api/dux/mine/stop", post(stop_dux_mining))
        .route("/api/dux/mine/status", get(get_dux_mining_status))
        .route("/api/dux/sync", post(sync_dux_balance))
        .route("/api/shutdown", post(shutdown_node))
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        .nest_service("/static", ServeDir::new("static"))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .layer(cors)
        .with_state(state)
} 