//! # Core Module
//!
//! This module contains the foundational logic and business rules for the DuxNet platform.
//! It aggregates and exposes submodules for identity, DHT, reputation, escrow, tasks, messaging, community fund, and shared data structures.
//!
//! ## Purpose
//! - Implements the main node struct (`DuxNetNode`) and orchestrates all core platform features.
//! - Provides abstractions for decentralized identity, distributed storage, reputation, escrow, task engine, and messaging.
//! - Serves as the backbone for all higher-level modules (API, wallet, network, frontend).
//!
//! ## Submodules
//! - `identity`: Digital identity management (DID, keys)
//! - `dht`: Distributed hash table for peer discovery/storage
//! - `reputation`: Reputation system and attestations
//! - `escrow`: Multi-signature escrow contracts
//! - `tasks`: Distributed task engine
//! - `messaging`: Messaging and communication
//! - `community_fund`: Community fund logic
//! - `data_structures`: Shared types and data models
//!
//! ## Best Practices
//! - Keep each submodule focused and well-documented.
//! - Use clear, consistent naming for structs and methods.
//! - Delegate business logic to the appropriate submodule.
//! - Add doc comments to all public types and functions.
//! - If the core grows, consider further splitting into feature-specific submodules.
//!
//! ## Future Improvements
//! - Add more granular error types for better error handling.
//! - Expand test coverage for all core features.
//! - Document cross-module interactions and data flows.
//!
//! This structure ensures the core logic is robust, maintainable, and easy to extend as the platform evolves.
pub mod data_structures;
pub mod dht;
pub mod identity;
pub mod reputation;
pub mod escrow;
pub mod tasks;
pub mod community_fund;
pub mod messaging;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

use data_structures::*;
use dht::DHT;
use identity::DIDManager;
use reputation::ReputationSystem;
use escrow::EscrowManager;
use tasks::TaskEngine;
use community_fund::CommunityFundManager;
use messaging::MessagingSystem;

// Add a constant for the platform escrow DuxCoin address
const ESCROW_DUXCOIN_ADDRESS: &str = "<YOUR_ESCROW_DUXCOIN_ADDRESS_HERE>"; // Replace with your real address

pub struct DuxNetNode {
    pub node_id: NodeId,
    pub did_manager: DIDManager,
    pub dht: DHT,
    pub reputation_system: ReputationSystem,
    pub escrow_manager: EscrowManager,
    pub task_engine: TaskEngine,
    pub community_fund_manager: Arc<CommunityFundManager>,
    pub messaging_system: Arc<MessagingSystem>,
    pub wallet: Arc<RwLock<crate::wallet::Wallet>>,
    pub is_running: Arc<RwLock<bool>>,
}

impl DuxNetNode {
    pub async fn new() -> Result<Self> {
        let node_id = NodeId(uuid::Uuid::new_v4().to_string());
        let endpoints = vec!["http://localhost:8081".to_string()];
        
        let did_manager = DIDManager::new(endpoints);
        let dht = DHT::new(node_id.clone());
        let reputation_system = ReputationSystem::new();
        let escrow_manager = EscrowManager::new();
        let community_fund_manager = Arc::new(CommunityFundManager::new(Arc::new(dht.clone())));
        let task_engine = TaskEngine::new().with_community_fund_manager(community_fund_manager.clone());
        let messaging_system = Arc::new(MessagingSystem::new(did_manager.clone()));
        let wallet = Arc::new(RwLock::new(crate::wallet::Wallet::new(did_manager.did.id.clone())?));
        let is_running = Arc::new(RwLock::new(false));

        Ok(DuxNetNode {
            node_id,
            did_manager,
            dht,
            reputation_system,
            escrow_manager,
            task_engine,
            community_fund_manager,
            messaging_system,
            wallet,
            is_running,
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting DuxNet node: {}", self.node_id.0);
        
        // Mark as running
        {
            let mut running = self.is_running.write().await;
            *running = true;
        }
        
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping DuxNet node: {}", self.node_id.0);
        
        // Mark as stopped
        {
            let mut running = self.is_running.write().await;
            *running = false;
        }
        
        Ok(())
    }

    // Service management
    pub async fn register_service(&self, name: String, description: String, 
                                  price: u64) -> Result<ServiceId> {
        let service_id = ServiceId(uuid::Uuid::new_v4().to_string());
        let service = ServiceMetadata {
            id: service_id.clone(),
            provider_did: self.did_manager.did.id.clone(),
            name,
            description,
            endpoint: self.did_manager.did.endpoints[0].clone(),
            price,
            reputation_score: self.reputation_system.get_reputation(&self.did_manager.did.id).await,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            // Enhanced fields with defaults
            categories: vec!["Other".to_string()],
            tags: vec![],
            sla: ServiceSLA {
                uptime_guarantee: 99.0,
                max_response_time_ms: 5000,
                support_response_hours: 24,
                refund_policy: RefundPolicy::PartialRefund { percentage: 50.0 },
                availability_zones: vec!["global".to_string()],
            },
            version: "1.0.0".to_string(),
            documentation_url: None,
            status: ServiceStatus::Active,
            uptime_percentage: 100.0,
            response_time_ms: 100,
            rate_limit_per_minute: 1000,
            supported_formats: vec!["JSON".to_string()],
            examples: vec![],
        };
        
        self.dht.announce_service(&service).await?;
        info!("Registered service: {}", service_id.0);
        Ok(service_id)
    }

    // Enhanced service registration with full metadata
    pub async fn register_service_enhanced(&self, request: RegisterServiceRequest) -> Result<ServiceId> {
        let service_id = ServiceId(uuid::Uuid::new_v4().to_string());
        let service = ServiceMetadata {
            id: service_id.clone(),
            provider_did: self.did_manager.did.id.clone(),
            name: request.name,
            description: request.description,
            endpoint: self.did_manager.did.endpoints[0].clone(),
            price: request.price,
            reputation_score: self.reputation_system.get_reputation(&self.did_manager.did.id).await,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            categories: request.categories,
            tags: request.tags,
            sla: request.sla,
            version: request.version,
            documentation_url: request.documentation_url,
            status: ServiceStatus::Active,
            uptime_percentage: 100.0,
            response_time_ms: 100,
            rate_limit_per_minute: request.rate_limit_per_minute,
            supported_formats: request.supported_formats,
            examples: request.examples,
        };
        
        self.dht.announce_service(&service).await?;
        info!("Registered enhanced service: {} with {} categories", service_id.0, service.categories.len());
        Ok(service_id)
    }

    pub async fn find_services(&self, query: &str) -> Vec<ServiceMetadata> {
        self.dht.find_services(query).await
    }

    // Enhanced service search with filters and sorting
    pub async fn find_services_enhanced(&self, request: &FindServicesRequest) -> Vec<ServiceMetadata> {
        let mut services = self.dht.find_services(&request.query).await;
        
        // Apply filters
        if let Some(categories) = &request.categories {
            services.retain(|service| {
                service.categories.iter().any(|cat| categories.contains(cat))
            });
        }
        
        if let Some(min_rating) = request.min_rating {
            services.retain(|service| service.reputation_score >= min_rating);
        }
        
        if let Some(max_price) = request.max_price {
            services.retain(|service| service.price <= max_price);
        }
        
        if let Some(status) = &request.status {
            services.retain(|service| std::mem::discriminant(&service.status) == std::mem::discriminant(status));
        }
        
        // Apply sorting
        if let Some(sort_by) = &request.sort_by {
            match sort_by {
                ServiceSortBy::Name => services.sort_by(|a, b| a.name.cmp(&b.name)),
                ServiceSortBy::Price => services.sort_by(|a, b| a.price.cmp(&b.price)),
                ServiceSortBy::Rating => services.sort_by(|a, b| b.reputation_score.partial_cmp(&a.reputation_score).unwrap()),
                ServiceSortBy::Uptime => services.sort_by(|a, b| b.uptime_percentage.partial_cmp(&a.uptime_percentage).unwrap()),
                ServiceSortBy::ResponseTime => services.sort_by(|a, b| a.response_time_ms.cmp(&b.response_time_ms)),
                ServiceSortBy::Popularity => services.sort_by(|a, b| b.reputation_score.partial_cmp(&a.reputation_score).unwrap()),
                ServiceSortBy::Newest => services.sort_by(|a, b| b.last_updated.cmp(&a.last_updated)),
            }
        }
        
        services
    }

    // Get detailed service information
    pub async fn get_service_details(&self, service_id: &str) -> Result<ServiceMetadata> {
        let services = self.dht.find_services(service_id).await;
        services.into_iter()
            .find(|service| service.id.0 == service_id)
            .ok_or_else(|| anyhow::anyhow!("Service not found: {}", service_id))
    }

    // Update service information
    pub async fn update_service(&self, service_id: &str, updates: ServiceMetadata) -> Result<()> {
        // Verify ownership
        if updates.provider_did != self.did_manager.did.id {
            return Err(anyhow::anyhow!("Not authorized to update this service"));
        }
        
        // Update the service in DHT
        self.dht.announce_service(&updates).await?;
        info!("Updated service: {}", service_id);
        Ok(())
    }

    // Delete service
    pub async fn delete_service(&self, service_id: &str) -> Result<()> {
        // In a real implementation, you'd mark it as deleted or remove from DHT
        info!("Service deletion requested: {}", service_id);
        Ok(())
    }

    // Escrow management
    pub async fn create_escrow_for_service(&self, service_id: &ServiceId, 
                                           seller_did: String, amount: u64) -> Result<String> {
        let arbiters = vec![
            "did:duxnet:arbiter1".to_string(),
            "did:duxnet:arbiter2".to_string(),
        ];
        
        let escrow_id = self.escrow_manager.create_escrow(
            self.did_manager.did.id.clone(),
            seller_did,
            arbiters,
            amount
        ).await?;
        
        info!("Created escrow: {}", escrow_id);
        Ok(escrow_id)
    }

    /// Submit a task with escrow-based DuxCoin payment
    pub async fn submit_task_with_escrow(&self, service_id: ServiceId, payload: Vec<u8>, requirements: TaskRequirements, buyer_address: String) -> Result<TaskId> {
        // 1. Lookup the service
        let service = self.get_service_details(&service_id.0).await?;
        let price = service.price as f64 / 100_000_000.0; // Convert to DUX
        let provider_address = service.endpoint.clone(); // Or use a dedicated provider DuxCoin address field

        // 2. Send DuxCoin from buyer to escrow address
        // (Assume buyer_address is managed by the platform for now)
        let txid = crate::api::handlers::DUXCOIN_API.send_dux(&buyer_address, ESCROW_DUXCOIN_ADDRESS, price).await?;

        // 3. Create escrow contract (record txid, buyer, provider, amount, etc.)
        // (You may want to expand this with more escrow logic)
        let escrow_id = format!("escrow-{}", uuid::Uuid::new_v4());
        // For now, just log it
        tracing::info!("Created escrow {} for service {}: buyer {} -> escrow {} (txid {})", escrow_id, service_id.0, buyer_address, ESCROW_DUXCOIN_ADDRESS, txid);

        // 4. Create the task, linking to the escrow
        let task_id = TaskId(uuid::Uuid::new_v4().to_string());
        let task = Task {
            id: task_id.clone(),
            escrow_id: escrow_id.clone(),
            service_id,
            payload,
            requirements,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        self.task_engine.submit_task(task).await?;
        tracing::info!("Submitted task {} with escrow {}", task_id.0, escrow_id);
        Ok(task_id)
    }

    /// Release escrow funds to provider on task completion
    pub async fn release_escrow_to_provider(&self, provider_address: String, amount: u64) -> Result<String> {
        let amount_dux = amount as f64 / 100_000_000.0;
        let txid = crate::api::handlers::DUXCOIN_API.send_dux(ESCROW_DUXCOIN_ADDRESS, &provider_address, amount_dux).await?;
        tracing::info!("Released {} DUX from escrow to provider {} (txid {})", amount_dux, provider_address, txid);
        Ok(txid)
    }

    // Reputation management
    pub async fn get_reputation(&self, did: &str) -> f64 {
        self.reputation_system.get_reputation(did).await
    }

    pub async fn add_reputation_attestation(&self, attestation: ReputationAttestation) -> Result<()> {
        self.reputation_system.add_attestation(attestation).await?;
        Ok(())
    }

    pub async fn register_aoi_key_for_service(&self, service_id: ServiceId, key_data: String) -> Result<()> {
        let aoi_key = AOIKey {
            service_id,
            key_data,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        self.dht.store_aoi_key(&aoi_key).await
    }

    pub async fn get_aoi_key_for_service(&self, service_id: ServiceId) -> Option<AOIKey> {
        self.dht.get_aoi_key(&service_id).await
    }

    // Community Fund Management
    pub async fn get_community_fund_stats(&self) -> Result<CommunityFundStats> {
        self.community_fund_manager.get_stats().await
    }

    pub async fn get_community_fund_balance(&self, currency: &crate::wallet::Currency) -> u64 {
        self.community_fund_manager.get_fund_balance(currency).await
    }

    pub async fn distribute_community_fund(&self, currency: crate::wallet::Currency) -> Result<CommunityFundDistribution> {
        self.community_fund_manager.distribute_fund(currency).await
    }

    pub async fn add_tax_to_community_fund(&self, currency: crate::wallet::Currency, tax_amount: u64) -> Result<()> {
        self.community_fund_manager.add_tax_to_fund(currency, tax_amount).await
    }
} 