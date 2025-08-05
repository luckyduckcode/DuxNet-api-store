mod core;
mod api;
mod wallet;
mod frontend;
mod container;
mod network;
mod gateway;
mod config;

use anyhow::Result;
use tracing::{info, error, warn};
use tracing_subscriber;
use config::ProductionConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Load production configuration
    let config = match ProductionConfig::from_env() {
        Ok(config) => {
            // Validate production configuration
            if let Err(e) = config.validate_production() {
                error!("Production configuration validation failed: {}", e);
                std::process::exit(1);
            }
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };
    
    // Initialize logging with appropriate level
    let log_level = config.log_level();
    std::env::set_var("RUST_LOG", format!("{}=debug,duxnet={}", log_level, log_level));
    tracing_subscriber::fmt::init();
    
    info!("Starting DuxNet API Store");
    info!("Environment: {}", config.environment);
    info!("Server: {}:{}", config.host, config.port);
    
    if config.environment == "production" {
        info!("🚀 Running in PRODUCTION mode");
        if !config.tls_configured() {
            warn!("⚠️ TLS not configured - consider enabling HTTPS for production");
        }
        if config.metrics_enabled {
            info!("📊 Metrics collection enabled");
        }
        if config.backup_enabled {
            info!("💾 Backup system enabled (interval: {}s)", config.backup_interval);
        }
    } else {
        warn!("🚧 Running in DEVELOPMENT mode");
    }
    
    // Create and start the DuxNet node
    let node = core::DuxNetNode::new().await?;
    
    // Start background tasks if enabled
    let mut tasks = Vec::new();
    
    if config.backup_enabled {
        let backup_node = node.clone();
        let backup_interval = config.backup_interval;
        let backup_dir = config.backup_directory.clone();
        
        tasks.push(tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(backup_interval));
            loop {
                interval.tick().await;
                if let Err(e) = perform_backup(&backup_node, &backup_dir).await {
                    error!("Backup failed: {}", e);
                }
            }
        }));
    }
    
    if config.metrics_enabled {
        let metrics_node = node.clone();
        tasks.push(tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = collect_metrics(&metrics_node).await {
                    error!("Metrics collection failed: {}", e);
                }
            }
        }));
    }
    
    // Start the web API server
    let api_config = config.clone();
    let api_handle = tokio::spawn(async move {
        if let Err(e) = api::start_api_server(api_config.port, node).await {
            error!("API server error: {}", e);
        }
    });
    
    info!("DuxNet API Store started successfully!");
    info!("Web API available at: http://{}:{}", config.host, config.port);
    info!("🔑 Demo API Keys: demo-api-key-123, admin-api-key-456, service-api-key-789");
    info!("📊 Analytics available at: http://{}:{}/api/analytics/usage", config.host, config.port);
    info!("👨‍💻 Developer portal at: http://{}:{}/api/developer/dashboard", config.host, config.port);
    
    // Wait for the API server
    api_handle.await?;
    
    Ok(())
}

async fn perform_backup(node: &core::DuxNetNode, backup_dir: &str) -> Result<()> {
    use std::fs;
    use std::path::Path;
    
    // Create backup directory if it doesn't exist
    fs::create_dir_all(backup_dir)?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    
    // Backup wallet
    let wallet = node.wallet.read().await;
    if let Ok(wallet_backup) = wallet.backup_wallet() {
        let wallet_backup_path = format!("{}/wallet_backup_{}.json", backup_dir, timestamp);
        fs::write(&wallet_backup_path, wallet_backup)?;
        info!("💾 Wallet backed up to: {}", wallet_backup_path);
    }
    
    // Backup DID
    let did_backup = serde_json::to_string_pretty(&node.did_manager)?;
    let did_backup_path = format!("{}/did_backup_{}.json", backup_dir, timestamp);
    fs::write(&did_backup_path, did_backup)?;
    info!("💾 DID backed up to: {}", did_backup_path);
    
    // Clean old backups (keep last 10)
    clean_old_backups(backup_dir, 10)?;
    
    Ok(())
}

async fn collect_metrics(node: &core::DuxNetNode) -> Result<()> {
    // Record system metrics
    let analytics = &node.analytics_engine;
    
    // Memory usage
    let memory_usage = get_memory_usage()?;
    analytics.record_metric(core::data_structures::MetricType::MemoryUsage, memory_usage).await?;
    
    // CPU usage would require additional system monitoring libraries
    // For now, we'll just record a placeholder
    analytics.record_metric(core::data_structures::MetricType::CpuUsage, 0.0).await?;
    
    info!("📊 Metrics collected - Memory: {:.2}%", memory_usage);
    
    Ok(())
}

fn get_memory_usage() -> Result<f64> {
    // This is a simplified memory usage calculation
    // In production, you'd use a proper system monitoring library
    use std::fs;
    
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                if let Some(value_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = value_str.parse::<f64>() {
                        // Convert KB to percentage (assuming 8GB total memory)
                        return Ok((kb / (8.0 * 1024.0 * 1024.0)) * 100.0);
                    }
                }
            }
        }
    }
    
    Ok(0.0) // Default if we can't read memory usage
}

fn clean_old_backups(backup_dir: &str, keep_count: usize) -> Result<()> {
    use std::fs;
    use std::path::Path;
    
    let mut backup_files: Vec<_> = fs::read_dir(backup_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_name().to_string_lossy().contains("backup_")
        })
        .collect();
    
    // Sort by modification time (newest first)
    backup_files.sort_by(|a, b| {
        let a_modified = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let b_modified = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        b_modified.cmp(&a_modified)
    });
    
    // Remove old backups
    for old_backup in backup_files.iter().skip(keep_count) {
        if let Err(e) = fs::remove_file(old_backup.path()) {
            warn!("Failed to remove old backup {:?}: {}", old_backup.path(), e);
        } else {
            info!("🗑️ Removed old backup: {:?}", old_backup.path());
        }
    }
    
    Ok(())
} 