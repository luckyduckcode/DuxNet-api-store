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
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ApiUsage {
    pub api_key: String,
    pub endpoint: String,
    pub timestamp: u64,
    pub response_time: u64,
    pub status_code: u16,
    pub user_agent: String,
    pub ip_address: String,
}

#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub requests_count: u64,
    pub window_start: u64,
    pub limit: u64,
    pub window_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct ServiceAnalytics {
    pub service_id: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub total_revenue: u64,
    pub last_updated: u64,
}

#[derive(Clone)]
pub struct ApiState {
    pub node: Arc<crate::core::DuxNetNode>,
    /// API keys: api_key -> user_did mapping (loaded from config or secure storage)
    pub api_keys: Arc<HashMap<String, String>>,
    /// API usage tracking for analytics and billing
    pub usage_tracker: Arc<RwLock<Vec<ApiUsage>>>,
    /// Rate limiting per API key
    pub rate_limits: Arc<RwLock<HashMap<String, RateLimitInfo>>>,
    /// Service analytics and performance metrics
    pub service_analytics: Arc<RwLock<HashMap<String, ServiceAnalytics>>>,
    /// API version configuration
    pub api_version: String,
    /// Rate limiting configuration
    pub default_rate_limit: u64,
    pub default_window_seconds: u64,
}

impl ApiState {
    pub fn new(node: Arc<crate::core::DuxNetNode>, api_keys: Arc<HashMap<String, String>>) -> Self {
        Self {
            node,
            api_keys,
            usage_tracker: Arc::new(RwLock::new(Vec::new())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            service_analytics: Arc::new(RwLock::new(HashMap::new())),
            api_version: "v1".to_string(),
            default_rate_limit: 1000, // 1000 requests per window
            default_window_seconds: 3600, // 1 hour window
        }
    }

    pub async fn track_usage(&self, usage: ApiUsage) {
        let mut tracker = self.usage_tracker.write().await;
        tracker.push(usage);
        
        // Keep only last 10,000 usage records to prevent memory bloat
        if tracker.len() > 10000 {
            tracker.drain(0..1000);
        }
    }

    pub async fn check_rate_limit(&self, api_key: &str) -> Result<bool, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut rate_limits = self.rate_limits.write().await;
        
        if let Some(limit_info) = rate_limits.get_mut(api_key) {
            // Check if window has expired
            if now - limit_info.window_start >= limit_info.window_seconds {
                // Reset window
                limit_info.requests_count = 1;
                limit_info.window_start = now;
                return Ok(true);
            }
            
            // Check if limit exceeded
            if limit_info.requests_count >= limit_info.limit {
                return Ok(false);
            }
            
            limit_info.requests_count += 1;
        } else {
            // Create new rate limit entry
            rate_limits.insert(api_key.to_string(), RateLimitInfo {
                requests_count: 1,
                window_start: now,
                limit: self.default_rate_limit,
                window_seconds: self.default_window_seconds,
            });
        }
        
        Ok(true)
    }

    pub async fn update_service_analytics(&self, service_id: &str, response_time: u64, success: bool, revenue: u64) {
        let mut analytics = self.service_analytics.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if let Some(service_stats) = analytics.get_mut(service_id) {
            service_stats.total_requests += 1;
            if success {
                service_stats.successful_requests += 1;
            } else {
                service_stats.failed_requests += 1;
            }
            
            // Update average response time
            let total_time = service_stats.average_response_time * (service_stats.total_requests - 1) as f64 + response_time as f64;
            service_stats.average_response_time = total_time / service_stats.total_requests as f64;
            
            service_stats.total_revenue += revenue;
            service_stats.last_updated = now;
        } else {
            analytics.insert(service_id.to_string(), ServiceAnalytics {
                service_id: service_id.to_string(),
                total_requests: 1,
                successful_requests: if success { 1 } else { 0 },
                failed_requests: if success { 0 } else { 1 },
                average_response_time: response_time as f64,
                total_revenue: revenue,
                last_updated: now,
            });
        }
    }

    pub async fn get_usage_stats(&self, api_key: Option<&str>, hours: u64) -> Vec<ApiUsage> {
        let tracker = self.usage_tracker.read().await;
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - (hours * 3600);
        
        tracker
            .iter()
            .filter(|usage| {
                usage.timestamp >= cutoff_time && 
                api_key.map_or(true, |key| usage.api_key == key)
            })
            .cloned()
            .collect()
    }
} 