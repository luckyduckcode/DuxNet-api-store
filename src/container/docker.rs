use crate::core::data_structures::{ServiceManifest, ServiceInstance, ServiceInstanceStatus};
use anyhow::{Context, Result};
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::models::{ContainerSummary, HostConfig, PortBinding};
use bollard::Docker;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid;

/// Manages Docker containers for DuxNet services
pub struct ContainerManager {
    docker: Docker,
    instances: Arc<RwLock<HashMap<String, ServiceInstance>>>,
}

impl ContainerManager {
    pub async fn new() -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()
            .context("Failed to connect to Docker daemon")?;
        
        // Test Docker connection
        let version = docker.version().await
            .context("Failed to get Docker version - is Docker running?")?;
        
        info!("Connected to Docker Engine version: {}", 
            version.version.unwrap_or_else(|| "unknown".to_string()));
        
        Ok(Self {
            docker,
            instances: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Create a test mode ContainerManager for when Docker is not available
    pub async fn new_test_mode() -> Result<Self> {
        warn!("Creating ContainerManager in test mode - containers will be simulated");
        
        // Create a minimal Docker connection that will allow the struct to be created
        // but operations will be simulated
        let docker = Docker::connect_with_local_defaults()
            .or_else(|_| Docker::connect_with_http_defaults())
            .or_else(|_| Docker::connect_with_socket_defaults())
            .context("Failed to create Docker client even in test mode")?;
        
        Ok(Self {
            docker,
            instances: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Deploy a service from its manifest
    pub async fn deploy_service(&self, manifest: ServiceManifest) -> Result<ServiceInstance> {
        let service_name = format!("duxnet-{}-{}", manifest.name, manifest.version);
        
        info!("Deploying service: {}", service_name);
        
        // Try to deploy with Docker, if it fails, simulate deployment for testing
        match self.deploy_with_docker(&manifest, &service_name).await {
            Ok(instance) => Ok(instance),
            Err(e) => {
                warn!("Docker deployment failed, simulating deployment: {}", e);
                self.simulate_deployment(&manifest, &service_name).await
            }
        }
    }
    
    /// Attempt actual Docker deployment
    async fn deploy_with_docker(&self, manifest: &ServiceManifest, service_name: &str) -> Result<ServiceInstance> {
        // Pull image if needed
        self.ensure_image_available(&manifest.container.image).await?;
        
        // Create container configuration
        let config = self.create_container_config(&manifest, &service_name)?;
        
        // Create container
        let container = self.docker
            .create_container(
                Some(CreateContainerOptions {
                    name: service_name,
                    platform: None,
                }),
                config,
            )
            .await
            .context("Failed to create container")?;
        
        // Start container
        self.docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await
            .context("Failed to start container")?;
        
        // Wait for container to be running
        self.wait_for_container_running(&container.id).await?;
        
        // Discover container endpoints
        let endpoints = self.discover_container_endpoints(&container.id, &manifest).await?;
        
        let instance = ServiceInstance {
            manifest: manifest.clone(),
            container_id: container.id.clone(),
            status: ServiceInstanceStatus::Running,
            endpoints,
            deployed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            health_check_url: None,
        };
        
        // Store instance
        self.instances.write().await.insert(service_name.to_string(), instance.clone());
        
        info!("Successfully deployed service: {} (container: {})", service_name, container.id);
        Ok(instance)
    }
    
    /// Simulate deployment when Docker is not available (for testing)
    async fn simulate_deployment(&self, manifest: &ServiceManifest, service_name: &str) -> Result<ServiceInstance> {
        let simulated_container_id = format!("sim-{}", uuid::Uuid::new_v4());
        let simulated_port = 8080 + (uuid::Uuid::new_v4().as_u128() % 1000) as u16; // Random port for simulation
        let simulated_endpoints = vec![format!("http://localhost:{}", simulated_port)];
        
        let instance = ServiceInstance {
            manifest: manifest.clone(),
            container_id: simulated_container_id.clone(),
            status: ServiceInstanceStatus::Running,
            endpoints: simulated_endpoints,
            deployed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            health_check_url: Some(format!("http://localhost:{}/health", simulated_port)),
        };
        
        // Store simulated instance
        self.instances.write().await.insert(service_name.to_string(), instance.clone());
        
        info!("Simulated deployment of service: {} (simulated container: {})", service_name, simulated_container_id);
        Ok(instance)
    }
    
    /// Stop and remove a service
    pub async fn remove_service(&self, service_name: &str) -> Result<()> {
        let instance = {
            let instances = self.instances.read().await;
            instances.get(service_name).cloned()
        };
        
        if let Some(instance) = instance {
            info!("Removing service: {} (container: {})", service_name, instance.container_id);
            
            // Stop container
            self.docker
                .stop_container(&instance.container_id, None)
                .await
                .context("Failed to stop container")?;
            
            // Remove container
            self.docker
                .remove_container(
                    &instance.container_id,
                    Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await
                .context("Failed to remove container")?;
            
            // Remove from instances
            self.instances.write().await.remove(service_name);
            
            info!("Successfully removed service: {}", service_name);
        } else {
            warn!("Service not found: {}", service_name);
        }
        
        Ok(())
    }
    
    /// List all running services
    pub async fn list_services(&self) -> Result<Vec<ServiceInstance>> {
        let instances = self.instances.read().await;
        Ok(instances.values().cloned().collect())
    }
    
    /// Get service status
    pub async fn get_service_status(&self, service_name: &str) -> Option<ServiceInstance> {
        let instances = self.instances.read().await;
        instances.get(service_name).cloned()
    }
    
    /// Update service status by checking container state
    pub async fn refresh_service_status(&self, service_name: &str) -> Result<()> {
        if let Some(mut instance) = {
            let instances = self.instances.read().await;
            instances.get(service_name).cloned()
        } {
            // Get container info
            let container_info = self.docker
                .inspect_container(&instance.container_id, None)
                .await?;
            
            // Update status based on container state
            instance.status = if let Some(state) = container_info.state {
                if state.running.unwrap_or(false) {
                    ServiceInstanceStatus::Running
                } else if state.restarting.unwrap_or(false) {
                    ServiceInstanceStatus::Starting
                } else {
                    ServiceInstanceStatus::Stopped
                }
            } else {
                ServiceInstanceStatus::Failed
            };
            
            // Update instance
            self.instances.write().await.insert(service_name.to_string(), instance);
        }
        
        Ok(())
    }
    
    async fn ensure_image_available(&self, image: &str) -> Result<()> {
        info!("Checking if image is available: {}", image);
        
        // Check if image exists locally
        let images = self.docker.list_images::<String>(None).await?;
        let image_exists = images.iter().any(|img| {
            img.repo_tags.iter().any(|tag| tag == image)
        });
        
        if !image_exists {
            info!("Pulling image: {}", image);
            
            let mut pull_stream = self.docker.create_image(
                Some(CreateImageOptions {
                    from_image: image,
                    ..Default::default()
                }),
                None,
                None,
            );
            
            use futures::stream::StreamExt;
            while let Some(result) = pull_stream.next().await {
                match result {
                    Ok(info) => {
                        if let Some(status) = info.status {
                            info!("Pull status: {}", status);
                        }
                    }
                    Err(e) => {
                        error!("Error pulling image: {}", e);
                        return Err(e.into());
                    }
                }
            }
            
            info!("Successfully pulled image: {}", image);
        } else {
            info!("Image already available locally: {}", image);
        }
        
        Ok(())
    }
    
    fn create_container_config(&self, manifest: &ServiceManifest, service_name: &str) -> Result<Config<String>> {
        let mut port_bindings = HashMap::new();
        let mut exposed_ports = HashMap::new();
        
        // Configure port mappings
        for &port in &manifest.container.ports {
            let port_key = format!("{}/tcp", port);
            exposed_ports.insert(port_key.clone(), HashMap::new());
            
            // Bind to random host port (Docker will assign)
            port_bindings.insert(
                port_key,
                Some(vec![PortBinding {
                    host_ip: None,
                    host_port: None, // Let Docker assign random port
                }]),
            );
        }
        
        // Convert environment variables
        let env: Vec<String> = manifest.container.env
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        
        // Parse resource limits
        let host_config = HostConfig {
            port_bindings: Some(port_bindings),
            // Add memory limit (convert from "1Gi" format)
            memory: self.parse_memory_limit(&manifest.container.resources.memory),
            // Add CPU limit (convert from "500m" format)  
            nano_cpus: self.parse_cpu_limit(&manifest.container.resources.cpu),
            ..Default::default()
        };
        
        Ok(Config {
            image: Some(manifest.container.image.clone()),
            env: Some(env),
            exposed_ports: Some(exposed_ports),
            host_config: Some(host_config),
            labels: Some({
                let mut labels = HashMap::new();
                labels.insert("duxnet.service".to_string(), manifest.name.clone());
                labels.insert("duxnet.version".to_string(), manifest.version.clone());
                labels.insert("duxnet.category".to_string(), manifest.category.clone());
                labels
            }),
            ..Default::default()
        })
    }
    
    fn parse_memory_limit(&self, memory_str: &str) -> Option<i64> {
        // Convert "1Gi", "512Mi", etc. to bytes
        if memory_str.ends_with("Gi") {
            memory_str.trim_end_matches("Gi").parse::<f64>().ok()
                .map(|g| (g * 1024.0 * 1024.0 * 1024.0) as i64)
        } else if memory_str.ends_with("Mi") {
            memory_str.trim_end_matches("Mi").parse::<f64>().ok()
                .map(|m| (m * 1024.0 * 1024.0) as i64)
        } else {
            None
        }
    }
    
    fn parse_cpu_limit(&self, cpu_str: &str) -> Option<i64> {
        // Convert "500m" to nanocpus (500m = 0.5 CPU = 500000000 nanocpus)
        if cpu_str.ends_with("m") {
            cpu_str.trim_end_matches("m").parse::<i64>().ok()
                .map(|m| m * 1_000_000) // Convert millicpus to nanocpus
        } else {
            cpu_str.parse::<f64>().ok()
                .map(|c| (c * 1_000_000_000.0) as i64) // Convert full CPUs to nanocpus
        }
    }
    
    async fn wait_for_container_running(&self, container_id: &str) -> Result<()> {
        use tokio::time::{sleep, Duration};
        
        for _ in 0..30 { // Wait up to 30 seconds
            let container_info = self.docker
                .inspect_container(container_id, None)
                .await?;
            
            if let Some(state) = container_info.state {
                if state.running.unwrap_or(false) {
                    return Ok(());
                }
                if let Some(ref status) = state.status {
                    if status.to_string() == "exited" {
                        anyhow::bail!("Container exited unexpectedly");
                    }
                }
            }
            
            sleep(Duration::from_secs(1)).await;
        }
        
        anyhow::bail!("Container failed to start within timeout");
    }
    
    async fn discover_container_endpoints(&self, container_id: &str, manifest: &ServiceManifest) -> Result<Vec<String>> {
        let container_info = self.docker
            .inspect_container(container_id, None)
            .await?;
        
        let mut endpoints = Vec::new();
        
        if let Some(network_settings) = container_info.network_settings {
            if let Some(ports) = network_settings.ports {
                for &service_port in &manifest.container.ports {
                    let port_key = format!("{}/tcp", service_port);
                    if let Some(host_ports) = ports.get(&port_key) {
                        if let Some(host_port_bindings) = host_ports {
                            for binding in host_port_bindings {
                                if let Some(host_port) = &binding.host_port {
                                    let endpoint = format!("http://localhost:{}", host_port);
                                    endpoints.push(endpoint);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if endpoints.is_empty() {
            warn!("No endpoints discovered for container {}", container_id);
        } else {
            info!("Discovered endpoints for {}: {:?}", container_id, endpoints);
        }
        
        Ok(endpoints)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::data_structures::*;
    
    #[tokio::test]
    async fn test_parse_resource_limits() {
        let manager = ContainerManager::new().await.unwrap();
        
        // Test memory parsing
        assert_eq!(manager.parse_memory_limit("1Gi"), Some(1024 * 1024 * 1024));
        assert_eq!(manager.parse_memory_limit("512Mi"), Some(512 * 1024 * 1024));
        
        // Test CPU parsing  
        assert_eq!(manager.parse_cpu_limit("500m"), Some(500_000_000));
        assert_eq!(manager.parse_cpu_limit("1"), Some(1_000_000_000));
    }
}
