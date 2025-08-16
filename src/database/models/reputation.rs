//! Reputation model for database operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database model for reputation attestations
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbReputationAttestation {
    pub id: Uuid,
    pub attester_did: String,
    pub target_did: String,
    pub score: f64,
    pub interaction_type: String,
    pub signature_hex: String, // Store signature as hex string
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database model for reputation scores (cached calculations)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbReputationScore {
    pub id: Uuid,
    pub did: String,
    pub current_score: f64,
    pub total_attestations: i32,
    pub positive_attestations: i32,
    pub negative_attestations: i32,
    pub last_calculated: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create reputation attestation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAttestationRequest {
    pub attester_did: String,
    pub target_did: String,
    pub score: f64,
    pub interaction_type: String,
    pub signature_hex: String, // Store signature as hex string
    pub timestamp: Option<DateTime<Utc>>,
}

/// Request to update reputation score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScoreRequest {
    pub did: String,
    pub current_score: f64,
    pub total_attestations: i32,
    pub positive_attestations: i32,
    pub negative_attestations: i32,
}
