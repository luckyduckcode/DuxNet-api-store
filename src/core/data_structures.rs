use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// Core identifiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ServiceId(pub String);

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