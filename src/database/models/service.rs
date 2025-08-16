//! Service model for database operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database model for services
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbService {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub manifest: serde_json::Value,
    pub service_type: String,
    pub version: String,
    pub status: String,
    pub tags: Vec<String>,
    pub pricing: serde_json::Value,
    pub metadata: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Service creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceRequest {
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub manifest: serde_json::Value,
    pub service_type: String,
    pub version: Option<String>,
    pub tags: Option<Vec<String>>,
    pub pricing: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

/// Service update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServiceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub manifest: Option<serde_json::Value>,
    pub version: Option<String>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
    pub pricing: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

/// Service status enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Pending,
    Active,
    Inactive,
    Suspended,
    Deprecated,
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::Pending => write!(f, "pending"),
            ServiceStatus::Active => write!(f, "active"),
            ServiceStatus::Inactive => write!(f, "inactive"),
            ServiceStatus::Suspended => write!(f, "suspended"),
            ServiceStatus::Deprecated => write!(f, "deprecated"),
        }
    }
}

impl DbService {
    /// Convert to DuxNet Service type
    pub fn to_duxnet_service(&self) -> anyhow::Result<crate::core::data_structures::Service> {
        let manifest: crate::core::data_structures::ServiceManifest = 
            serde_json::from_value(self.manifest.clone())?;
            
        Ok(crate::core::data_structures::Service {
            id: self.id.to_string(),
            manifest,
            owner: self.owner_id.to_string(),
            status: self.status.clone(),
            created_at: self.created_at.timestamp(),
        })
    }
}
