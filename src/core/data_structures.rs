use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// === YAML MANIFEST SYSTEM ===

/// Complete service manifest structure matching the YAML specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub author: Author,
    pub container: ContainerDefinition,
    pub api: ApiDefinition,
    pub pricing: PricingModel,
    pub sla: ServiceLevel,
    pub reputation: Option<ReputationInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: Option<String>,
    pub did: String,
    pub contact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerDefinition {
    pub image: String,
    pub ports: Vec<u16>,
    pub env: HashMap<String, String>,
    pub resources: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu: String,          // e.g., "2000m"
    pub memory: String,       // e.g., "4Gi"
    pub gpu: Option<String>,  // e.g., "1x nvidia-tesla-t4"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDefinition {
    pub openapi: String,
    pub base_path: Option<String>,
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub path: String,
    pub method: String,
    pub description: String,
    pub pricing: EndpointPricing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointPricing {
    pub model: PricingModel,
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingModel {
    #[serde(rename = "per-call")]
    PerCall,
    #[serde(rename = "subscription")]
    Subscription,
    #[serde(rename = "free")]
    Free,
    #[serde(rename = "usage-based")]
    UsageBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLevel {
    pub uptime: f64,            // percentage
    pub response_time_ms: u64,
    pub throughput_rps: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationInfo {
    pub score: f64,
    pub total_calls: u64,
    pub reviews: u64,
    pub uptime_actual: f64,
}

// === CONTAINER MANAGEMENT ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub manifest: ServiceManifest,
    pub container_id: String,
    pub status: ServiceInstanceStatus,
    pub endpoints: Vec<String>,
    pub deployed_at: u64,
    pub health_check_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceInstanceStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
    Updating,
}

// Core identifiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ServiceId(pub String);

impl ServiceId {
    pub fn new() -> Self {
        ServiceId(uuid::Uuid::new_v4().to_string())
    }
}

/// User representation in the DuxNet system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub wallet_address: Option<String>,
    pub public_key: Option<String>,
    pub reputation_score: f64,
    pub total_earnings: i64,
    pub total_spent: i64,
    pub service_count: i32,
    pub rating: f64,
    pub metadata: serde_json::Value,
    pub is_active: bool,
    pub created_at: i64,
}

/// Service representation in the DuxNet system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub manifest: ServiceManifest,
    pub owner: String,
    pub status: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TaskId(pub String);

// Decentralized Identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DID {
    pub id: String,
    pub public_key: Vec<u8>,
    pub endpoints: Vec<String>,
    pub created_at: u64,
}

// Service metadata for DHT storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    pub id: ServiceId,
    pub provider_did: String,
    pub name: String,
    pub description: String,
    pub endpoint: String,
    pub price: u64,
    pub reputation_score: f64,
    pub last_updated: u64,
    // Enhanced fields for API store
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub sla: ServiceSLA,
    pub version: String,
    pub documentation_url: Option<String>,
    pub status: ServiceStatus,
    pub uptime_percentage: f64,
    pub response_time_ms: u64,
    pub rate_limit_per_minute: u64,
    pub supported_formats: Vec<String>,
    pub examples: Vec<ServiceExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSLA {
    pub uptime_guarantee: f64, // percentage
    pub max_response_time_ms: u64,
    pub support_response_hours: u64,
    pub refund_policy: RefundPolicy,
    pub availability_zones: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefundPolicy {
    NoRefund,
    PartialRefund { percentage: f64 },
    FullRefund,
    ConditionalRefund { conditions: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Active,
    Maintenance,
    Deprecated,
    Beta,
    Alpha,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceExample {
    pub name: String,
    pub description: String,
    pub request: String,
    pub response: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceReview {
    pub id: String,
    pub service_id: String,
    pub reviewer_did: String,
    pub rating: u8, // 1-5 stars
    pub comment: String,
    pub timestamp: u64,
    pub helpful_votes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub service_id: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub uptime_percentage: f64,
    pub total_revenue: u64,
    pub unique_users: u64,
    pub last_updated: u64,
}

// Reputation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationAttestation {
    pub attester_did: String,
    pub target_did: String,
    pub score: f64,
    pub interaction_type: String,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

// Escrow system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowContract {
    pub id: String,
    pub buyer_did: String,
    pub seller_did: String,
    pub arbiters: Vec<String>,
    pub amount: u64,
    pub state: EscrowState,
    pub multisig_address: String,
    pub signatures: HashMap<String, Vec<u8>>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EscrowState {
    Created,
    Funded,
    InProgress,
    Completed,
    Disputed,
    Refunded,
}

// Task system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub escrow_id: String,
    pub service_id: ServiceId,
    pub payload: Vec<u8>,
    pub requirements: TaskRequirements,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequirements {
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub timeout_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub processor_did: String,
    pub result: Vec<u8>,
    pub proof: Vec<u8>,
    pub completed_at: u64,
}

// Messaging system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from_did: String,
    pub to_did: String,
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: u64,
    pub signature: Vec<u8>,
    pub is_read: bool,
    pub reply_to: Option<String>, // ID of message being replied to
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Text,
    File,
    ServiceRequest,
    TaskUpdate,
    EscrowUpdate,
    ReputationUpdate,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRequest {
    pub to_did: String,
    pub content: String,
    pub message_type: MessageType,
    pub reply_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub message_id: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub peer_did: String,
    pub last_message: Option<Message>,
    pub unread_count: usize,
    pub message_count: usize,
}

// Network messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    // Service discovery
    ServiceAnnouncement(ServiceMetadata),
    ServiceQuery(String),
    ServiceResponse(Vec<ServiceMetadata>),
    
    // Task management
    TaskSubmission(Task),
    TaskAcceptance(TaskId, String), // task_id, processor_did
    TaskCompletion(TaskResult),
    
    // Escrow management
    EscrowCreation(EscrowContract),
    EscrowSignature(String, String, Vec<u8>), // escrow_id, signer_did, signature
    EscrowStateUpdate(String, EscrowState),
    
    // Reputation
    ReputationAttestation(ReputationAttestation),
    ReputationQuery(String), // target_did
    ReputationResponse(String, f64), // target_did, score
    
    // Messaging
    DirectMessage(Message),
    MessageAck(String), // message_id
    MessageDelivery(String), // message_id
    
    // P2P ping/pong
    Ping,
    Pong,
}

// Enhanced API request/response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterServiceRequest {
    pub name: String,
    pub description: String,
    pub price: u64,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub sla: ServiceSLA,
    pub version: String,
    pub documentation_url: Option<String>,
    pub rate_limit_per_minute: u64,
    pub supported_formats: Vec<String>,
    pub examples: Vec<ServiceExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterServiceResponse {
    pub service_id: String,
    pub success: bool,
    pub message: String,
    pub api_key: Option<String>, // API key for the service
    pub documentation_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindServicesRequest {
    pub query: String,
    pub categories: Option<Vec<String>>,
    pub min_rating: Option<f64>,
    pub max_price: Option<u64>,
    pub status: Option<ServiceStatus>,
    pub sort_by: Option<ServiceSortBy>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceSortBy {
    Name,
    Price,
    Rating,
    Uptime,
    ResponseTime,
    Popularity,
    Newest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindServicesResponse {
    pub services: Vec<ServiceMetadata>,
    pub total_count: u64,
    pub success: bool,
    pub message: String,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub current_page: u32,
    pub total_pages: u32,
    pub items_per_page: u32,
    pub has_next: bool,
    pub has_previous: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTaskRequest {
    pub service_id: String,
    pub payload: String,
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub timeout_seconds: u32,
    pub priority: TaskPriority,
    pub callback_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEscrowRequest {
    pub service_id: String,
    pub seller_did: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEscrowResponse {
    pub escrow_id: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAOIKeyRequest {
    pub service_id: String,
    pub key_data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAOIKeyResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAOIKeyRequest {
    pub service_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAOIKeyResponse {
    pub key_data: Option<String>,
    pub success: bool,
    pub message: String,
}

// Node status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node_id: String,
    pub did: String,
    pub is_online: bool,
    pub uptime_seconds: u64,
    pub services_count: usize,
    pub reputation_score: f64,
    pub peers_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AOIKey {
    pub service_id: ServiceId,
    pub key_data: String, // or Vec<u8> if binary
    pub created_at: u64,
}

// Community Fund System
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityFund {
    pub currency: crate::wallet::Currency,
    pub balance: u64,
    pub last_distribution: u64,
    pub total_distributed: u64,
    pub distribution_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityFundBalance {
    pub currency: String,
    pub balance: u64,
    pub formatted_balance: String,
    pub last_distribution: u64,
    pub next_distribution: u64,
    pub total_distributed: u64,
    pub distribution_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityFundDistribution {
    pub currency: crate::wallet::Currency,
    pub amount_per_user: u64,
    pub total_users: usize,
    pub distribution_timestamp: u64,
    pub transaction_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityFundStats {
    pub total_balance_usd: f64,
    pub currencies: Vec<CommunityFundBalance>,
    pub next_distribution_in: u64, // seconds until next distribution
    pub total_distributed_all_time: u64,
}

// Utility functions
pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// === MARKETPLACE SEARCH & DISCOVERY ===

/// Search filters for marketplace discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub min_reputation: Option<f64>,
    pub max_price: Option<f64>,
    pub author: Option<String>,
}

/// Sort options for marketplace results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOption {
    Relevance,
    Newest,
    Oldest,
    MostPopular,
    HighestRated,
    LowestPrice,
    HighestPrice,
}

/// Search request structure for marketplace API
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub sort_by: SortOption,
    pub limit: u32,
    pub offset: u32,
    pub filters: Option<SearchFilters>,
}

/// Search response structure for marketplace API
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub services: Vec<ServiceManifest>,
    pub total: usize,
    pub query: String,
    pub filters_applied: Option<SearchFilters>,
    pub sort_by: SortOption,
    pub page: u32,
    pub total_pages: u32,
}

// === PHASE 5: Advanced Analytics & Monitoring ===

/// Real-time analytics data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSnapshot {
    pub timestamp: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub active_services: usize,
    pub total_revenue: f64,
    pub network_health: NetworkHealth,
    pub top_services: Vec<TopServiceMetric>,
}

/// Network health indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealth {
    pub peer_count: usize,
    pub dht_entries: usize,
    pub uptime_percentage: f64,
    pub average_latency: f64,
    pub error_rate: f64,
}

/// Top service performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopServiceMetric {
    pub service_id: String,
    pub service_name: String,
    pub request_count: u64,
    pub revenue: f64,
    pub average_response_time: f64,
    pub success_rate: f64,
}

/// Service performance monitoring data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePerformanceMetrics {
    pub service_id: String,
    pub timestamp: u64,
    pub response_time: u64,
    pub status_code: u16,
    pub bytes_transferred: u64,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

/// Time-series data point for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesDataPoint {
    pub timestamp: u64,
    pub value: f64,
    pub metric_type: MetricType,
}

/// Types of metrics we track
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricType {
    RequestCount,
    ResponseTime,
    ErrorRate,
    Revenue,
    ActiveUsers,
    Bandwidth,
    DiskUsage,
    MemoryUsage,
    CpuUsage,
    NetworkIO,
}

/// Analytics query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsQuery {
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub service_id: Option<String>,
    pub metric_type: Option<MetricType>,
    pub granularity: TimeGranularity,
    pub limit: Option<usize>,
}

/// Time granularity for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeGranularity {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub metric_type: MetricType,
    pub threshold: f64,
    pub comparison: AlertComparison,
    pub duration_seconds: u64,
    pub enabled: bool,
    pub notification_channels: Vec<NotificationChannel>,
}

/// Alert comparison operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertComparison {
    GreaterThan,
    LessThan,
    Equals,
    NotEquals,
}

/// Notification channels for alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email(String),
    Webhook(String),
    Console,
}

/// Active alert instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub triggered_at: u64,
    pub resolved_at: Option<u64>,
    pub message: String,
    pub severity: AlertSeverity,
    pub current_value: f64,
    pub threshold: f64,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub id: String,
    pub name: String,
    pub widgets: Vec<DashboardWidget>,
    pub refresh_interval_seconds: u64,
}

/// Dashboard widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: String,
    pub widget_type: WidgetType,
    pub title: String,
    pub query: AnalyticsQuery,
    pub position: WidgetPosition,
    pub size: WidgetSize,
}

/// Types of dashboard widgets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WidgetType {
    LineChart,
    BarChart,
    PieChart,
    Counter,
    Table,
    Heatmap,
}

/// Widget position on dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
}

/// Widget size on dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: u32,
    pub height: u32,
} 