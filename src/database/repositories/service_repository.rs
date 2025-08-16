//! Service repository for database operations

use crate::database::models::{DbService, CreateServiceRequest, UpdateServiceRequest};
use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Repository for service operations
#[derive(Clone)]
pub struct ServiceRepository {
    pool: Arc<PgPool>,
}

impl ServiceRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Create a new service
    pub async fn create(&self, service: CreateServiceRequest) -> Result<DbService> {
        let now = Utc::now();
        let service_id = Uuid::new_v4(); // Generate new UUID for service
        
        let query = "INSERT INTO services (id, owner_id, name, description, manifest, service_type, version, tags, pricing, metadata, is_active, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) RETURNING *";
        
        let service = sqlx::query_as::<_, DbService>(query)
            .bind(service_id)
            .bind(service.owner_id)
            .bind(&service.name)
            .bind(&service.description)
            .bind(&service.manifest)
            .bind(&service.service_type)
            .bind(&service.version.unwrap_or_else(|| "1.0.0".to_string()))
            .bind(&service.tags.unwrap_or_default())
            .bind(&service.pricing.unwrap_or_else(|| serde_json::json!({})))
            .bind(&service.metadata.unwrap_or_else(|| serde_json::json!({})))
            .bind(true)
            .bind("starting")
            .bind(now)
            .bind(now)
            .fetch_one(&*self.pool)
            .await?;
            
        Ok(service)
    }

    /// Find service by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<DbService>> {
        let query = "SELECT * FROM services WHERE id = $1";
        
        let service = sqlx::query_as::<_, DbService>(query)
            .bind(id)
            .fetch_optional(&*self.pool)
            .await?;
            
        Ok(service)
    }

    /// Find services by owner
    pub async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<DbService>> {
        let query = "SELECT * FROM services WHERE owner_id = $1 ORDER BY created_at DESC";
        
        let services = sqlx::query_as::<_, DbService>(query)
            .bind(owner_id)
            .fetch_all(&*self.pool)
            .await
            .context("Failed to find services by owner")?;

        Ok(services)
    }

    /// Find services by name pattern
    pub async fn find_by_name(&self, name_pattern: &str) -> Result<Vec<DbService>> {
        let query = "SELECT * FROM services WHERE name ILIKE $1 ORDER BY created_at DESC";
        
        let services = sqlx::query_as::<_, DbService>(query)
            .bind(format!("%{}%", name_pattern))
            .fetch_all(&*self.pool)
            .await
            .context("Failed to find services by name")?;

        Ok(services)
    }

    /// Find active services
    pub async fn find_active(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<DbService>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        
        let query = "SELECT * FROM services WHERE is_active = true ORDER BY created_at DESC LIMIT $1 OFFSET $2";
        
        let services = sqlx::query_as::<_, DbService>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await
            .context("Failed to find active services")?;

        Ok(services)
    }

    /// Update service
    pub async fn update(&self, id: Uuid, request: UpdateServiceRequest) -> Result<Option<DbService>> {
        let now = chrono::Utc::now();
        
        let query = "UPDATE services SET name = COALESCE($1, name), description = COALESCE($2, description), manifest = COALESCE($3, manifest), version = COALESCE($4, version), status = COALESCE($5, status), tags = COALESCE($6, tags), pricing = COALESCE($7, pricing), metadata = COALESCE($8, metadata), is_active = COALESCE($9, is_active), updated_at = $10 WHERE id = $11 RETURNING *";
        
        let service = sqlx::query_as::<_, DbService>(query)
            .bind(request.name)
            .bind(request.description)
            .bind(request.manifest)
            .bind(request.version)
            .bind(request.status)
            .bind(request.tags.as_deref())
            .bind(request.pricing)
            .bind(request.metadata)
            .bind(request.is_active)
            .bind(now)
            .bind(id)
            .fetch_optional(&*self.pool)
            .await
            .context("Failed to update service")?;

        Ok(service)
    }

    /// Update service status
    pub async fn update_status(&self, id: Uuid, status: &str) -> Result<Option<DbService>> {
        let now = chrono::Utc::now();
        
        let query = "UPDATE services SET status = $1, updated_at = $2 WHERE id = $3 RETURNING *";
        
        let service = sqlx::query_as::<_, DbService>(query)
            .bind(status)
            .bind(now)
            .bind(id)
            .fetch_optional(&*self.pool)
            .await
            .context("Failed to update service status")?;

        Ok(service)
    }

    /// Delete service (soft delete by setting is_active = false)
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let now = chrono::Utc::now();
        
        let query = "UPDATE services SET is_active = false, updated_at = $1 WHERE id = $2";
        
        let result = sqlx::query(query)
            .bind(now)
            .bind(id)
            .execute(&*self.pool)
            .await
            .context("Failed to delete service")?;

        Ok(result.rows_affected() > 0)
    }

    /// Hard delete service (permanently remove from database)
    pub async fn hard_delete(&self, id: Uuid) -> Result<bool> {
        let query = "DELETE FROM services WHERE id = $1";
        
        let result = sqlx::query(query)
            .bind(id)
            .execute(&*self.pool)
            .await
            .context("Failed to hard delete service")?;

        Ok(result.rows_affected() > 0)
    }

    /// List all services with pagination
    pub async fn list(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<DbService>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        
        let query = "SELECT * FROM services ORDER BY created_at DESC LIMIT $1 OFFSET $2";
        
        let services = sqlx::query_as::<_, DbService>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await
            .context("Failed to list services")?;

        Ok(services)
    }

    /// Count total services
    pub async fn count(&self) -> Result<i64> {
        let query = "SELECT COUNT(*) FROM services";
        
        let count: Option<i64> = sqlx::query_scalar(query)
            .fetch_one(&*self.pool)
            .await
            .context("Failed to count services")?;

        Ok(count.unwrap_or(0))
    }

    /// Count active services
    pub async fn count_active(&self) -> Result<i64> {
        let query = "SELECT COUNT(*) FROM services WHERE is_active = true";
        
        let count: Option<i64> = sqlx::query_scalar(query)
            .fetch_one(&*self.pool)
            .await
            .context("Failed to count active services")?;

        Ok(count.unwrap_or(0))
    }
}
