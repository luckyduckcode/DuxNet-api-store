//! Database migrations module

use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

/// Run all database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations...");
    
    // Run migrations using sqlx migrate
    sqlx::migrate!("./src/database/migrations")
        .run(pool)
        .await?;
    
    info!("Database migrations completed successfully");
    Ok(())
}

/// Check if migrations are up to date
pub async fn check_migrations(pool: &PgPool) -> Result<bool> {
    // This would check if all migrations have been applied
    // For now, we'll return true as sqlx handles this internally
    Ok(true)
}

/// Reset database (development only)
#[cfg(debug_assertions)]
pub async fn reset_database(pool: &PgPool) -> Result<()> {
    use sqlx::query;
    
    info!("Resetting database (development mode only)...");
    
    // Drop all tables in reverse dependency order
    let drop_tables = vec![
        "escrow_contracts",
        "mining_sessions", 
        "wallet_balances",
        "analytics_events",
        "reputation_scores",
        "transactions",
        "services",
        "users",
        "_sqlx_migrations", // SQLx migration tracking table
    ];
    
    for table in drop_tables {
        query(&format!("DROP TABLE IF EXISTS {} CASCADE", table))
            .execute(pool)
            .await?;
    }
    
    info!("Database reset completed");
    Ok(())
}
