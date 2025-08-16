//! Repository patterns for database operations

mod user_repository;
mod service_repository;
mod transaction_repository;

// Re-export repository structs (all enabled)
pub use user_repository::UserRepository;
pub use service_repository::ServiceRepository;
pub use transaction_repository::TransactionRepository;

use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;

/// Base repository trait for common operations
#[async_trait::async_trait]
pub trait Repository<T, ID> {
    async fn create(&self, entity: &T) -> Result<T>;
    async fn find_by_id(&self, id: &ID) -> Result<Option<T>>;
    async fn update(&self, entity: &T) -> Result<()>;
    async fn delete(&self, id: &ID) -> Result<()>;
    async fn list(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<T>>;
}

/// Repository manager that holds all repositories
#[derive(Clone)]
pub struct RepositoryManager {
    pool: Arc<PgPool>,
    pub users: UserRepository,
    pub services: ServiceRepository,
    pub transactions: TransactionRepository,
}

impl RepositoryManager {
    /// Create new repository manager
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            users: UserRepository::new(pool.clone()),
            services: ServiceRepository::new(pool.clone()),
            transactions: TransactionRepository::new(pool.clone()),
            pool,
        }
    }
    
    /// Get database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
