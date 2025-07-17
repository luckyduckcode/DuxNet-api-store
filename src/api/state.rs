//! # API State Module
//!
//! This module defines the shared state structure (`ApiState`) for the DuxNet API.
//!
//! ## Purpose
//! - Encapsulates all shared resources needed by API handlers (e.g., the main DuxNet node).
//! - Passed to handlers via Axum's `State` extractor for safe, concurrent access.
//!
//! ## Best Practices
//! - Only include fields that need to be accessed by multiple handlers (e.g., node, caches, config).
//! - Use `Arc` for thread-safe, shared ownership of stateful resources.
//! - Keep the state struct minimal; delegate complex logic to core modules.
//! - If state grows, consider splitting into sub-structs or using feature-specific state.
//!
//! ## Example
//! ```rust
//! #[derive(Clone)]
//! pub struct ApiState {
//!     pub node: Arc<crate::core::DuxNetNode>,
//! }
//! ```
//!
//! ## Future Improvements
//! - Add support for API keys, authentication, or per-request context as needed.
//! - Use feature flags or generics if different API variants require different state.
//!
//! This structure ensures handlers have access to the resources they need while keeping state management clear and maintainable.
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Clone)]
pub struct ApiState {
    pub node: Arc<crate::core::DuxNetNode>,
    /// API keys: api_key -> user_did mapping (loaded from config or secure storage)
    pub api_keys: Arc<HashMap<String, String>>,
} 