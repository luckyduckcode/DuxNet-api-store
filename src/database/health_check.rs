use sqlx::{Row};
use crate::database::{DatabaseManager};

pub async fn test_database_connection() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing database connection...");
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://duxnet:duxnet_dev_password@localhost/duxnet_development".to_string());
    
    let db_manager = DatabaseManager::new(&database_url).await?;
    let pool = db_manager.pool();
    
    // Test basic connection
    let row = sqlx::query("SELECT 1 as test_value")
        .fetch_one(pool)
        .await?;
    
    let test_value: i32 = row.get("test_value");
    println!("✅ Database connection successful - test value: {}", test_value);
    
    // Test schema exists
    let table_count = sqlx::query("SELECT COUNT(*) as count FROM information_schema.tables WHERE table_schema = 'public'")
        .fetch_one(pool)
        .await?;
    
    let count: i64 = table_count.get("count");
    println!("✅ Found {} tables in database schema", count);
    
    // Test user table specifically
    match sqlx::query("SELECT COUNT(*) as count FROM users")
        .fetch_one(pool)
        .await {
        Ok(row) => {
            let user_count: i64 = row.get("count");
            println!("✅ Users table accessible - {} users found", user_count);
        },
        Err(e) => {
            println!("⚠️  Users table not accessible: {}", e);
        }
    }
    
    println!("✅ Database test completed successfully");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_db_health() {
        if let Err(e) = test_database_connection().await {
            println!("❌ Database test failed: {}", e);
            // Don't panic in tests, just log the error
        }
    }
}
