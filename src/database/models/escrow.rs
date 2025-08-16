//! Escrow model for database operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database model for escrow contracts
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbEscrowContract {
    pub id: Uuid,
    pub contract_id: String,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub service_id: Option<Uuid>,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub terms: serde_json::Value,
    pub signatures: serde_json::Value,
    pub dispute_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Escrow contract creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEscrowContractRequest {
    pub contract_id: String,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub service_id: Option<Uuid>,
    pub amount: i64,
    pub currency: String,
    pub terms: serde_json::Value,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Escrow contract update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEscrowContractRequest {
    pub status: Option<String>,
    pub signatures: Option<serde_json::Value>,
    pub dispute_data: Option<serde_json::Value>,
    pub completed_at: Option<DateTime<Utc>>,
}
