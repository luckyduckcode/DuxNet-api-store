use std::env;

/// Production configuration for DuxNet
#[derive(Debug, Clone)]
pub struct ProductionConfig {
    // Server configuration
    pub host: String,
    pub port: u16,
    pub environment: String,
    
    // DuxCoin configuration
    pub duxcoin_rpc_url: String,
    pub duxcoin_rpc_user: String,
    pub duxcoin_rpc_password: String,
    pub duxcoin_wallet_passphrase: Option<String>,
    
    // Security configuration
    pub api_key_salt: String,
    pub jwt_secret: String,
    pub cors_origins: Vec<String>,
    
    // P2P configuration
    pub p2p_port: u16,
    pub bootstrap_nodes: Vec<String>,
    pub dht_replication_factor: usize,
    
    // Rate limiting
    pub default_rate_limit: u64,
    pub default_window_seconds: u64,
    pub burst_rate_limit: u64,
    
    // Community fund
    pub community_tax_rate: f64,
    pub distribution_interval: u64,
    
    // Monitoring
    pub metrics_enabled: bool,
    pub prometheus_port: Option<u16>,
    pub health_check_interval: u64,
    
    // Backup
    pub backup_enabled: bool,
    pub backup_interval: u64,
    pub backup_directory: String,
}

impl ProductionConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            // Server configuration
            host: env::var("DUXNET_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("DUXNET_PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()
                .map_err(|_| "Invalid DUXNET_PORT")?,
            environment: env::var("DUXNET_ENV").unwrap_or_else(|_| "development".to_string()),
            
            // DuxCoin configuration
            duxcoin_rpc_url: env::var("DUXCOIN_RPC_URL")
                .unwrap_or_else(|_| "http://localhost:8332".to_string()),
            duxcoin_rpc_user: env::var("DUXCOIN_RPC_USER")
                .unwrap_or_else(|_| "duxnetuser".to_string()),
            duxcoin_rpc_password: env::var("DUXCOIN_RPC_PASSWORD")
                .map_err(|_| "DUXCOIN_RPC_PASSWORD is required")?,
            duxcoin_wallet_passphrase: env::var("DUXCOIN_WALLET_PASSPHRASE").ok(),
            
            // Security configuration
            api_key_salt: env::var("API_KEY_SALT")
                .map_err(|_| "API_KEY_SALT is required for production")?,
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|_| "JWT_SECRET is required for production")?,
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            
            // P2P configuration
            p2p_port: env::var("P2P_PORT")
                .unwrap_or_else(|_| "9000".to_string())
                .parse()
                .map_err(|_| "Invalid P2P_PORT")?,
            bootstrap_nodes: env::var("P2P_BOOTSTRAP_NODES")
                .unwrap_or_default()
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
                .collect(),
            dht_replication_factor: env::var("DHT_REPLICATION_FACTOR")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .map_err(|_| "Invalid DHT_REPLICATION_FACTOR")?,
            
            // Rate limiting
            default_rate_limit: env::var("DEFAULT_RATE_LIMIT")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .map_err(|_| "Invalid DEFAULT_RATE_LIMIT")?,
            default_window_seconds: env::var("DEFAULT_WINDOW_SECONDS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .map_err(|_| "Invalid DEFAULT_WINDOW_SECONDS")?,
            burst_rate_limit: env::var("BURST_RATE_LIMIT")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .map_err(|_| "Invalid BURST_RATE_LIMIT")?,
            
            // Community fund
            community_tax_rate: env::var("COMMUNITY_TAX_RATE")
                .unwrap_or_else(|_| "0.02".to_string())
                .parse()
                .map_err(|_| "Invalid COMMUNITY_TAX_RATE")?,
            distribution_interval: env::var("DISTRIBUTION_INTERVAL")
                .unwrap_or_else(|_| "43200".to_string())
                .parse()
                .map_err(|_| "Invalid DISTRIBUTION_INTERVAL")?,
            
            // Monitoring
            metrics_enabled: env::var("METRICS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            prometheus_port: env::var("PROMETHEUS_PORT")
                .ok()
                .and_then(|s| s.parse().ok()),
            health_check_interval: env::var("HEALTH_CHECK_INTERVAL")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .map_err(|_| "Invalid HEALTH_CHECK_INTERVAL")?,
            
            // Backup
            backup_enabled: env::var("BACKUP_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            backup_interval: env::var("BACKUP_INTERVAL")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .map_err(|_| "Invalid BACKUP_INTERVAL")?,
            backup_directory: env::var("BACKUP_DIRECTORY")
                .unwrap_or_else(|_| "/var/backups/duxnet".to_string()),
        })
    }
    
    /// Validate configuration for production environment
    pub fn validate_production(&self) -> Result<(), String> {
        if self.environment == "production" {
            // Security checks
            if self.api_key_salt.len() < 32 {
                return Err("API_KEY_SALT must be at least 32 characters in production".to_string());
            }
            
            if self.jwt_secret.len() < 32 {
                return Err("JWT_SECRET must be at least 32 characters in production".to_string());
            }
            
            if self.duxcoin_rpc_password.len() < 16 {
                return Err("DUXCOIN_RPC_PASSWORD must be at least 16 characters in production".to_string());
            }
            
            // Network checks
            if self.host == "0.0.0.0" && !self.cors_origins.iter().any(|origin| origin != "*") {
                return Err("CORS_ORIGINS must be properly configured when binding to 0.0.0.0".to_string());
            }
        }
        
        Ok(())
    }
    
    /// Get log level based on environment
    pub fn log_level(&self) -> String {
        match self.environment.as_str() {
            "production" => "info".to_string(),
            "development" => "debug".to_string(),
            _ => "warn".to_string(),
        }
    }
    
    /// Check if TLS is configured
    pub fn tls_configured(&self) -> bool {
        env::var("TLS_CERT_PATH").is_ok() && env::var("TLS_KEY_PATH").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_config_from_env() {
        // Set test environment variables
        env::set_var("DUXCOIN_RPC_PASSWORD", "test_password_12345");
        env::set_var("API_KEY_SALT", "test_salt_32_characters_minimum");
        env::set_var("JWT_SECRET", "test_jwt_secret_32_characters_minimum");
        
        let config = ProductionConfig::from_env().expect("Should load config from env");
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8081);
        assert_eq!(config.duxcoin_rpc_password, "test_password_12345");
    }
    
    #[test]
    fn test_production_validation() {
        env::set_var("DUXNET_ENV", "production");
        env::set_var("DUXCOIN_RPC_PASSWORD", "production_password_123");
        env::set_var("API_KEY_SALT", "production_salt_32_characters_min");
        env::set_var("JWT_SECRET", "production_jwt_secret_32_chars_min");
        
        let config = ProductionConfig::from_env().expect("Should load config");
        config.validate_production().expect("Should pass production validation");
    }
}
