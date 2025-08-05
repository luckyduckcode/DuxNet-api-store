use crate::core::data_structures::*;
use crate::core::data_structures::{ServiceManifest, SearchFilters};
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct DHTEntry {
    pub key: String,
    pub value: Vec<u8>,
    pub timestamp: u64,
    pub ttl: u64,
}

#[derive(Debug, Clone)]
pub struct DHT {
    pub node_id: NodeId,
    pub entries: Arc<RwLock<HashMap<String, DHTEntry>>>,
    pub peers: Arc<RwLock<Vec<String>>>,
    pub k_bucket_size: usize,
}

impl DHT {
    pub fn new(node_id: NodeId) -> Self {
        DHT {
            node_id,
            entries: Arc::new(RwLock::new(HashMap::new())),
            peers: Arc::new(RwLock::new(Vec::new())),
            k_bucket_size: 20,
        }
    }

    pub async fn store(&self, key: String, value: Vec<u8>, ttl: u64) -> Result<()> {
        let entry = DHTEntry {
            key: key.clone(),
            value,
            ttl,
            timestamp: get_current_timestamp(),
        };
        
        let mut store = self.entries.write().await;
        store.insert(key.clone(), entry);
        debug!("Stored DHT entry: {}", key);
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let entries = self.entries.read().await;
        let now = get_current_timestamp();
        
        if let Some(entry) = entries.get(key) {
            if now < entry.timestamp + entry.ttl {
                debug!("Retrieved DHT entry: {}", key);
                return Some(entry.value.clone());
            } else {
                debug!("DHT entry expired: {}", key);
            }
        }
        None
    }

    pub async fn remove(&self, key: &str) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.remove(key);
        debug!("Removed DHT entry: {}", key);
        Ok(())
    }

    pub async fn announce_service(&self, service: &ServiceMetadata) -> Result<()> {
        let key = format!("service:{}", service.id.0);
        let value = serde_json::to_vec(service)?;
        self.store(key, value, 3600).await // 1 hour TTL
    }

    pub async fn find_services(&self, query: &str) -> Vec<ServiceMetadata> {
        let entries = self.entries.read().await;
        let mut services = Vec::new();
        
        for (key, entry) in entries.iter() {
            if key.starts_with("service:") {
                if let Ok(service) = serde_json::from_slice::<ServiceMetadata>(&entry.value) {
                    if service.name.to_lowercase().contains(&query.to_lowercase()) || 
                       service.description.to_lowercase().contains(&query.to_lowercase()) {
                        services.push(service);
                    }
                }
            }
        }
        
        debug!("Found {} services for query: {}", services.len(), query);
        services
    }

    pub async fn store_reputation_attestation(&self, attestation: &ReputationAttestation) -> Result<()> {
        let key = format!("reputation:{}:{}", attestation.target_did, attestation.timestamp);
        let value = serde_json::to_vec(attestation)?;
        self.store(key, value, 86400).await // 24 hour TTL
    }

    pub async fn get_reputation_attestations(&self, target_did: &str) -> Vec<ReputationAttestation> {
        let entries = self.entries.read().await;
        let mut attestations = Vec::new();
        
        for (key, entry) in entries.iter() {
            if key.starts_with(&format!("reputation:{}:", target_did)) {
                if let Ok(attestation) = serde_json::from_slice::<ReputationAttestation>(&entry.value) {
                    attestations.push(attestation);
                }
            }
        }
        
        debug!("Found {} reputation attestations for: {}", attestations.len(), target_did);
        attestations
    }

    pub async fn store_escrow_contract(&self, contract: &EscrowContract) -> Result<()> {
        let key = format!("escrow:{}", contract.id);
        let value = serde_json::to_vec(contract)?;
        self.store(key, value, 7200).await // 2 hour TTL
    }

    pub async fn get_escrow_contract(&self, escrow_id: &str) -> Option<EscrowContract> {
        let key = format!("escrow:{}", escrow_id);
        if let Some(value) = self.get(&key).await {
            serde_json::from_slice(&value).ok()
        } else {
            None
        }
    }

    pub async fn store_aoi_key(&self, aoi_key: &AOIKey) -> Result<()> {
        let key = format!("aoi:{}", aoi_key.service_id.0);
        let value = serde_json::to_vec(aoi_key)?;
        self.store(key, value, 3600).await // 1 hour TTL
    }

    pub async fn get_aoi_key(&self, service_id: &ServiceId) -> Option<AOIKey> {
        let key = format!("aoi:{}", service_id.0);
        if let Some(value) = self.get(&key).await {
            serde_json::from_slice(&value).ok()
        } else {
            None
        }
    }

    // Community Fund DHT operations
    pub async fn store_community_fund_transaction(
        &self, 
        tx_id: &str, 
        currency: &crate::wallet::Currency, 
        recipient_did: &str, 
        amount: u64
    ) -> Result<()> {
        let key = format!("cf_tx:{}", tx_id);
        let value = serde_json::json!({
            "tx_id": tx_id,
            "currency": currency.symbol(),
            "recipient_did": recipient_did,
            "amount": amount,
            "timestamp": get_current_timestamp()
        });
        let value_bytes = serde_json::to_vec(&value)?;
        self.store(key, value_bytes, 86400).await?;
        Ok(())
    }

    pub async fn get_community_fund_transaction(&self, tx_id: &str) -> Option<serde_json::Value> {
        let key = format!("cf_tx:{}", tx_id);
        if let Some(value) = self.get(&key).await {
            serde_json::from_slice(&value).ok()
        } else {
            None
        }
    }

    pub async fn store_active_did(&self, did: &str) -> Result<()> {
        let key = format!("active_did:{}", did);
        let value = serde_json::json!({
            "did": did,
            "last_seen": get_current_timestamp(),
            "active": true
        });
        let value_bytes = serde_json::to_vec(&value)?;
        self.store(key, value_bytes, 86400).await?; // 24 hour TTL
        Ok(())
    }

    pub async fn get_active_dids(&self) -> Vec<String> {
        let entries = self.entries.read().await;
        let mut active_dids = Vec::new();
        
        for (key, entry) in entries.iter() {
            if key.starts_with("active_did:") {
                if let Ok(data) = serde_json::from_slice::<serde_json::Value>(&entry.value) {
                    if let Some(did) = data["did"].as_str() {
                        active_dids.push(did.to_string());
                    }
                }
            }
        }
        
        debug!("Found {} active DIDs", active_dids.len());
        active_dids
    }

    pub async fn add_peer(&self, peer_id: String) -> Result<()> {
        let mut peers = self.peers.write().await;
        if !peers.contains(&peer_id) {
            peers.push(peer_id.clone());
            if peers.len() > self.k_bucket_size {
                peers.remove(0); // Remove oldest peer
            }
            debug!("Added peer: {}", peer_id);
        }
        Ok(())
    }

    pub async fn remove_peer(&self, peer_id: &str) -> Result<()> {
        let mut peers = self.peers.write().await;
        peers.retain(|p| p != peer_id);
        debug!("Removed peer: {}", peer_id);
        Ok(())
    }

    pub async fn get_peers(&self) -> Vec<String> {
        let peers = self.peers.read().await;
        peers.clone()
    }

    pub async fn cleanup_expired_entries(&self) -> Result<usize> {
        let mut entries = self.entries.write().await;
        let now = get_current_timestamp();
        let initial_count = entries.len();
        
        entries.retain(|_, entry| now < entry.timestamp + entry.ttl);
        
        let removed_count = initial_count - entries.len();
        if removed_count > 0 {
            debug!("Cleaned up {} expired DHT entries", removed_count);
        }
        
        Ok(removed_count)
    }

    pub async fn get_stats(&self) -> DHTStats {
        let entries = self.entries.read().await;
        let peers = self.peers.read().await;
        
        DHTStats {
            total_entries: entries.len(),
            total_peers: peers.len(),
            service_entries: entries.keys().filter(|k| k.starts_with("service:")).count(),
            reputation_entries: entries.keys().filter(|k| k.starts_with("reputation:")).count(),
            escrow_entries: entries.keys().filter(|k| k.starts_with("escrow:")).count(),
            manifest_entries: entries.keys().filter(|k| k.starts_with("manifest:")).count(),
        }
    }

    // === YAML MANIFEST DHT OPERATIONS ===
    
    /// Store a service manifest in the DHT
    pub async fn store_manifest(&self, manifest: &ServiceManifest) -> Result<()> {
        let key = format!("manifest:{}/{}", manifest.name, manifest.version);
        let value = serde_yaml::to_string(manifest)?;
        self.store(key, value.into_bytes(), 86400).await // 24 hour TTL
    }
    
    /// Get a specific manifest by service name and version
    pub async fn get_manifest(&self, service_id: &str) -> Result<Option<ServiceManifest>> {
        // Try direct lookup first
        let key = format!("manifest:{}", service_id);
        if let Some(value) = self.get(&key).await {
            if let Ok(manifest) = serde_yaml::from_slice::<ServiceManifest>(&value) {
                return Ok(Some(manifest));
            }
        }
        
        // Search through all manifests
        let entries = self.entries.read().await;
        for (key, entry) in entries.iter() {
            if key.starts_with("manifest:") {
                if let Ok(manifest) = serde_yaml::from_slice::<ServiceManifest>(&entry.value) {
                    if format!("{}-{}", manifest.name, manifest.version) == service_id {
                        return Ok(Some(manifest));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Search for manifests matching criteria
    pub async fn search_manifests(&self, query: &str, filters: SearchFilters) -> Vec<ServiceManifest> {
        let entries = self.entries.read().await;
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        
        for (key, entry) in entries.iter() {
            if key.starts_with("manifest:") {
                if let Ok(manifest) = serde_yaml::from_slice::<ServiceManifest>(&entry.value) {
                    let mut matches = false;
                    
                    // Text search
                    if query.is_empty() || 
                       manifest.name.to_lowercase().contains(&query_lower) ||
                       manifest.description.to_lowercase().contains(&query_lower) ||
                       manifest.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) {
                        matches = true;
                    }
                    
                    // Category filter
                    if let Some(category) = &filters.category {
                        if manifest.category != *category {
                            matches = false;
                        }
                    }
                    
                    // Tags filter
                    if !filters.tags.is_empty() {
                        let has_any_tag = filters.tags.iter()
                            .any(|filter_tag| manifest.tags.contains(filter_tag));
                        if !has_any_tag {
                            matches = false;
                        }
                    }
                    
                    if matches {
                        results.push(manifest);
                    }
                }
            }
        }
        
        debug!("Found {} manifests matching query: {}", results.len(), query);
        results
    }
    
    /// Get all available service categories from stored manifests
    pub async fn get_manifest_categories(&self) -> Vec<String> {
        let entries = self.entries.read().await;
        let mut categories = std::collections::HashSet::new();
        
        for (key, entry) in entries.iter() {
            if key.starts_with("manifest:") {
                if let Ok(manifest) = serde_yaml::from_slice::<ServiceManifest>(&entry.value) {
                    categories.insert(manifest.category);
                }
            }
        }
        
        let mut result: Vec<String> = categories.into_iter().collect();
        result.sort();
        result
    }
    
    /// Get all available tags from stored manifests
    pub async fn get_manifest_tags(&self) -> Vec<String> {
        let entries = self.entries.read().await;
        let mut tags = std::collections::HashSet::new();
        
        for (key, entry) in entries.iter() {
            if key.starts_with("manifest:") {
                if let Ok(manifest) = serde_yaml::from_slice::<ServiceManifest>(&entry.value) {
                    for tag in &manifest.tags {
                        tags.insert(tag.clone());
                    }
                }
            }
        }
        
        let mut result: Vec<String> = tags.into_iter().collect();
        result.sort();
        result
    }
    
    /// Remove a manifest from the DHT
    pub async fn remove_manifest(&self, service_id: &str) -> Result<bool> {
        let key = format!("manifest:{}", service_id);
        let mut entries = self.entries.write().await;
        Ok(entries.remove(&key).is_some())
    }

    // === PHASE 4: Enhanced Discovery & Marketplace ===

    /// Store a service manifest in the DHT with multiple index keys
    pub async fn store_manifest_enhanced(&self, manifest: &ServiceManifest) -> Result<()> {
        let service_id = format!("{}:{}", manifest.name, manifest.version);
        
        // Main manifest storage
        let manifest_key = format!("manifest:{}", service_id);
        let manifest_value = serde_yaml::to_string(manifest)?;
        self.store(manifest_key, manifest_value.into_bytes(), 3600).await?;
        
        // Category index
        let category_key = format!("category:{}:{}", manifest.category, service_id);
        self.store(category_key, service_id.as_bytes().to_vec(), 3600).await?;
        
        // Tag indices
        for tag in &manifest.tags {
            let tag_key = format!("tag:{}:{}", tag, service_id);
            self.store(tag_key, service_id.as_bytes().to_vec(), 3600).await?;
        }
        
        // Author index
        let author_key = format!("author:{}:{}", manifest.author.did, service_id);
        self.store(author_key, service_id.as_bytes().to_vec(), 3600).await?;
        
        info!("Stored manifest for service: {} in DHT with indices", service_id);
        Ok(())
    }

    /// Search manifests with advanced filtering
    pub async fn search_manifests_enhanced(&self, query: &str, filters: SearchFilters) -> Vec<ServiceManifest> {
        let entries = self.entries.read().await;
        let mut results = Vec::new();
        let mut service_ids = std::collections::HashSet::new();
        
        // Search by category if specified
        if let Some(category) = &filters.category {
            let category_prefix = format!("category:{}", category);
            for (key, _) in entries.iter() {
                if key.starts_with(&category_prefix) {
                    if let Some(service_id) = key.split(':').nth(2) {
                        service_ids.insert(service_id.to_string());
                    }
                }
            }
        }
        
        // Search by tags if specified
        for tag in &filters.tags {
            let tag_prefix = format!("tag:{}", tag);
            for (key, _) in entries.iter() {
                if key.starts_with(&tag_prefix) {
                    if let Some(service_id) = key.split(':').nth(2) {
                        service_ids.insert(service_id.to_string());
                    }
                }
            }
        }
        
        // If no specific filters, search all manifests
        if filters.category.is_none() && filters.tags.is_empty() {
            for (key, _) in entries.iter() {
                if key.starts_with("manifest:") {
                    if let Some(service_id) = key.strip_prefix("manifest:") {
                        service_ids.insert(service_id.to_string());
                    }
                }
            }
        }
        
        // Retrieve and filter manifests
        for service_id in service_ids {
            let manifest_key = format!("manifest:{}", service_id);
            if let Some(entry) = entries.get(&manifest_key) {
                if let Ok(manifest) = serde_yaml::from_slice::<ServiceManifest>(&entry.value) {
                    // Apply query filter
                    if !query.is_empty() {
                        let query_lower = query.to_lowercase();
                        let matches_query = manifest.name.to_lowercase().contains(&query_lower)
                            || manifest.description.to_lowercase().contains(&query_lower)
                            || manifest.category.to_lowercase().contains(&query_lower)
                            || manifest.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower));
                        
                        if !matches_query {
                            continue;
                        }
                    }
                    
                    results.push(manifest);
                }
            }
        }
        
        results
    }

    /// Get all available categories from stored manifests
    pub async fn get_manifest_categories_enhanced(&self) -> Vec<String> {
        let entries = self.entries.read().await;
        let mut categories = std::collections::HashSet::new();
        
        for (key, _) in entries.iter() {
            if key.starts_with("category:") {
                if let Some(category) = key.split(':').nth(1) {
                    categories.insert(category.to_string());
                }
            }
        }
        
        let mut result: Vec<String> = categories.into_iter().collect();
        result.sort();
        result
    }

    /// Get popular services based on DHT activity
    pub async fn get_popular_services(&self, limit: usize) -> Vec<ServiceManifest> {
        let entries = self.entries.read().await;
        let mut manifests = Vec::new();
        
        for (key, entry) in entries.iter() {
            if key.starts_with("manifest:") {
                if let Ok(manifest) = serde_yaml::from_slice::<ServiceManifest>(&entry.value) {
                    manifests.push(manifest);
                }
                if manifests.len() >= limit {
                    break;
                }
            }
        }
        manifests
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DHTStats {
    pub total_entries: usize,
    pub total_peers: usize,
    pub service_entries: usize,
    pub reputation_entries: usize,
    pub escrow_entries: usize,
    pub manifest_entries: usize,
}
