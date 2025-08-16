//! Reputation model for database operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database model for reputation scores
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbReputationScore {
    pub id: Uuid,
    pub user_id: Uuid,
    pub peer_id: Uuid,
    pub service_id: Option<Uuid>,
    pub score: i32,
    pub review: Option<String>,
    pub transaction_id: Option<Uuid>,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Reputation score creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReputationRequest {
    pub user_id: Uuid,
    pub peer_id: Uuid,
    pub service_id: Option<Uuid>,
    pub score: i32,
    pub review: Option<String>,
    pub transaction_id: Option<Uuid>,
}

/// Reputation score update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReputationRequest {
    pub score: Option<i32>,
    pub review: Option<String>,
    pub is_verified: Option<bool>,
}
