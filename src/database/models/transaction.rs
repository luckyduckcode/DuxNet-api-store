//! Transaction model for database operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database model for transactions
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbTransaction {
    pub id: Uuid,
    pub transaction_hash: String,
    pub from_user_id: Option<Uuid>,
    pub to_user_id: Option<Uuid>,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub amount: i64,
    pub currency: String,
    pub transaction_type: String,
    pub status: String,
    pub confirmations: i32,
    pub block_hash: Option<String>,
    pub block_height: Option<i64>,
    pub fee: i64,
    pub gas_used: i64,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

/// Transaction creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransactionRequest {
    pub transaction_hash: String,
    pub from_user_id: Option<Uuid>,
    pub to_user_id: Option<Uuid>,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub amount: i64,
    pub currency: String,
    pub transaction_type: String,
    pub fee: Option<i64>,
    pub gas_used: Option<i64>,
    pub data: Option<serde_json::Value>,
}

/// Transaction update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTransactionRequest {
    pub status: Option<String>,
    pub confirmations: Option<i32>,
    pub block_hash: Option<String>,
    pub block_height: Option<i64>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

/// Transaction status enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Confirmed => write!(f, "confirmed"),
            TransactionStatus::Failed => write!(f, "failed"),
            TransactionStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Transaction type enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    Mining,
    ServicePayment,
    Escrow,
    Fee,
    Reward,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Transfer => write!(f, "transfer"),
            TransactionType::Mining => write!(f, "mining"),
            TransactionType::ServicePayment => write!(f, "service_payment"),
            TransactionType::Escrow => write!(f, "escrow"),
            TransactionType::Fee => write!(f, "fee"),
            TransactionType::Reward => write!(f, "reward"),
        }
    }
}

impl DbTransaction {
    /// Convert to DuxNet Transaction type
    pub fn to_duxnet_transaction(&self) -> crate::wallet::Transaction {
        crate::wallet::Transaction {
            id: self.id.to_string(),
            from: self.from_address.clone().unwrap_or_default(),
            to: self.to_address.clone().unwrap_or_default(),
            amount: self.amount as u64,
            currency: self.currency.parse().unwrap_or(crate::wallet::Currency::DUX),
            timestamp: self.created_at.timestamp() as u64,
            status: match self.status.as_str() {
                "confirmed" => crate::wallet::TransactionStatus::Confirmed,
                "failed" => crate::wallet::TransactionStatus::Failed,
                _ => crate::wallet::TransactionStatus::Pending,
            },
            signature: vec![], // Will be populated from blockchain
            fee: self.fee as u64,
            block_height: self.block_height.map(|h| h as u64),
            confirmations: self.confirmations as u32,
            memo: None, // Can be added to data field later
        }
    }
}
