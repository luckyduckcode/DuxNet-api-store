//! User model for database operations

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database model for users
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbUser {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub wallet_address: Option<String>,
    pub public_key: Option<String>,
    pub reputation_score: Option<Decimal>,
    pub total_earnings: Option<i64>,
    pub total_spent: Option<i64>,
    pub service_count: Option<i32>,
    pub rating: Option<Decimal>,
    pub metadata: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// User creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub wallet_address: Option<String>,
    pub public_key: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// User update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub wallet_address: Option<String>,
    pub public_key: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

impl DbUser {
    /// Convert to DuxNet User type
    pub fn to_duxnet_user(&self) -> crate::core::data_structures::User {
        crate::core::data_structures::User {
            id: self.id.to_string(),
            username: self.username.clone(),
            display_name: self.display_name.clone(),
            email: self.email.clone(),
            wallet_address: self.wallet_address.clone(),
            public_key: self.public_key.clone(),
            reputation_score: self.reputation_score.map(|r| r.to_string().parse().unwrap_or(0.0)).unwrap_or(0.0),
            total_earnings: self.total_earnings.unwrap_or(0),
            total_spent: self.total_spent.unwrap_or(0),
            service_count: self.service_count.unwrap_or(0),
            rating: self.rating.map(|r| r.to_string().parse().unwrap_or(0.0)).unwrap_or(0.0),
            metadata: self.metadata.clone().unwrap_or(serde_json::json!({})),
            is_active: self.is_active.unwrap_or(true),
            created_at: self.created_at.map(|dt| dt.timestamp()).unwrap_or(0),
        }
    }
}
