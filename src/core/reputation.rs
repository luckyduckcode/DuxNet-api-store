use crate::core::data_structures::*;
use crate::database::{RepositoryManager, models::{CreateAttestationRequest, UpdateScoreRequest}};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Clone)]
pub struct ReputationSystem {
    pub attestations: Arc<RwLock<HashMap<String, Vec<ReputationAttestation>>>>,
    pub scores: Arc<RwLock<HashMap<String, f64>>>,
    pub db_repos: Option<RepositoryManager>, // Optional database integration
}

impl ReputationSystem {
    pub fn new() -> Self {
        ReputationSystem {
            attestations: Arc::new(RwLock::new(HashMap::new())),
            scores: Arc::new(RwLock::new(HashMap::new())),
            db_repos: None,
        }
    }

    /// Create new reputation system with database integration
    pub fn with_database(db_repos: RepositoryManager) -> Self {
        ReputationSystem {
            attestations: Arc::new(RwLock::new(HashMap::new())),
            scores: Arc::new(RwLock::new(HashMap::new())),
            db_repos: Some(db_repos),
        }
    }

    pub async fn add_attestation(&self, attestation: ReputationAttestation) -> Result<()> {
        // Store in database if available
        if let Some(ref db_repos) = self.db_repos {
            let request = CreateAttestationRequest {
                attester_did: attestation.attester_did.clone(),
                target_did: attestation.target_did.clone(),
                score: attestation.score,
                interaction_type: attestation.interaction_type.clone(),
                signature_hex: hex::encode(&attestation.signature),
                timestamp: Some(chrono::DateTime::from_timestamp(attestation.timestamp as i64, 0).unwrap_or_else(chrono::Utc::now)),
            };

            match db_repos.reputation.create_attestation(request).await {
                Ok(db_attestation) => {
                    info!("Stored attestation in database: {} -> {}", 
                        attestation.attester_did, attestation.target_did);
                },
                Err(e) => {
                    warn!("Failed to store attestation in database: {}", e);
                }
            }
        }

        // Store in memory (for backward compatibility and caching)
        let mut attestations = self.attestations.write().await;
        attestations
            .entry(attestation.target_did.clone())
            .or_insert_with(Vec::new)
            .push(attestation.clone());
        
        self.recalculate_score(&attestation.target_did).await;
        debug!("Added reputation attestation for: {}", attestation.target_did);
        Ok(())
    }

    pub async fn get_reputation(&self, did: &str) -> f64 {
        // Try to get from database first
        if let Some(ref db_repos) = self.db_repos {
            match db_repos.reputation.get_score(did).await {
                Ok(Some(db_score)) => {
                    // Update in-memory cache
                    let mut scores = self.scores.write().await;
                    scores.insert(did.to_string(), db_score.current_score);
                    return db_score.current_score;
                },
                Ok(None) => {
                    debug!("No reputation score found in database for: {}", did);
                },
                Err(e) => {
                    warn!("Failed to get reputation score from database: {}", e);
                }
            }
        }

        // Fallback to in-memory
        let scores = self.scores.read().await;
        scores.get(did).copied().unwrap_or(0.0)
    }

    pub async fn get_attestations(&self, did: &str) -> Vec<ReputationAttestation> {
        // Try to get from database first
        if let Some(ref db_repos) = self.db_repos {
            match db_repos.reputation.get_attestations(did).await {
                Ok(db_attestations) => {
                    // Convert database attestations to in-memory format
                    let attestations: Vec<ReputationAttestation> = db_attestations
                        .into_iter()
                        .map(|db_att| {
                            let signature = hex::decode(&db_att.signature_hex).unwrap_or_default();
                            ReputationAttestation {
                                attester_did: db_att.attester_did,
                                target_did: db_att.target_did,
                                score: db_att.score,
                                interaction_type: db_att.interaction_type,
                                timestamp: db_att.timestamp.timestamp() as u64,
                                signature,
                            }
                        })
                        .collect();

                    // Update in-memory cache
                    let mut mem_attestations = self.attestations.write().await;
                    mem_attestations.insert(did.to_string(), attestations.clone());
                    return attestations;
                },
                Err(e) => {
                    warn!("Failed to get attestations from database: {}", e);
                }
            }
        }

        // Fallback to in-memory
        let attestations = self.attestations.read().await;
        attestations.get(did).cloned().unwrap_or_default()
    }

    async fn recalculate_score(&self, did: &str) {
        let attestations = self.attestations.read().await;
        if let Some(atts) = attestations.get(did) {
            let now = get_current_timestamp();
            
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;
            let mut positive_count = 0;
            let mut negative_count = 0;
            
            for att in atts {
                // Apply time decay (older attestations have less weight)
                let age_days = (now - att.timestamp) / 86400;
                let decay_factor = 0.95_f64.powi(age_days as i32);
                let weight = decay_factor;
                
                weighted_sum += att.score * weight;
                weight_sum += weight;

                // Count positive and negative attestations
                if att.score > 0.0 {
                    positive_count += 1;
                } else if att.score < 0.0 {
                    negative_count += 1;
                }
            }
            
            let score = if weight_sum > 0.0 {
                weighted_sum / weight_sum
            } else {
                0.0
            };

            // Update in-memory cache
            let mut scores = self.scores.write().await;
            scores.insert(did.to_string(), score);

            // Store in database if available
            if let Some(ref db_repos) = self.db_repos {
                let score_request = UpdateScoreRequest {
                    did: did.to_string(),
                    current_score: score,
                    total_attestations: atts.len() as i32,
                    positive_attestations: positive_count,
                    negative_attestations: negative_count,
                };

                match db_repos.reputation.upsert_score(score_request).await {
                    Ok(_) => {
                        info!("Updated reputation score in database for {}: {}", did, score);
                    },
                    Err(e) => {
                        warn!("Failed to update reputation score in database: {}", e);
                    }
                }
            }
            
            debug!("Recalculated reputation score for {}: {}", did, score);
        }
    }

    pub async fn remove_attestation(&self, target_did: &str, attester_did: &str, timestamp: u64) -> Result<()> {
        let mut attestations = self.attestations.write().await;
        if let Some(atts) = attestations.get_mut(target_did) {
            atts.retain(|att| {
                !(att.attester_did == attester_did && att.timestamp == timestamp)
            });
        }
        
        self.recalculate_score(target_did).await;
        debug!("Removed attestation for: {}", target_did);
        Ok(())
    }

    pub async fn get_top_nodes(&self, limit: usize) -> Vec<(String, f64)> {
        let scores = self.scores.read().await;
        let mut sorted_scores: Vec<(String, f64)> = scores
            .iter()
            .map(|(did, score)| (did.clone(), *score))
            .collect();
        
        sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sorted_scores.truncate(limit);
        
        sorted_scores
    }

    pub async fn get_stats(&self) -> ReputationStats {
        // Try to get stats from database first for more accurate counts
        if let Some(ref db_repos) = self.db_repos {
            // Get counts from database individually
            let attestations_result = db_repos.reputation.count_attestations().await;
            let scores_result = db_repos.reputation.count_scores().await;
            
            if let (Ok(total_attestations), Ok(total_nodes)) = (attestations_result, scores_result) {
                // Get average score from top scores in database
                match db_repos.reputation.get_top_scores(1000).await {
                    Ok(scores) => {
                        let avg_score = if scores.is_empty() {
                            0.0
                        } else {
                            scores.iter().map(|s| s.current_score).sum::<f64>() / scores.len() as f64
                        };
                        
                        return ReputationStats {
                            total_nodes: total_nodes as usize,
                            total_attestations: total_attestations as usize,
                            average_score: avg_score,
                        };
                    },
                    Err(e) => warn!("Failed to get scores for stats: {}", e),
                }
            } else {
                warn!("Failed to get database stats");
            }
        }

        // Fallback to in-memory stats
        let attestations = self.attestations.read().await;
        let scores = self.scores.read().await;
        
        let total_attestations: usize = attestations.values().map(|v| v.len()).sum();
        let avg_score: f64 = if scores.is_empty() {
            0.0
        } else {
            scores.values().sum::<f64>() / scores.len() as f64
        };
        
        ReputationStats {
            total_nodes: scores.len(),
            total_attestations,
            average_score: avg_score,
        }
    }

    // ===== DATABASE OPERATIONS =====

    /// Get top nodes from database
    pub async fn get_top_nodes_from_db(&self, limit: i32) -> Result<Vec<(String, f64)>> {
        if let Some(ref db_repos) = self.db_repos {
            let scores = db_repos.reputation.get_top_scores(limit).await?;
            Ok(scores.into_iter().map(|score| (score.did, score.current_score)).collect())
        } else {
            warn!("Database repository not available");
            Ok(vec![])
        }
    }

    /// Get recent attestations from database
    pub async fn get_recent_attestations_from_db(&self, did: &str, hours: i32) -> Result<Vec<ReputationAttestation>> {
        if let Some(ref db_repos) = self.db_repos {
            let db_attestations = db_repos.reputation.get_recent_attestations(did, hours).await?;
            
            let attestations: Vec<ReputationAttestation> = db_attestations
                .into_iter()
                .map(|db_att| {
                    ReputationAttestation {
                        attester_did: db_att.attester_did,
                        target_did: db_att.target_did,
                        score: db_att.score,
                        interaction_type: db_att.interaction_type,
                        timestamp: db_att.timestamp.timestamp() as u64,
                        signature: hex::decode(&db_att.signature_hex).unwrap_or_default(),
                    }
                })
                .collect();
                
            Ok(attestations)
        } else {
            warn!("Database repository not available");
            Ok(vec![])
        }
    }

    /// Get attestations by attester from database
    pub async fn get_attestations_by_attester_from_db(&self, attester_did: &str) -> Result<Vec<ReputationAttestation>> {
        if let Some(ref db_repos) = self.db_repos {
            let db_attestations = db_repos.reputation.get_attestations_by_attester(attester_did).await?;
            
            let attestations: Vec<ReputationAttestation> = db_attestations
                .into_iter()
                .map(|db_att| {
                    ReputationAttestation {
                        attester_did: db_att.attester_did,
                        target_did: db_att.target_did,
                        score: db_att.score,
                        interaction_type: db_att.interaction_type,
                        timestamp: db_att.timestamp.timestamp() as u64,
                        signature: hex::decode(&db_att.signature_hex).unwrap_or_default(),
                    }
                })
                .collect();
                
            Ok(attestations)
        } else {
            warn!("Database repository not available");
            Ok(vec![])
        }
    }

    /// Recalculate scores for stale entries
    pub async fn recalculate_stale_scores(&self, hours: i32) -> Result<usize> {
        if let Some(ref db_repos) = self.db_repos {
            let stale_scores = db_repos.reputation.get_stale_scores(hours).await?;
            let count = stale_scores.len();
            
            for score in stale_scores {
                // Load attestations and recalculate
                let attestations = self.get_attestations(&score.did).await;
                if !attestations.is_empty() {
                    // Update in-memory cache with fresh attestations
                    let mut mem_attestations = self.attestations.write().await;
                    mem_attestations.insert(score.did.clone(), attestations);
                    drop(mem_attestations);
                    
                    // Recalculate and store
                    self.recalculate_score(&score.did).await;
                }
            }
            
            info!("Recalculated {} stale reputation scores", count);
            Ok(count)
        } else {
            warn!("Database repository not available");
            Ok(0)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReputationStats {
    pub total_nodes: usize,
    pub total_attestations: usize,
    pub average_score: f64,
} 