//! Reputation repository for database operations

use crate::database::models::{DbReputationAttestation, DbReputationScore, CreateAttestationRequest, UpdateScoreRequest};
use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Repository for reputation operations
#[derive(Clone)]
pub struct ReputationRepository {
    pool: Arc<PgPool>,
}

impl ReputationRepository {
    /// Create new reputation repository
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    // ===== ATTESTATION OPERATIONS =====

    /// Create new reputation attestation
    pub async fn create_attestation(&self, request: CreateAttestationRequest) -> Result<DbReputationAttestation> {
        let now = Utc::now();
        let attestation_id = Uuid::new_v4();
        let timestamp = request.timestamp.unwrap_or(now);
        
        let query = "INSERT INTO reputation_attestations (id, attester_did, target_did, score, interaction_type, signature_hex, timestamp, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *";
        
        let attestation = sqlx::query_as::<_, DbReputationAttestation>(query)
            .bind(attestation_id)
            .bind(&request.attester_did)
            .bind(&request.target_did)
            .bind(request.score)
            .bind(&request.interaction_type)
            .bind(&request.signature_hex)
            .bind(timestamp)
            .bind(now)
            .bind(now)
            .fetch_one(&*self.pool)
            .await?;
            
        Ok(attestation)
    }

    /// Get attestations for a DID
    pub async fn get_attestations(&self, did: &str) -> Result<Vec<DbReputationAttestation>> {
        let query = "SELECT * FROM reputation_attestations WHERE target_did = $1 ORDER BY timestamp DESC";
        
        let attestations = sqlx::query_as::<_, DbReputationAttestation>(query)
            .bind(did)
            .fetch_all(&*self.pool)
            .await
            .context("Failed to get attestations")?;
            
        Ok(attestations)
    }

    /// Get attestations by attester
    pub async fn get_attestations_by_attester(&self, attester_did: &str) -> Result<Vec<DbReputationAttestation>> {
        let query = "SELECT * FROM reputation_attestations WHERE attester_did = $1 ORDER BY timestamp DESC";
        
        let attestations = sqlx::query_as::<_, DbReputationAttestation>(query)
            .bind(attester_did)
            .fetch_all(&*self.pool)
            .await
            .context("Failed to get attestations by attester")?;
            
        Ok(attestations)
    }

    /// Get recent attestations with time window
    pub async fn get_recent_attestations(&self, did: &str, hours: i32) -> Result<Vec<DbReputationAttestation>> {
        let query = "SELECT * FROM reputation_attestations WHERE target_did = $1 AND timestamp > NOW() - INTERVAL '$2 hours' ORDER BY timestamp DESC";
        
        let attestations = sqlx::query_as::<_, DbReputationAttestation>(query)
            .bind(did)
            .bind(hours)
            .fetch_all(&*self.pool)
            .await
            .context("Failed to get recent attestations")?;
            
        Ok(attestations)
    }

    // ===== SCORE OPERATIONS =====

    /// Create or update reputation score
    pub async fn upsert_score(&self, request: UpdateScoreRequest) -> Result<DbReputationScore> {
        let now = Utc::now();
        let score_id = Uuid::new_v4();
        
        let query = r#"
            INSERT INTO reputation_scores (id, did, current_score, total_attestations, positive_attestations, negative_attestations, last_calculated, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (did) DO UPDATE SET
                current_score = EXCLUDED.current_score,
                total_attestations = EXCLUDED.total_attestations,
                positive_attestations = EXCLUDED.positive_attestations,
                negative_attestations = EXCLUDED.negative_attestations,
                last_calculated = EXCLUDED.last_calculated,
                updated_at = EXCLUDED.updated_at
            RETURNING *
        "#;
        
        let score = sqlx::query_as::<_, DbReputationScore>(query)
            .bind(score_id)
            .bind(&request.did)
            .bind(request.current_score)
            .bind(request.total_attestations)
            .bind(request.positive_attestations)
            .bind(request.negative_attestations)
            .bind(now)
            .bind(now)
            .bind(now)
            .fetch_one(&*self.pool)
            .await?;
            
        Ok(score)
    }

    /// Get reputation score for a DID
    pub async fn get_score(&self, did: &str) -> Result<Option<DbReputationScore>> {
        let query = "SELECT * FROM reputation_scores WHERE did = $1";
        
        let score = sqlx::query_as::<_, DbReputationScore>(query)
            .bind(did)
            .fetch_optional(&*self.pool)
            .await
            .context("Failed to get reputation score")?;
            
        Ok(score)
    }

    /// Get top reputation scores
    pub async fn get_top_scores(&self, limit: i32) -> Result<Vec<DbReputationScore>> {
        let query = "SELECT * FROM reputation_scores ORDER BY current_score DESC LIMIT $1";
        
        let scores = sqlx::query_as::<_, DbReputationScore>(query)
            .bind(limit)
            .fetch_all(&*self.pool)
            .await
            .context("Failed to get top scores")?;
            
        Ok(scores)
    }

    /// Get scores that need recalculation (older than specified hours)
    pub async fn get_stale_scores(&self, hours: i32) -> Result<Vec<DbReputationScore>> {
        let query = "SELECT * FROM reputation_scores WHERE last_calculated < NOW() - INTERVAL '$1 hours'";
        
        let scores = sqlx::query_as::<_, DbReputationScore>(query)
            .bind(hours)
            .fetch_all(&*self.pool)
            .await
            .context("Failed to get stale scores")?;
            
        Ok(scores)
    }

    // ===== STATISTICS =====

    /// Count total attestations
    pub async fn count_attestations(&self) -> Result<i64> {
        let query = "SELECT COUNT(*) FROM reputation_attestations";
        
        let count: Option<i64> = sqlx::query_scalar(query)
            .fetch_one(&*self.pool)
            .await
            .context("Failed to count attestations")?;
            
        Ok(count.unwrap_or(0))
    }

    /// Count attestations for a DID
    pub async fn count_attestations_for_did(&self, did: &str) -> Result<i64> {
        let query = "SELECT COUNT(*) FROM reputation_attestations WHERE target_did = $1";
        
        let count: Option<i64> = sqlx::query_scalar(query)
            .bind(did)
            .fetch_one(&*self.pool)
            .await
            .context("Failed to count attestations for DID")?;
            
        Ok(count.unwrap_or(0))
    }

    /// Count total reputation scores
    pub async fn count_scores(&self) -> Result<i64> {
        let query = "SELECT COUNT(*) FROM reputation_scores";
        
        let count: Option<i64> = sqlx::query_scalar(query)
            .fetch_one(&*self.pool)
            .await
            .context("Failed to count scores")?;
            
        Ok(count.unwrap_or(0))
    }
}
