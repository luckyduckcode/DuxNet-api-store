//! Database connection management
//! 
//! Provides connection pooling and configuration for PostgreSQL database

use anyhow::{Context, Result};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use sqlx::{PgPool, Pool as SqlxPool, Postgres};
use std::sync::Arc;
use tracing::{info, warn};

/// Database connection configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub max_connections: u32,
    pub ssl_mode: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            username: "duxnet".to_string(),
            password: "duxnet_dev_password".to_string(),
            database_name: "duxnet_development".to_string(),
            max_connections: 20,
            ssl_mode: "prefer".to_string(),
        }
    }
}

impl DatabaseConfig {
    /// Get database URL from configuration
    pub fn database_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}",
            self.username,
            self.password,
            self.host,
            self.port,
            self.database_name,
            self.ssl_mode
        )
    }
    
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: std::env::var("DATABASE_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("DATABASE_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .context("Invalid DATABASE_PORT")?,
            username: std::env::var("DATABASE_USER").unwrap_or_else(|_| "duxnet".to_string()),
            password: std::env::var("DATABASE_PASSWORD")
                .unwrap_or_else(|_| "duxnet_dev_password".to_string()),
            database_name: std::env::var("DATABASE_NAME")
                .unwrap_or_else(|_| "duxnet_development".to_string()),
            max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .context("Invalid DATABASE_MAX_CONNECTIONS")?,
            ssl_mode: std::env::var("DATABASE_SSL_MODE")
                .unwrap_or_else(|_| "prefer".to_string()),
        })
    }
}

/// Database connection pool manager
#[derive(Clone)]
pub struct ConnectionPool {
    pool: Arc<PgPool>,
    config: DatabaseConfig,
}

impl ConnectionPool {
    /// Create new connection pool
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        info!("Connecting to database: {}", config.database_name);
        
        let database_url = config.database_url();
        let pool = Arc::new(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(config.max_connections)
                .connect(&database_url)
                .await
                .context("Failed to connect to database")?
        );
        
        info!("Database connection pool created with {} max connections", config.max_connections);
        
        Ok(Self { pool, config })
    }
    
    /// Get database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    
    /// Get database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }
    
    /// Test database connection
    pub async fn test_connection(&self) -> Result<()> {
        let result: (i64,) = sqlx::query_as("SELECT 1 as test")
            .fetch_one(&*self.pool)
            .await
            .context("Failed to execute test query")?;
        
        if result.0 != 1 {
            anyhow::bail!("Database test query returned unexpected result");
        }
        
        info!("Database connection test successful");
        Ok(())
    }
    
    /// Get connection pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
            max_connections: self.config.max_connections,
        }
    }
    
    /// Close all connections in pool
    pub async fn close(&self) {
        info!("Closing database connection pool");
        self.pool.close().await;
    }
}

/// Database connection pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: usize,
    pub max_connections: u32,
}

impl std::fmt::Display for PoolStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pool Stats: {}/{} connections (idle: {})",
            self.size, self.max_connections, self.idle
        )
    }
}
