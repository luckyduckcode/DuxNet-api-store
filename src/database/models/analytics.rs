//! Analytics model for database operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database model for analytics events
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbAnalyticsEvent {
    pub id: Uuid,
    pub event_type: String,
    pub user_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub event_data: serde_json::Value,
    pub metadata: serde_json::Value,
    pub ip_address: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Analytics event creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAnalyticsEventRequest {
    pub event_type: String,
    pub user_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub event_data: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
}
