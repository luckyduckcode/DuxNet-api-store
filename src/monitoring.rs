use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn, error};
use crate::core::DuxNetNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub version: String,
    pub components: ComponentHealth,
    pub performance: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub api_server: bool,
    pub wallet_system: bool,
    pub messaging_system: bool,
    pub dht_network: bool,
    pub duxcoin_integration: bool,
    pub analytics_engine: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub active_connections: u64,
    pub requests_per_minute: f64,
    pub average_response_time_ms: u64,
}

pub struct HealthMonitor {
    start_time: SystemTime,
    node: std::sync::Arc<crate::core::DuxNetNode>,
}

impl HealthMonitor {
    pub fn new(node: std::sync::Arc<crate::core::DuxNetNode>) -> Self {
        Self {
            start_time: SystemTime::now(),
            node,
        }
    }

    pub async fn get_health_status(&self) -> HealthStatus {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let uptime = SystemTime::now()
            .duration_since(self.start_time)
            .unwrap()
            .as_secs();

        let components = self.check_components().await;
        let performance = self.get_performance_metrics().await;

        let overall_status = if self.is_healthy(&components) {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        };

        HealthStatus {
            status: overall_status,
            timestamp,
            uptime_seconds: uptime,
            version: env!("CARGO_PKG_VERSION").to_string(),
            components,
            performance,
        }
    }

    async fn check_components(&self) -> ComponentHealth {
        ComponentHealth {
            api_server: true, // If we're running this, API server is up
            wallet_system: self.check_wallet_system().await,
            messaging_system: self.check_messaging_system().await,
            dht_network: self.check_dht_network().await,
            duxcoin_integration: self.check_duxcoin_integration().await,
            analytics_engine: self.check_analytics_engine().await,
        }
    }

    async fn check_wallet_system(&self) -> bool {
        // Try to read wallet quickly
        match self.node.wallet.try_read() {
            Ok(_) => true,
            Err(_) => {
                warn!("Wallet system health check failed - lock contention");
                false
            }
        }
    }

    async fn check_messaging_system(&self) -> bool {
        // Try to get message stats quickly
        match tokio::time::timeout(
            std::time::Duration::from_millis(500),
            self.node.messaging_system.get_message_stats()
        ).await {
            Ok(_) => true,
            _ => {
                warn!("Messaging system health check failed");
                false
            }
        }
    }

    async fn check_dht_network(&self) -> bool {
        // Check if DHT is responsive
        match tokio::time::timeout(
            std::time::Duration::from_millis(500),
            self.node.dht.get_peers()
        ).await {
            Ok(_) => true,
            _ => {
                warn!("DHT network health check failed");
                false
            }
        }
    }

    async fn check_duxcoin_integration(&self) -> bool {
        // Quick DuxCoin connectivity test
        match tokio::time::timeout(
            std::time::Duration::from_secs(2),
            crate::api::handlers::DUXCOIN_API.get_network_info()
        ).await {
            Ok(Ok(_)) => true,
            _ => false, // Expected to fail if no DuxCoin daemon
        }
    }

    async fn check_analytics_engine(&self) -> bool {
        // Check analytics engine responsiveness
        match tokio::time::timeout(
            std::time::Duration::from_millis(500),
            self.node.analytics_engine.get_analytics_summary()
        ).await {
            Ok(Ok(_)) => true,
            _ => {
                warn!("Analytics engine health check failed");
                false
            }
        }
    }

    async fn get_performance_metrics(&self) -> PerformanceMetrics {
        let memory_usage = self.get_memory_usage().await;
        let cpu_usage = self.get_cpu_usage().await;
        
        PerformanceMetrics {
            memory_usage_mb: memory_usage,
            cpu_usage_percent: cpu_usage,
            active_connections: 0, // TODO: Track active connections
            requests_per_minute: 0.0, // TODO: Calculate from analytics
            average_response_time_ms: 0, // TODO: Calculate from analytics
        }
    }

    async fn get_memory_usage(&self) -> f64 {
        // Get memory usage in MB
        match sys_info::mem_info() {
            Ok(mem) => {
                let used = mem.total - mem.free;
                (used as f64) / 1024.0 // Convert KB to MB
            }
            Err(_) => 0.0,
        }
    }

    async fn get_cpu_usage(&self) -> f64 {
        // Simplified CPU usage - in production use proper monitoring
        0.0
    }

    fn is_healthy(&self, components: &ComponentHealth) -> bool {
        // Consider system healthy if core components are working
        components.api_server && 
        components.wallet_system && 
        components.messaging_system
        // Note: DuxCoin integration failure is acceptable if daemon not running
    }

    pub async fn log_health_summary(&self) {
        let health = self.get_health_status().await;
        
        info!(
            "🏥 Health Check - Status: {} | Uptime: {}s | Memory: {:.1}MB",
            health.status,
            health.uptime_seconds,
            health.performance.memory_usage_mb
        );

        if health.status != "healthy" {
            warn!("⚠️  System degraded - Components: {:?}", health.components);
        }
    }
}
