//! Wallet model for database operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database model for wallet balances
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbWalletBalance {
    pub id: Uuid,
    pub user_id: Uuid,
    pub currency: String,
    pub balance: i64,
    pub locked_balance: i64,
    pub address: Option<String>,
    pub last_updated: DateTime<Utc>,
}

/// Wallet balance creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWalletBalanceRequest {
    pub user_id: Uuid,
    pub currency: String,
    pub balance: i64,
    pub locked_balance: Option<i64>,
    pub address: Option<String>,
}

/// Wallet balance update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWalletBalanceRequest {
    pub balance: Option<i64>,
    pub locked_balance: Option<i64>,
    pub address: Option<String>,
}
