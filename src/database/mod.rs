//! Database module for DuxNet
//! 
//! This module provides database connectivity, models, and migrations
//! for the DuxNet platform. It uses PostgreSQL as the primary database
//! with SQLx for type-safe database operations.

pub mod connection;
pub mod migrations;
pub mod models;
pub mod repositories;
pub mod health_check;

pub use connection::*;
pub use models::*;
pub use repositories::*;

use anyhow::Result;
use sqlx::{PgPool, Pool, Postgres};
use std::sync::Arc;

/// Database manager for DuxNet
#[derive(Clone)]
pub struct DatabaseManager {
    pool: Arc<PgPool>,
}

impl DatabaseManager {
    /// Create new database manager with connection pool
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = Arc::new(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(20)
                .connect(database_url)
                .await?
        );
        
        Ok(Self { pool })
    }
    
    /// Get database connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    
    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./src/database/migrations")
            .run(&*self.pool)
            .await?;
        Ok(())
    }
    
    /// Check database health
    pub async fn health_check(&self) -> Result<bool> {
        let result: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(&*self.pool)
            .await?;
        Ok(result.0 == 1)
    }
}
