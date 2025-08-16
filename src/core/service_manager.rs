use crate::core::data_structures::{ServiceManifest, ServiceInstance, ServiceId, ServiceInstanceStatus};
use crate::container::docker::ContainerManager;
use crate::core::dht::DHT;
use crate::database::{RepositoryManager, models::{CreateServiceRequest, UpdateServiceRequest}};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Central service lifecycle manager that coordinates container deployment,
/// P2P announcement, and service registry management
pub struct ServiceManager {
    container_manager: ContainerManager,
    dht: Arc<DHT>,
    db_repos: Option<RepositoryManager>,
    registry: Arc<RwLock<HashMap<ServiceId, ServiceInstance>>>,
}

impl ServiceManager {
    pub async fn new(dht: Arc<DHT>) -> Result<Self> {
        let container_manager = match ContainerManager::new().await {
            Ok(manager) => {
                info!("ServiceManager initialized with Docker integration");
                manager
            }
            Err(e) => {
                warn!("Docker not available, ServiceManager running in test mode: {}", e);
                // Create a mock container manager for testing
                ContainerManager::new_test_mode().await?
            }
        };
        
        Ok(Self {
            container_manager,
            dht,
            db_repos: None,
            registry: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Initialize with database repository manager
    pub fn with_database(mut self, db_repos: RepositoryManager) -> Self {
        self.db_repos = Some(db_repos);
        self
    }
    
    /// Deploy a service from its manifest
    /// This is the main entry point for service deployment
    pub async fn deploy_service(&self, manifest: ServiceManifest) -> Result<ServiceId> {
        let service_id = ServiceId::new();
        
        info!("Deploying service: {} (ID: {})", manifest.name, service_id.0);
        
        // 1. Deploy container using Docker
        let mut instance = self.container_manager.deploy_service(manifest.clone()).await
            .context("Failed to deploy service container")?;
        
        // 2. Store in database if available
        if let Some(ref db_repos) = self.db_repos {
            let service_request = CreateServiceRequest {
                owner_id: Uuid::parse_str(&service_id.0).unwrap_or_else(|_| Uuid::new_v4()),
                name: manifest.name.clone(),
                description: Some(manifest.description.clone()),
                manifest: serde_json::to_value(&manifest).context("Failed to serialize manifest")?,
                service_type: manifest.category.clone(),
                version: Some(manifest.version.clone()),
                tags: Some(manifest.tags.clone()),
                pricing: Some(serde_json::json!({
                    "compute_units": 100,
                    "cost_per_request": 10
                })),
                metadata: Some(serde_json::json!({
                    "container_id": instance.container_id,
                    "endpoints": instance.endpoints,
                    "status": format!("{:?}", instance.status)
                })),
            };
            
            match db_repos.services.create(service_request).await {
                Ok(db_service) => {
                    info!("Service stored in database with ID: {}", db_service.id);
                },
                Err(e) => {
                    warn!("Failed to store service in database: {}", e);
                    // Continue with deployment even if database fails
                }
            }
        }
        
        // 3. Announce service in P2P network
        self.announce_service_p2p(&manifest, &service_id).await
            .context("Failed to announce service in P2P network")?;
        
        // 4. Store in local registry
        self.registry.write().await.insert(service_id.clone(), instance.clone());
        
        // 5. Update instance with service ID for future reference
        instance.manifest = manifest;
        
        info!("Successfully deployed service: {} with ID: {}", 
            instance.manifest.name, service_id.0);
        
        Ok(service_id)
    }
    
    /// Stop and remove a service
    pub async fn remove_service(&self, service_id: &ServiceId) -> Result<()> {
        let instance = {
            let registry = self.registry.read().await;
            registry.get(service_id).cloned()
        };
        
        if let Some(instance) = instance {
            let service_name = format!("duxnet-{}-{}", 
                instance.manifest.name, instance.manifest.version);
            
            info!("Removing service: {} (ID: {})", service_name, service_id.0);
            
            // 1. Remove from container manager
            self.container_manager.remove_service(&service_name).await
                .context("Failed to remove container")?;
            
            // 2. Remove from database if available
            if let Some(uuid) = Uuid::parse_str(&service_id.0).ok() {
                if let Err(e) = self.remove_service_from_db(uuid).await {
                    warn!("Failed to remove service from database: {}", e);
                    // Continue with removal even if database fails
                }
            }
            
            // 3. Remove from P2P network
            self.remove_service_p2p(service_id).await
                .context("Failed to remove service from P2P network")?;
            
            // 4. Remove from local registry
            self.registry.write().await.remove(service_id);
            
            info!("Successfully removed service: {}", service_name);
        } else {
            warn!("Service not found in registry: {}", service_id.0);
        }
        
        Ok(())
    }
    
    /// Get a service instance by ID
    pub async fn get_service(&self, service_id: &ServiceId) -> Option<ServiceInstance> {
        let registry = self.registry.read().await;
        registry.get(service_id).cloned()
    }
    
    /// List all active services
    pub async fn list_services(&self) -> Vec<(ServiceId, ServiceInstance)> {
        let registry = self.registry.read().await;
        registry.iter().map(|(id, instance)| (id.clone(), instance.clone())).collect()
    }
    
    /// Get service by ServiceId (alias for get_service, required by API gateway)
    pub async fn get_service_by_id(&self, service_id: &ServiceId) -> Option<ServiceInstance> {
        self.get_service(service_id).await
    }
    
    /// Find services by name pattern
    pub async fn find_services_by_name(&self, name_pattern: &str) -> Vec<(ServiceId, ServiceInstance)> {
        let registry = self.registry.read().await;
        registry.iter()
            .filter(|(_, instance)| instance.manifest.name.contains(name_pattern))
            .map(|(id, instance)| (id.clone(), instance.clone()))
            .collect()
    }
    
    /// Update service status by refreshing container state
    pub async fn refresh_service_status(&self, service_id: &ServiceId) -> Result<()> {
        if let Some(instance) = self.get_service(service_id).await {
            let service_name = format!("duxnet-{}-{}", 
                instance.manifest.name, instance.manifest.version);
            
            // Refresh status from container manager
            self.container_manager.refresh_service_status(&service_name).await
                .context("Failed to refresh container status")?;
            
            // Get updated status
            if let Some(updated_instance) = self.container_manager.get_service_status(&service_name).await {
                let mut registry = self.registry.write().await;
                registry.insert(service_id.clone(), updated_instance);
            }
        }
        
        Ok(())
    }
    
    /// Announce service availability in P2P network
    async fn announce_service_p2p(&self, manifest: &ServiceManifest, service_id: &ServiceId) -> Result<()> {
        info!("Announcing service in P2P network: {}", manifest.name);
        
        // Store manifest in DHT for discovery
        self.dht.store_manifest(manifest).await
            .context("Failed to store manifest in DHT")?;
        
        // Store service mapping (service_id -> manifest)
        let mapping_key = format!("service_mapping/{}", service_id.0);
        let manifest_yaml = serde_yaml::to_string(manifest)
            .context("Failed to serialize manifest to YAML")?;
        
        self.dht.store(mapping_key, manifest_yaml.into_bytes(), 3600).await
            .context("Failed to store service mapping in DHT")?;
        
        // Announce in service category
        let category_key = format!("category/{}/{}", manifest.category, service_id.0);
        self.dht.store(category_key, service_id.0.clone().into_bytes(), 3600).await
            .context("Failed to announce service in category")?;
        
        // Announce with tags
        for tag in &manifest.tags {
            let tag_key = format!("tag/{}/{}", tag, service_id.0);
            self.dht.store(tag_key, service_id.0.clone().into_bytes(), 3600).await
                .context("Failed to announce service with tag")?;
        }
        
        info!("Successfully announced service {} in P2P network", manifest.name);
        Ok(())
    }
    
    /// Remove service from P2P network
    async fn remove_service_p2p(&self, service_id: &ServiceId) -> Result<()> {
        info!("Removing service from P2P network: {}", service_id.0);
        
        // Remove service mapping
        let mapping_key = format!("service_mapping/{}", service_id.0);
        self.dht.remove(&mapping_key).await
            .context("Failed to remove service mapping from DHT")?;
        
        // Note: We could also remove from categories and tags, but that would require
        // knowing the manifest. For now, we'll rely on TTL expiration.
        
        info!("Successfully removed service {} from P2P network", service_id.0);
        Ok(())
    }
    
    /// Discover endpoints for a running container
    pub async fn discover_endpoints(&self, _container_id: &str) -> Result<Vec<String>> {
        // This would inspect the container and return accessible endpoints
        // For now, we'll return a placeholder based on container inspection
        let endpoints = vec![
            format!("http://localhost:8080"), // Default endpoint
        ];
        
        Ok(endpoints)
    }
    
    /// Health check for a service
    pub async fn health_check_service(&self, service_id: &ServiceId) -> Result<bool> {
        if let Some(instance) = self.get_service(service_id).await {
            if let Some(health_url) = &instance.health_check_url {
                // Perform HTTP health check
                let client = reqwest::Client::new();
                match client.get(health_url).send().await {
                    Ok(response) => Ok(response.status().is_success()),
                    Err(_) => Ok(false),
                }
            } else {
                // If no health check URL, assume healthy if container is running
                Ok(matches!(instance.status, ServiceInstanceStatus::Running))
            }
        } else {
            Ok(false)
        }
    }
    
    /// Get service statistics
    pub async fn get_service_stats(&self) -> ServiceManagerStats {
        let registry = self.registry.read().await;
        let total_services = registry.len();
        let running_services = registry.values()
            .filter(|instance| matches!(instance.status, ServiceInstanceStatus::Running))
            .count();
        let failed_services = registry.values()
            .filter(|instance| matches!(instance.status, ServiceInstanceStatus::Failed))
            .count();
        
        ServiceManagerStats {
            total_services,
            running_services,
            failed_services,
            starting_services: registry.values()
                .filter(|instance| matches!(instance.status, ServiceInstanceStatus::Starting))
                .count(),
        }
    }

    // ===== DATABASE OPERATIONS =====
    
    /// Get services from database with pagination
    pub async fn get_services_from_db(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<crate::database::models::DbService>> {
        if let Some(ref db_repos) = self.db_repos {
            db_repos.services.list(limit, offset).await
        } else {
            warn!("Database repository not available");
            Ok(vec![])
        }
    }
    
    /// Find service in database by ID
    pub async fn find_service_in_db(&self, service_id: Uuid) -> Result<Option<crate::database::models::DbService>> {
        if let Some(ref db_repos) = self.db_repos {
            db_repos.services.find_by_id(service_id).await
        } else {
            warn!("Database repository not available");
            Ok(None)
        }
    }
    
    /// Find services by owner in database
    pub async fn find_services_by_owner_in_db(&self, owner_id: Uuid) -> Result<Vec<crate::database::models::DbService>> {
        if let Some(ref db_repos) = self.db_repos {
            db_repos.services.find_by_owner(owner_id).await
        } else {
            warn!("Database repository not available");
            Ok(vec![])
        }
    }
    
    /// Find active services from database
    pub async fn get_active_services_from_db(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<crate::database::models::DbService>> {
        if let Some(ref db_repos) = self.db_repos {
            db_repos.services.find_active(limit, offset).await
        } else {
            warn!("Database repository not available");
            Ok(vec![])
        }
    }
    
    /// Update service status in database
    pub async fn update_service_status_in_db(&self, service_id: Uuid, status: &str) -> Result<()> {
        if let Some(ref db_repos) = self.db_repos {
            match db_repos.services.update_status(service_id, status).await {
                Ok(Some(_)) => {
                    info!("Updated service status in database: {} -> {}", service_id, status);
                    Ok(())
                },
                Ok(None) => {
                    warn!("Service not found in database: {}", service_id);
                    Ok(())
                },
                Err(e) => {
                    error!("Failed to update service status in database: {}", e);
                    Err(e)
                }
            }
        } else {
            warn!("Database repository not available");
            Ok(())
        }
    }
    
    /// Remove service from database (soft delete)
    pub async fn remove_service_from_db(&self, service_id: Uuid) -> Result<()> {
        if let Some(ref db_repos) = self.db_repos {
            match db_repos.services.delete(service_id).await {
                Ok(true) => {
                    info!("Removed service from database: {}", service_id);
                    Ok(())
                },
                Ok(false) => {
                    warn!("Service not found in database: {}", service_id);
                    Ok(())
                },
                Err(e) => {
                    error!("Failed to remove service from database: {}", e);
                    Err(e)
                }
            }
        } else {
            warn!("Database repository not available");
            Ok(())
        }
    }
    
    /// Search services by name pattern in database
    pub async fn search_services_in_db(&self, name_pattern: &str) -> Result<Vec<crate::database::models::DbService>> {
        if let Some(ref db_repos) = self.db_repos {
            db_repos.services.find_by_name(name_pattern).await
        } else {
            warn!("Database repository not available");
            Ok(vec![])
        }
    }
    
    /// Get database statistics
    pub async fn get_service_stats_from_db(&self) -> Result<(i64, i64)> {
        if let Some(ref db_repos) = self.db_repos {
            let total = db_repos.services.count().await?;
            let active = db_repos.services.count_active().await?;
            Ok((total, active))
        } else {
            warn!("Database repository not available");
            Ok((0, 0))
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServiceManagerStats {
    pub total_services: usize,
    pub running_services: usize,
    pub failed_services: usize,
    pub starting_services: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::data_structures::*;
    
    fn create_test_manifest() -> ServiceManifest {
        ServiceManifest {
            name: "test-service".to_string(),
            version: "1.0.0".to_string(),
            description: "Test service for unit tests".to_string(),
            category: "testing".to_string(),
            tags: vec!["test".to_string(), "unit".to_string()],
            author: Author {
                name: "Test Author".to_string(),
                email: Some("test@example.com".to_string()),
                did: "did:test:123".to_string(),
                contact: None,
            },
            container: ContainerDefinition {
                image: "nginx:alpine".to_string(),
                ports: vec![80],
                env: std::collections::HashMap::new(),
                resources: ResourceLimits {
                    cpu: "100m".to_string(),
                    memory: "128Mi".to_string(),
                    gpu: None,
                },
            },
            api: ApiDefinition {
                openapi: "3.0.0".to_string(),
                base_path: Some("/api".to_string()),
                endpoints: vec![],
            },
            pricing: PricingModel::Free,
            sla: ServiceLevel {
                uptime: 99.9,
                response_time_ms: 100,
                throughput_rps: 1000,
            },
            reputation: Some(ReputationInfo {
                score: 85.0,
                total_calls: 100,
                reviews: 20,
                uptime_actual: 99.5,
            }),
        }
    }

    #[test]
    fn test_service_deployment() {
        // Test service deployment with mock containers
        // This will be expanded when container runtime is implemented
    }
}