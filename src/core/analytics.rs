use crate::core::data_structures::*;
use anyhow::Result;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use serde_json::json;

/// Phase 5: Advanced Analytics & Monitoring Engine
/// 
/// This module provides comprehensive analytics, monitoring, and alerting
/// capabilities for the DuxNet platform including:
/// - Real-time metrics collection and aggregation
/// - Time-series data storage and querying
/// - Alert system with configurable rules
/// - Dashboard management and widgets
/// - Performance monitoring and health tracking

#[derive(Debug)]
pub struct AnalyticsEngine {
    /// In-memory time-series storage for recent metrics
    metrics_store: Arc<RwLock<HashMap<String, VecDeque<TimeSeriesDataPoint>>>>,
    
    /// Service performance metrics
    service_metrics: Arc<RwLock<HashMap<String, VecDeque<ServicePerformanceMetrics>>>>,
    
    /// Active alert rules
    alert_rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    
    /// Currently active alerts
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    
    /// Dashboard configurations
    dashboards: Arc<RwLock<HashMap<String, DashboardConfig>>>,
    
    /// Real-time analytics snapshot cache
    current_snapshot: Arc<RwLock<Option<AnalyticsSnapshot>>>,
    
    /// Maximum number of data points to keep in memory per metric
    max_data_points: usize,
}

impl AnalyticsEngine {
    /// Create a new analytics engine instance
    pub fn new() -> Self {
        Self {
            metrics_store: Arc::new(RwLock::new(HashMap::new())),
            service_metrics: Arc::new(RwLock::new(HashMap::new())),
            alert_rules: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            dashboards: Arc::new(RwLock::new(HashMap::new())),
            current_snapshot: Arc::new(RwLock::new(None)),
            max_data_points: 10000, // Keep last 10k data points per metric
        }
    }

    /// Record a new metric data point
    pub async fn record_metric(&self, metric_type: MetricType, value: f64) -> Result<()> {
        let timestamp = get_current_timestamp();
        let data_point = TimeSeriesDataPoint {
            timestamp,
            value,
            metric_type: metric_type.clone(),
        };

        let metric_key = format!("{:?}", metric_type);
        let mut store = self.metrics_store.write().await;
        
        let series = store.entry(metric_key.clone()).or_insert_with(VecDeque::new);
        series.push_back(data_point);
        
        // Keep only the most recent data points
        while series.len() > self.max_data_points {
            series.pop_front();
        }

        info!("Recorded metric: {} = {}", metric_key, value);
        
        // Check for alert conditions
        self.check_alerts(&metric_type, value).await?;
        
        Ok(())
    }

    /// Record service performance metrics
    pub async fn record_service_metrics(&self, metrics: ServicePerformanceMetrics) -> Result<()> {
        let mut store = self.service_metrics.write().await;
        let series = store.entry(metrics.service_id.clone()).or_insert_with(VecDeque::new);
        
        series.push_back(metrics.clone());
        
        // Keep only recent metrics per service
        while series.len() > 1000 {
            series.pop_front();
        }

        // Derive aggregate metrics
        let response_time = metrics.response_time as f64;
        self.record_metric(MetricType::ResponseTime, response_time).await?;
        
        if metrics.status_code >= 200 && metrics.status_code < 400 {
            self.record_metric(MetricType::RequestCount, 1.0).await?;
        } else {
            self.record_metric(MetricType::ErrorRate, 1.0).await?;
        }

        Ok(())
    }

    /// Get metrics for a specific time range
    pub async fn get_metrics(
        &self,
        query: AnalyticsQuery,
    ) -> Result<Vec<TimeSeriesDataPoint>> {
        let store = self.metrics_store.read().await;
        let mut results = Vec::new();

        for (metric_key, series) in store.iter() {
            // Filter by metric type if specified
            if let Some(ref metric_type) = query.metric_type {
                if metric_key != &format!("{:?}", metric_type) {
                    continue;
                }
            }

            for point in series.iter() {
                // Filter by time range
                if let Some(start_time) = query.start_time {
                    if point.timestamp < start_time {
                        continue;
                    }
                }
                if let Some(end_time) = query.end_time {
                    if point.timestamp > end_time {
                        continue;
                    }
                }

                results.push(point.clone());
            }
        }

        // Sort by timestamp
        results.sort_by_key(|p| p.timestamp);

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Generate real-time analytics snapshot
    pub async fn generate_snapshot(&self) -> Result<AnalyticsSnapshot> {
        let current_time = get_current_timestamp();
        let metrics_store = self.metrics_store.read().await;
        let service_metrics = self.service_metrics.read().await;

        // Calculate aggregate metrics from recent data
        let mut total_requests = 0u64;
        let mut successful_requests = 0u64;
        let mut failed_requests = 0u64;
        let mut response_times = Vec::new();

        // Analyze service metrics from last hour
        let one_hour_ago = current_time - 3600;
        
        for series in service_metrics.values() {
            for metric in series.iter() {
                if metric.timestamp > one_hour_ago {
                    total_requests += 1;
                    
                    if metric.status_code >= 200 && metric.status_code < 400 {
                        successful_requests += 1;
                    } else {
                        failed_requests += 1;
                    }
                    
                    response_times.push(metric.response_time as f64);
                }
            }
        }

        let average_response_time = if !response_times.is_empty() {
            response_times.iter().sum::<f64>() / response_times.len() as f64
        } else {
            0.0
        };

        // Generate top services metrics
        let top_services = self.calculate_top_services().await?;

        // Calculate network health
        let network_health = NetworkHealth {
            peer_count: 0, // TODO: Get from DHT
            dht_entries: 0, // TODO: Get from DHT
            uptime_percentage: 99.5, // TODO: Calculate actual uptime
            average_latency: average_response_time,
            error_rate: if total_requests > 0 {
                (failed_requests as f64 / total_requests as f64) * 100.0
            } else {
                0.0
            },
        };

        let snapshot = AnalyticsSnapshot {
            timestamp: current_time,
            total_requests,
            successful_requests,
            failed_requests,
            average_response_time,
            active_services: service_metrics.len(),
            total_revenue: 0.0, // TODO: Calculate from escrow transactions
            network_health,
            top_services,
        };

        // Cache the snapshot
        *self.current_snapshot.write().await = Some(snapshot.clone());

        Ok(snapshot)
    }

    /// Calculate top performing services
    async fn calculate_top_services(&self) -> Result<Vec<TopServiceMetric>> {
        let service_metrics = self.service_metrics.read().await;
        let mut service_stats: HashMap<String, (u64, f64, u64, u64)> = HashMap::new();

        let one_hour_ago = get_current_timestamp() - 3600;

        for (service_id, series) in service_metrics.iter() {
            let mut request_count = 0u64;
            let mut total_response_time = 0f64;
            let mut successful_requests = 0u64;
            let mut total_requests = 0u64;

            for metric in series.iter() {
                if metric.timestamp > one_hour_ago {
                    request_count += 1;
                    total_response_time += metric.response_time as f64;
                    total_requests += 1;
                    
                    if metric.status_code >= 200 && metric.status_code < 400 {
                        successful_requests += 1;
                    }
                }
            }

            if request_count > 0 {
                service_stats.insert(
                    service_id.clone(),
                    (request_count, total_response_time, successful_requests, total_requests),
                );
            }
        }

        let mut top_services = Vec::new();
        
        for (service_id, (request_count, total_response_time, successful_requests, total_requests)) in service_stats {
            let average_response_time = if request_count > 0 {
                total_response_time / request_count as f64
            } else {
                0.0
            };
            
            let success_rate = if total_requests > 0 {
                (successful_requests as f64 / total_requests as f64) * 100.0
            } else {
                0.0
            };

            top_services.push(TopServiceMetric {
                service_id: service_id.clone(),
                service_name: service_id, // TODO: Get actual name from service registry
                request_count,
                revenue: 0.0, // TODO: Calculate from pricing and usage
                average_response_time,
                success_rate,
            });
        }

        // Sort by request count (most active first)
        top_services.sort_by(|a, b| b.request_count.cmp(&a.request_count));
        
        // Take top 10
        top_services.truncate(10);

        Ok(top_services)
    }

    /// Add or update an alert rule
    pub async fn add_alert_rule(&self, rule: AlertRule) -> Result<()> {
        let mut rules = self.alert_rules.write().await;
        info!("Adding alert rule: {} for metric {:?}", rule.name, rule.metric_type);
        rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Check for alert conditions
    async fn check_alerts(&self, metric_type: &MetricType, current_value: f64) -> Result<()> {
        let rules = self.alert_rules.read().await;
        let mut alerts = self.active_alerts.write().await;

        for rule in rules.values() {
            if !rule.enabled || format!("{:?}", rule.metric_type) != format!("{:?}", metric_type) {
                continue;
            }

            let should_trigger = match rule.comparison {
                AlertComparison::GreaterThan => current_value > rule.threshold,
                AlertComparison::LessThan => current_value < rule.threshold,
                AlertComparison::Equals => (current_value - rule.threshold).abs() < f64::EPSILON,
                AlertComparison::NotEquals => (current_value - rule.threshold).abs() > f64::EPSILON,
            };

            if should_trigger {
                let alert_id = format!("{}_{}", rule.id, get_current_timestamp());
                
                if !alerts.contains_key(&alert_id) {
                    let alert = Alert {
                        id: alert_id.clone(),
                        rule_id: rule.id.clone(),
                        triggered_at: get_current_timestamp(),
                        resolved_at: None,
                        message: format!(
                            "Alert: {} - {} is {} {} (threshold: {})",
                            rule.name,
                            format!("{:?}", metric_type),
                            current_value,
                            match rule.comparison {
                                AlertComparison::GreaterThan => "above",
                                AlertComparison::LessThan => "below",
                                AlertComparison::Equals => "equal to",
                                AlertComparison::NotEquals => "not equal to",
                            },
                            rule.threshold
                        ),
                        severity: if current_value > rule.threshold * 2.0 {
                            AlertSeverity::Critical
                        } else if current_value > rule.threshold * 1.5 {
                            AlertSeverity::Error
                        } else {
                            AlertSeverity::Warning
                        },
                        current_value,
                        threshold: rule.threshold,
                    };

                    warn!("🚨 Alert triggered: {}", alert.message);
                    alerts.insert(alert_id, alert);

                    // TODO: Send notifications via configured channels
                }
            }
        }

        Ok(())
    }

    /// Get current active alerts
    pub async fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        let alerts = self.active_alerts.read().await;
        Ok(alerts.values().cloned().collect())
    }

    /// Get analytics summary for dashboard
    pub async fn get_analytics_summary(&self) -> Result<serde_json::Value> {
        let snapshot = match self.current_snapshot.read().await.as_ref() {
            Some(snapshot) => snapshot.clone(),
            None => self.generate_snapshot().await?,
        };

        Ok(json!({
            "timestamp": snapshot.timestamp,
            "overview": {
                "total_requests": snapshot.total_requests,
                "success_rate": if snapshot.total_requests > 0 {
                    (snapshot.successful_requests as f64 / snapshot.total_requests as f64) * 100.0
                } else { 100.0 },
                "average_response_time": snapshot.average_response_time,
                "active_services": snapshot.active_services,
                "error_rate": snapshot.network_health.error_rate
            },
            "network_health": snapshot.network_health,
            "top_services": snapshot.top_services,
            "alerts": self.get_active_alerts().await?
        }))
    }

    /// Create a default monitoring dashboard
    pub async fn create_default_dashboard(&self) -> Result<DashboardConfig> {
        let dashboard = DashboardConfig {
            id: "default".to_string(),
            name: "DuxNet Analytics Dashboard".to_string(),
            refresh_interval_seconds: 30,
            widgets: vec![
                DashboardWidget {
                    id: "requests_chart".to_string(),
                    widget_type: WidgetType::LineChart,
                    title: "Request Volume".to_string(),
                    query: AnalyticsQuery {
                        start_time: Some(get_current_timestamp() - 3600),
                        end_time: None,
                        service_id: None,
                        metric_type: Some(MetricType::RequestCount),
                        granularity: TimeGranularity::Minute,
                        limit: Some(60),
                    },
                    position: WidgetPosition { x: 0, y: 0 },
                    size: WidgetSize { width: 6, height: 4 },
                },
                DashboardWidget {
                    id: "response_time_chart".to_string(),
                    widget_type: WidgetType::LineChart,
                    title: "Response Time".to_string(),
                    query: AnalyticsQuery {
                        start_time: Some(get_current_timestamp() - 3600),
                        end_time: None,
                        service_id: None,
                        metric_type: Some(MetricType::ResponseTime),
                        granularity: TimeGranularity::Minute,
                        limit: Some(60),
                    },
                    position: WidgetPosition { x: 6, y: 0 },
                    size: WidgetSize { width: 6, height: 4 },
                },
                DashboardWidget {
                    id: "error_rate_counter".to_string(),
                    widget_type: WidgetType::Counter,
                    title: "Error Rate".to_string(),
                    query: AnalyticsQuery {
                        start_time: Some(get_current_timestamp() - 3600),
                        end_time: None,
                        service_id: None,
                        metric_type: Some(MetricType::ErrorRate),
                        granularity: TimeGranularity::Hour,
                        limit: Some(1),
                    },
                    position: WidgetPosition { x: 0, y: 4 },
                    size: WidgetSize { width: 3, height: 2 },
                },
            ],
        };

        let mut dashboards = self.dashboards.write().await;
        dashboards.insert(dashboard.id.clone(), dashboard.clone());

        Ok(dashboard)
    }
}

/// Helper function to get current timestamp
fn get_current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl Default for AnalyticsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    // Helper function to create test analytics engine
    fn create_test_engine() -> AnalyticsEngine {
        AnalyticsEngine::new()
    }

    // Helper function to create test service metrics
    fn create_test_service_metrics(service_id: &str, response_time: u64, status_code: u16) -> ServicePerformanceMetrics {
        ServicePerformanceMetrics {
            service_id: service_id.to_string(),
            timestamp: super::get_current_timestamp(),
            response_time,
            status_code,
            bytes_transferred: 1024,
            user_agent: Some("test-agent".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
        }
    }

    #[tokio::test]
    async fn test_analytics_engine_creation() {
        let engine = create_test_engine();
        
        // Verify initial state
        let snapshot = engine.generate_snapshot().await.unwrap();
        assert_eq!(snapshot.total_requests, 0);
        assert_eq!(snapshot.successful_requests, 0);
        assert_eq!(snapshot.failed_requests, 0);
        assert_eq!(snapshot.active_services, 0);
    }

    #[tokio::test]
    async fn test_metric_recording() {
        let engine = create_test_engine();
        
        // Record a metric
        let result = engine.record_metric(MetricType::ResponseTime, 150.0).await;
        assert!(result.is_ok());
        
        // Query the metric back
        let query = AnalyticsQuery {
            start_time: None,
            end_time: None,
            service_id: None,
            metric_type: Some(MetricType::ResponseTime),
            granularity: TimeGranularity::Minute,
            limit: Some(10),
        };
        
        let metrics = engine.get_metrics(query).await.unwrap();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].value, 150.0);
        assert_eq!(metrics[0].metric_type, MetricType::ResponseTime);
    }

    #[tokio::test]
    async fn test_service_metrics_recording() {
        let engine = create_test_engine();
        
        // Record service metrics
        let service_metrics = create_test_service_metrics("test-service", 200, 200);
        let result = engine.record_service_metrics(service_metrics).await;
        assert!(result.is_ok());
        
        // Generate snapshot to verify aggregation
        let snapshot = engine.generate_snapshot().await.unwrap();
        assert!(snapshot.total_requests > 0);
    }

    #[tokio::test]
    async fn test_alert_system() {
        let engine = create_test_engine();
        
        // Create an alert rule
        let alert_rule = AlertRule {
            id: "test-alert".to_string(),
            name: "Test Alert".to_string(),
            metric_type: MetricType::ResponseTime,
            comparison: AlertComparison::GreaterThan,
            threshold: 100.0,
            enabled: true,
            notification_channels: vec![NotificationChannel::Email("test@example.com".to_string())],
            duration_seconds: 60,
        };
        
        // Add the alert rule
        let result = engine.add_alert_rule(alert_rule).await;
        assert!(result.is_ok());
        
        // Record a metric that should trigger the alert
        let result = engine.record_metric(MetricType::ResponseTime, 150.0).await;
        assert!(result.is_ok());
        
        // Check if alert was triggered
        let alerts = engine.get_active_alerts().await.unwrap();
        assert!(!alerts.is_empty());
        assert!(alerts[0].message.contains("ResponseTime"));
    }

    #[tokio::test]
    async fn test_dashboard_creation() {
        let engine = create_test_engine();
        
        // Create default dashboard
        let dashboard = engine.create_default_dashboard().await.unwrap();
        
        // Verify dashboard structure
        assert_eq!(dashboard.id, "default");
        assert_eq!(dashboard.name, "DuxNet Analytics Dashboard");
        assert!(dashboard.refresh_interval_seconds > 0);
        assert!(!dashboard.widgets.is_empty());
        
        // Verify widget types
        let widget_types: Vec<_> = dashboard.widgets.iter()
            .map(|w| &w.widget_type)
            .collect();
        
        assert!(widget_types.contains(&&WidgetType::LineChart));
        assert!(widget_types.contains(&&WidgetType::Counter));
    }
}
