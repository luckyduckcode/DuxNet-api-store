//! Mining model for database operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database model for mining sessions
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbMiningSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub hash_rate: i64,
    pub blocks_mined: i32,
    pub total_rewards: i64,
    pub status: String,
    pub thread_count: i32,
    pub metadata: serde_json::Value,
}

/// Mining session creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMiningSessionRequest {
    pub user_id: Uuid,
    pub thread_count: i32,
    pub metadata: Option<serde_json::Value>,
}

/// Mining session update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMiningSessionRequest {
    pub end_time: Option<DateTime<Utc>>,
    pub hash_rate: Option<i64>,
    pub blocks_mined: Option<i32>,
    pub total_rewards: Option<i64>,
    pub status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
