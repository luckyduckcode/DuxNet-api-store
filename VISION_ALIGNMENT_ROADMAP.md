# DuxNet Vision Alignment Roadmap

## 🎯 Current State vs Vision Gap Analysis

### ✅ **What's Already Excellent**
1. **Solid Rust Foundation** - Tokio async, modular architecture
2. **Digital Identity** - Ed25519 DID system implemented
3. **P2P Core** - DHT, reputation, messaging modules
4. **Wallet Integration** - DUX token support
5. **Enhanced API Store** - Advanced service management, analytics
6. **Web Interface** - Tauri desktop app

### ❌ **Critical Gaps to Address**

## 🚀 **Phase 1: YAML Manifest System (Week 1-2)**

### **1.1 YAML Service Manifest Support**
```rust
// Add to src/core/data_structures.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub author: Author,
    pub api: ApiDefinition,
    pub container: ContainerDefinition,
    pub pricing: PricingModel,
    pub sla: ServiceLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerDefinition {
    pub image: String,
    pub ports: Vec<u16>,
    pub env: HashMap<String, String>,
    pub resources: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDefinition {
    pub openapi: String,
    pub endpoints: Vec<Endpoint>,
    pub base_path: Option<String>,
}
```

### **1.2 YAML Parsing & Validation**
```rust
// Add to Cargo.toml
serde_yaml = "0.9"
jsonschema = "0.17"

// Add to src/core/manifest.rs
use serde_yaml;
use jsonschema::JSONSchema;

pub struct ManifestValidator {
    schema: JSONSchema,
}

impl ManifestValidator {
    pub fn validate_manifest(&self, yaml_content: &str) -> Result<ServiceManifest> {
        let manifest: ServiceManifest = serde_yaml::from_str(yaml_content)?;
        // Validate against JSON schema
        self.schema.validate(&serde_json::to_value(&manifest)?)?;
        Ok(manifest)
    }
}
```

### **1.3 Enhanced Service Registration**
```rust
// Update src/api/handlers.rs
#[derive(Debug, Deserialize)]
pub struct RegisterManifestRequest {
    pub manifest_yaml: String,
    pub signature: Option<String>,
}

pub async fn register_service_manifest(
    State(state): State<ApiState>,
    Json(request): Json<RegisterManifestRequest>,
) -> impl IntoResponse {
    let validator = ManifestValidator::new();
    match validator.validate_manifest(&request.manifest_yaml) {
        Ok(manifest) => {
            let service_id = state.node.register_manifest(manifest).await?;
            Json(RegisterServiceResponse { service_id, success: true })
        }
        Err(e) => Json(ErrorResponse { error: e.to_string() })
    }
}
```

## 🚀 **Phase 2: Container Integration (Week 2-3)**

### **2.1 Docker Integration**
```rust
// Add to Cargo.toml
bollard = "0.14"  // Docker API client

// Add src/container/docker.rs
use bollard::Docker;
use bollard::container::{CreateContainerOptions, Config};

pub struct ContainerManager {
    docker: Docker,
}

impl ContainerManager {
    pub async fn deploy_service(&self, manifest: &ServiceManifest) -> Result<String> {
        let config = Config {
            image: Some(manifest.container.image.clone()),
            env: Some(manifest.container.env.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect()),
            exposed_ports: Some(manifest.container.ports.iter()
                .map(|p| (format!("{}/tcp", p), HashMap::new()))
                .collect()),
            ..Default::default()
        };
        
        let container = self.docker
            .create_container(Some(CreateContainerOptions {
                name: &format!("duxnet-{}", manifest.name),
            }), config)
            .await?;
            
        self.docker.start_container(&container.id, None).await?;
        Ok(container.id)
    }
}
```

### **2.2 Service Lifecycle Management**
```rust
// Add src/core/service_manager.rs
pub struct ServiceManager {
    container_manager: ContainerManager,
    registry: Arc<RwLock<HashMap<ServiceId, ServiceInstance>>>,
}

#[derive(Debug)]
pub struct ServiceInstance {
    pub manifest: ServiceManifest,
    pub container_id: String,
    pub status: ServiceStatus,
    pub endpoints: Vec<String>,
}

impl ServiceManager {
    pub async fn deploy_service(&self, manifest: ServiceManifest) -> Result<ServiceId> {
        // Deploy container
        let container_id = self.container_manager.deploy_service(&manifest).await?;
        
        // Register in P2P network
        self.announce_service_p2p(&manifest).await?;
        
        // Store instance
        let service_id = ServiceId::new();
        let instance = ServiceInstance {
            manifest,
            container_id,
            status: ServiceStatus::Running,
            endpoints: self.discover_endpoints(&container_id).await?,
        };
        
        self.registry.write().await.insert(service_id, instance);
        Ok(service_id)
    }
}
```

## 🚀 **Phase 3: P2P API Gateway (Week 3-4)**

### **3.1 P2P API Proxy**
```rust
// Add src/gateway/proxy.rs
use axum::{extract::Path, response::Response};
use reqwest::Client;

pub struct P2PApiGateway {
    client: Client,
    service_registry: Arc<ServiceManager>,
}

impl P2PApiGateway {
    pub async fn proxy_request(
        &self,
        service_id: &str,
        path: &str,
        method: Method,
        body: Bytes,
    ) -> Result<Response> {
        // Discover service endpoint via P2P
        let service = self.service_registry.find_service(service_id).await?;
        
        // Route to container endpoint
        let endpoint = service.endpoints[0].clone();
        let url = format!("{}{}", endpoint, path);
        
        let response = self.client
            .request(method, &url)
            .body(body)
            .send()
            .await?;
            
        Ok(response.into())
    }
}
```

### **3.2 NAT Traversal for Direct P2P API Calls**
```rust
// Add src/network/nat_traversal.rs
use libp2p::core::transport::Transport;
use libp2p::webrtc;

pub struct P2PApiTransport {
    transport: Box<dyn Transport<Output = Connection>>,
}

impl P2PApiTransport {
    pub async fn call_peer_api(
        &self,
        peer_id: PeerId,
        service_id: &str,
        request: ApiRequest,
    ) -> Result<ApiResponse> {
        // Establish direct P2P connection
        let mut conn = self.transport.dial(peer_id).await?;
        
        // Send API request over P2P channel
        let protocol = format!("/duxnet/api/{}/1.0.0", service_id);
        let mut stream = conn.open_stream(protocol).await?;
        
        // Serialize and send request
        let request_bytes = bincode::serialize(&request)?;
        stream.write_all(&request_bytes).await?;
        
        // Read response
        let mut response_bytes = Vec::new();
        stream.read_to_end(&mut response_bytes).await?;
        let response = bincode::deserialize(&response_bytes)?;
        
        Ok(response)
    }
}
```

## 🚀 **Phase 4: Enhanced Discovery & Marketplace (Week 4-5)**

### **4.1 YAML Manifest Storage in DHT**
```rust
// Update src/core/dht.rs
impl DHT {
    pub async fn store_manifest(&self, manifest: &ServiceManifest) -> Result<()> {
        let key = format!("manifest/{}/{}", manifest.name, manifest.version);
        let value = serde_yaml::to_string(manifest)?;
        self.store(key, value.into_bytes(), 3600).await
    }
    
    pub async fn search_manifests(&self, query: &str, filters: SearchFilters) -> Vec<ServiceManifest> {
        let mut results = Vec::new();
        
        // Search by category
        if let Some(category) = &filters.category {
            let key = format!("category/{}", category);
            let manifests = self.get_records_by_prefix(&key).await;
            results.extend(manifests);
        }
        
        // Search by tags
        for tag in &filters.tags {
            let key = format!("tag/{}", tag);
            let manifests = self.get_records_by_prefix(&key).await;
            results.extend(manifests);
        }
        
        results
    }
}
```

### **4.2 GitHub-like Interface**
```rust
// Add src/api/marketplace.rs
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub sort_by: SortOption,
    pub limit: u32,
    pub offset: u32,
}

pub async fn search_marketplace(
    State(state): State<ApiState>,
    Json(request): Json<SearchRequest>,
) -> impl IntoResponse {
    let results = state.node.dht
        .search_manifests(&request.query, SearchFilters {
            category: request.category,
            tags: request.tags,
        })
        .await;
        
    let sorted_results = sort_services(results, request.sort_by);
    let paginated = paginate(sorted_results, request.limit, request.offset);
    
    Json(SearchResponse {
        services: paginated,
        total: results.len(),
        query: request.query,
    })
}
```

## 🚀 **Phase 5: Integration & Testing (Week 5-6)**

### **5.1 End-to-End Service Flow**
1. **Publish YAML Manifest** → Validate → Deploy Container → Announce P2P
2. **Discover Services** → Search DHT → Find Peers → Connect P2P
3. **Make API Calls** → Route via Gateway OR Direct P2P
4. **Payment & Escrow** → DUX tokens → Smart contracts

### **5.2 Frontend Updates**
```javascript
// Update frontend/script.js
async function registerServiceFromYaml() {
    const yamlContent = document.getElementById('serviceManifest').value;
    
    const response = await fetch('/api/services/manifest', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ manifest_yaml: yamlContent })
    });
    
    if (response.ok) {
        showNotification('Service deployed successfully!');
        loadServices();
    }
}
```

## 📋 **Implementation Priority**

1. **🔥 Critical (Week 1):** YAML manifest system + validation
2. **🔥 Critical (Week 2):** Basic container integration 
3. **⚡ High (Week 3):** P2P API gateway
4. **⚡ High (Week 4):** Enhanced DHT search
5. **📈 Medium (Week 5):** Direct P2P API calls
6. **📈 Medium (Week 6):** Frontend integration

## 🎯 **Success Metrics**

- ✅ YAML service manifests deployable
- ✅ Services running in Docker containers
- ✅ P2P discovery working
- ✅ API calls routable via gateway
- ✅ Direct P2P API calls functional
- ✅ GitHub-like marketplace UI

This roadmap transforms your current excellent foundation into the full vision from your README!
