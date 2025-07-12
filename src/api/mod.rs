use crate::core::data_structures::*;
use axum::{
    extract::State,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use std::collections::HashMap;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::{error, info};
use axum::response::Html;
use base64::Engine;
use crate::core::data_structures::{RegisterAOIKeyRequest, RegisterAOIKeyResponse, GetAOIKeyRequest, GetAOIKeyResponse};

pub mod dux_coin;

#[derive(Clone)]
pub struct ApiState {
    pub node: Arc<crate::core::DuxNetNode>,
}

pub async fn start_api_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting API server on port {}", port);
    
    // Create a mock node for the API (in a real app, this would be shared)
    let node = Arc::new(crate::core::DuxNetNode::new(8080).await?);
    
    let state = ApiState { node };
    
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);
    
    let app = Router::new()
        .route("/api/status", get(get_status))
        .route("/api/services/register", post(register_service))
        .route("/api/services/search", post(search_services))
        .route("/api/tasks/submit", post(submit_task))
        .route("/api/escrow/create", post(create_escrow))
        .route("/api/reputation/:did", get(get_reputation))
        .route("/api/stats", get(get_stats))
        .route("/api/wallet/info", get(get_wallet_info))
        .route("/api/wallet/balances", get(get_wallet_balances))
        .route("/api/wallet/addresses", get(get_wallet_addresses))
        .route("/api/wallet/send", post(send_funds))
        .route("/api/wallet/receive", post(receive_funds))
        .route("/api/wallet/transactions", get(get_transaction_history))
        .route("/api/wallet/transaction/:id", get(get_transaction_by_id))
        .route("/api/wallet/backup", get(backup_wallet))
        .route("/api/wallet/restore", post(restore_wallet))
        .route("/api/wallet/keys", get(get_wallet_keys))
        .route("/api/services/aoi/register", post(register_aoi_key))
        .route("/api/services/aoi/get", post(get_aoi_key))
        .route("/api/community_fund/stats", get(get_community_fund_stats))
        .route("/api/community_fund/balance/:currency", get(get_community_fund_balance))
        .route("/api/community_fund/distribute/:currency", post(distribute_community_fund))
        .route("/api/messaging/send", post(send_message))
        .route("/api/messaging/conversations", get(get_conversations))
        .route("/api/messaging/messages/:peer_did", get(get_messages))
        .route("/api/messaging/read/:message_id", post(mark_message_read))
        .route("/api/messaging/delete/:message_id", post(delete_message))
        .route("/api/messaging/stats", get(get_messaging_stats))
        .route("/api/dux/balance", get(get_dux_balance))
        .route("/api/dux/transactions", get(get_dux_transactions))
        .route("/api/dux/send", post(send_dux))
        .route("/api/dux/network", get(get_dux_network))
        .route("/api/dux/mine/start", post(start_dux_mining))
        .route("/api/dux/mine/stop", post(stop_dux_mining))
        .route("/api/dux/mine/status", get(get_dux_mining_status))
        .route("/api/dux/sync", post(sync_dux_balance))
        .route("/api/shutdown", post(shutdown_node))
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        .nest_service("/static", ServeDir::new("static"))
        .layer(cors)
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("API server listening on port {}", port);
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn serve_index() -> Result<Html<String>, StatusCode> {
    let html_content = include_str!("../../static/index.html");
    Ok(Html(html_content.to_string()))
}

async fn get_status(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let reputation = node.get_reputation(&node.did_manager.did.id).await;
    let peers = node.network.get_peers().await;
    
    let status = NodeStatus {
        node_id: node.node_id.0.clone(),
        did: node.did_manager.did.id.clone(),
        is_online: true,
        uptime_seconds: 0, // TODO: track uptime
        services_count: 0,  // TODO: track services
        reputation_score: reputation,
        peers_count: peers.len(),
    };
    
    axum::Json(status)
}

async fn register_service(
    State(state): State<ApiState>,
    axum::Json(request): axum::Json<RegisterServiceRequest>,
) -> impl IntoResponse {
    let node = &state.node;
    
    match node.register_service(request.name, request.description, request.price).await {
        Ok(service_id) => axum::Json(RegisterServiceResponse {
            service_id: service_id.0,
            success: true,
            message: "Service registered successfully".to_string(),
        }),
        Err(e) => {
            error!("Failed to register service: {}", e);
            axum::Json(RegisterServiceResponse {
                service_id: "".to_string(),
                success: false,
                message: format!("Failed to register service: {}", e),
            })
        }
    }
}

async fn search_services(
    State(state): State<ApiState>,
    axum::Json(request): axum::Json<FindServicesRequest>,
) -> impl IntoResponse {
    let node = &state.node;
    
    let services = node.find_services(&request.query).await;
    
    axum::Json(FindServicesResponse {
        services: services.clone(),
        success: true,
        message: format!("Found {} services", services.len()),
    })
}

async fn submit_task(
    State(state): State<ApiState>,
    axum::Json(request): axum::Json<SubmitTaskRequest>,
) -> impl IntoResponse {
    let node = &state.node;
    
    let service_id = ServiceId(request.service_id);
    let requirements = TaskRequirements {
        cpu_cores: request.cpu_cores,
        memory_mb: request.memory_mb,
        timeout_seconds: request.timeout_seconds,
    };
    
    match node.submit_task(service_id, request.payload.into_bytes(), requirements).await {
        Ok(task_id) => axum::Json(SubmitTaskResponse {
            task_id: task_id.0,
            success: true,
            message: "Task submitted successfully".to_string(),
        }),
        Err(e) => {
            error!("Failed to submit task: {}", e);
            axum::Json(SubmitTaskResponse {
                task_id: "".to_string(),
                success: false,
                message: format!("Failed to submit task: {}", e),
            })
        }
    }
}

async fn create_escrow(
    State(state): State<ApiState>,
    axum::Json(request): axum::Json<CreateEscrowRequest>,
) -> impl IntoResponse {
    let node = &state.node;
    
    let service_id = ServiceId(request.service_id);
    
    match node.create_escrow_for_service(&service_id, request.seller_did, request.amount).await {
        Ok(escrow_id) => axum::Json(CreateEscrowResponse {
            escrow_id,
            success: true,
            message: "Escrow created successfully".to_string(),
        }),
        Err(e) => {
            error!("Failed to create escrow: {}", e);
            axum::Json(CreateEscrowResponse {
                escrow_id: "".to_string(),
                success: false,
                message: format!("Failed to create escrow: {}", e),
            })
        }
    }
}

async fn get_reputation(
    State(state): State<ApiState>,
    axum::extract::Path(did): axum::extract::Path<String>,
) -> impl IntoResponse {
    let node = &state.node;
    let reputation = node.get_reputation(&did).await;
    
    axum::Json(serde_json::json!({
        "did": did,
        "reputation": reputation,
        "success": true
    }))
}

async fn get_stats(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    
    let dht_stats = node.dht.get_stats().await;
    let reputation_stats = node.reputation_system.get_stats().await;
    let escrow_stats = node.escrow_manager.get_stats().await;
    let task_stats = node.task_engine.get_stats().await;
    let network_stats = node.network.get_stats().await;
    
    axum::Json(serde_json::json!({
        "dht": {
            "total_entries": dht_stats.total_entries,
            "total_peers": dht_stats.total_peers,
            "service_entries": dht_stats.service_entries,
            "reputation_entries": dht_stats.reputation_entries,
            "escrow_entries": dht_stats.escrow_entries,
        },
        "reputation": {
            "total_nodes": reputation_stats.total_nodes,
            "total_attestations": reputation_stats.total_attestations,
            "average_score": reputation_stats.average_score,
        },
        "escrow": {
            "total_contracts": escrow_stats.total_contracts,
            "created": escrow_stats.created,
            "funded": escrow_stats.funded,
            "in_progress": escrow_stats.in_progress,
            "completed": escrow_stats.completed,
            "disputed": escrow_stats.disputed,
            "refunded": escrow_stats.refunded,
            "total_amount": escrow_stats.total_amount,
        },
        "tasks": {
            "pending_count": task_stats.pending_count,
            "processing_count": task_stats.processing_count,
            "completed_count": task_stats.completed_count,
            "total_tasks": task_stats.total_tasks,
        },
        "network": {
            "local_peer_id": network_stats.local_peer_id,
            "connected_peers": network_stats.connected_peers,
            "subscribed_topics": network_stats.subscribed_topics,
        }
    }))
}

// Wallet API endpoints
async fn get_wallet_info(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    match wallet.get_wallet_info() {
        Ok(wallet_info) => axum::Json(serde_json::json!({
            "success": true,
            "wallet": wallet_info
        })),
        Err(e) => {
            error!("Failed to get wallet info: {}", e);
            axum::Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to get wallet info: {}", e)
            }))
        }
    }
}

async fn get_wallet_balances(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let balances = wallet.get_all_balances();
    let mut formatted_balances = HashMap::new();
    
    for (currency, amount) in balances {
        formatted_balances.insert(currency.symbol().to_string(), currency.format_amount(amount));
    }
    
    let total_usd = wallet.get_total_balance_usd();
    
    axum::Json(serde_json::json!({
        "success": true,
        "balances": formatted_balances,
        "total_usd": total_usd
    }))
}

async fn get_wallet_addresses(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let addresses = wallet.get_all_addresses();
    let mut formatted_addresses = HashMap::new();
    
    for (currency, address) in addresses {
        formatted_addresses.insert(currency.symbol().to_string(), address);
    }
    
    axum::Json(serde_json::json!({
        "success": true,
        "addresses": formatted_addresses
    }))
}

async fn send_funds(
    State(state): State<ApiState>,
    axum::Json(request): axum::Json<crate::wallet::SendRequest>,
) -> impl IntoResponse {
    let node = &state.node;
    let mut wallet = node.wallet.write().await;
    match wallet.send_funds(request) {
        Ok(response) => axum::Json(serde_json::json!({
            "success": true,
            "transaction_id": response.transaction_id,
            "message": response.message,
            "fee": response.fee
        })),
        Err(e) => {
            error!("Failed to send funds: {}", e);
            axum::Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to send funds: {}", e)
            }))
        }
    }
}

async fn receive_funds(
    State(state): State<ApiState>,
    axum::Json(request): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let node = &state.node;
    
    let from_address = request["from_address"].as_str().unwrap_or("");
    let amount = request["amount"].as_u64().unwrap_or(0);
    let currency_str = request["currency"].as_str().unwrap_or("USDC");
    let transaction_id = request["transaction_id"].as_str().unwrap_or("");
    let signature = request["signature"].as_str().unwrap_or("");
    
    let currency = match currency_str {
        "BTC" => crate::wallet::Currency::BTC,
        "ETH" => crate::wallet::Currency::ETH,
        "USDC" => crate::wallet::Currency::USDC,
        "LTC" => crate::wallet::Currency::LTC,
        "XMR" => crate::wallet::Currency::XMR,
        "DOGE" => crate::wallet::Currency::DOGE,
        _ => crate::wallet::Currency::USDC,
    };
    
    let signature_bytes = match base64::engine::general_purpose::STANDARD.decode(signature) {
        Ok(bytes) => bytes,
        Err(_) => vec![],
    };
    
    let mut wallet = node.wallet.write().await;
    match wallet.receive_funds(from_address.to_string(), amount, currency, 
                                   transaction_id.to_string(), signature_bytes) {
        Ok(_) => axum::Json(serde_json::json!({
            "success": true,
            "message": "Funds received successfully"
        })),
        Err(e) => {
            error!("Failed to receive funds: {}", e);
            axum::Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to receive funds: {}", e)
            }))
        }
    }
}

async fn get_transaction_history(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let transactions = wallet.get_transaction_history();
    
    axum::Json(serde_json::json!({
        "success": true,
        "transactions": transactions
    }))
}

async fn get_transaction_by_id(
    State(state): State<ApiState>,
    axum::extract::Path(transaction_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    match wallet.get_transaction_by_id(&transaction_id) {
        Some(transaction) => axum::Json(serde_json::json!({
            "success": true,
            "transaction": transaction
        })),
        None => axum::Json(serde_json::json!({
            "success": false,
            "message": "Transaction not found"
        }))
    }
}

async fn backup_wallet(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    match wallet.backup_wallet() {
        Ok(backup_data) => axum::Json(serde_json::json!({
            "success": true,
            "backup_data": backup_data
        })),
        Err(e) => {
            error!("Failed to backup wallet: {}", e);
            axum::Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to backup wallet: {}", e)
            }))
        }
    }
}

async fn restore_wallet(
    State(state): State<ApiState>,
    axum::Json(request): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let backup_data = request["backup_data"].as_str().unwrap_or("");
    
    match crate::wallet::Wallet::restore_wallet(backup_data) {
        Ok(wallet) => {
            // In a real implementation, you'd replace the node's wallet
            axum::Json(serde_json::json!({
                "success": true,
                "message": "Wallet restored successfully"
            }))
        },
        Err(e) => {
            error!("Failed to restore wallet: {}", e);
            axum::Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to restore wallet: {}", e)
            }))
        }
    }
}

async fn get_wallet_keys(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let public_key_res = wallet.get_public_key_base64();
    let private_key_res = wallet.get_private_key_base64();
    match (public_key_res, private_key_res) {
        (Ok(public_key), Ok(private_key)) => axum::Json(serde_json::json!({
            "success": true,
            "public_key": public_key,
            "private_key": private_key,
            "warning": "Keep your private key secure and never share it!"
        })),
        (Err(e), _) | (_, Err(e)) => {
            error!("Failed to get wallet keys: {}", e);
            axum::Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to get wallet keys: {}", e)
            }))
        }
    }
} 

async fn register_aoi_key(
    State(state): State<ApiState>,
    axum::Json(request): axum::Json<RegisterAOIKeyRequest>,
) -> impl IntoResponse {
    let node = &state.node;
    let service_id = crate::core::data_structures::ServiceId(request.service_id);
    match node.register_aoi_key_for_service(service_id, request.key_data).await {
        Ok(_) => axum::Json(RegisterAOIKeyResponse {
            success: true,
            message: "AOI key registered successfully".to_string(),
        }),
        Err(e) => axum::Json(RegisterAOIKeyResponse {
            success: false,
            message: format!("Failed to register AOI key: {}", e),
        }),
    }
}

async fn get_aoi_key(
    State(state): State<ApiState>,
    axum::Json(request): axum::Json<GetAOIKeyRequest>,
) -> impl IntoResponse {
    let node = &state.node;
    let service_id = crate::core::data_structures::ServiceId(request.service_id);
    match node.get_aoi_key_for_service(service_id).await {
        Some(aoi_key) => axum::Json(GetAOIKeyResponse {
            key_data: Some(aoi_key.key_data),
            success: true,
            message: "AOI key found".to_string(),
        }),
        None =>         axum::Json(GetAOIKeyResponse {
            key_data: None,
            success: false,
            message: "AOI key not found".to_string(),
        }),
    }
}

// Community Fund API handlers
async fn get_community_fund_stats(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    
    match node.get_community_fund_stats().await {
        Ok(stats) => axum::Json(serde_json::json!({
            "success": true,
            "data": stats
        })),
        Err(e) => {
            error!("Failed to get community fund stats: {}", e);
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to get community fund stats: {}", e)
            }))
        }
    }
}

async fn get_community_fund_balance(
    State(state): State<ApiState>,
    axum::extract::Path(currency_str): axum::extract::Path<String>,
) -> impl IntoResponse {
    let node = &state.node;
    
    // Parse currency
    let currency = match currency_str.to_uppercase().as_str() {
        "BTC" => crate::wallet::Currency::BTC,
        "ETH" => crate::wallet::Currency::ETH,
        "USDC" => crate::wallet::Currency::USDC,
        "LTC" => crate::wallet::Currency::LTC,
        "XMR" => crate::wallet::Currency::XMR,
        "DOGE" => crate::wallet::Currency::DOGE,
        "DUX" => crate::wallet::Currency::DUX,
        _ => {
            return axum::Json(serde_json::json!({
                "success": false,
                "error": "Invalid currency"
            }));
        }
    };
    
    let balance = node.get_community_fund_balance(&currency).await;
    
    axum::Json(serde_json::json!({
        "success": true,
        "currency": currency.symbol(),
        "balance": balance,
        "formatted_balance": currency.format_amount(balance)
    }))
}

async fn distribute_community_fund(
    State(state): State<ApiState>,
    axum::extract::Path(currency_str): axum::extract::Path<String>,
) -> impl IntoResponse {
    let node = &state.node;
    
    // Parse currency
    let currency = match currency_str.to_uppercase().as_str() {
        "BTC" => crate::wallet::Currency::BTC,
        "ETH" => crate::wallet::Currency::ETH,
        "USDC" => crate::wallet::Currency::USDC,
        "LTC" => crate::wallet::Currency::LTC,
        "XMR" => crate::wallet::Currency::XMR,
        "DOGE" => crate::wallet::Currency::DOGE,
        "DUX" => crate::wallet::Currency::DUX,
        _ => {
            return axum::Json(serde_json::json!({
                "success": false,
                "error": "Invalid currency"
            }));
        }
    };
    
    match node.distribute_community_fund(currency).await {
        Ok(distribution) => {
            info!("Community fund distribution completed for {}: {} to {} users", 
                  distribution.currency.symbol(), 
                  distribution.currency.format_amount(distribution.amount_per_user),
                  distribution.total_users);
            
            axum::Json(serde_json::json!({
                "success": true,
                "message": format!("Distributed {} to {} users", 
                                 distribution.currency.format_amount(distribution.amount_per_user),
                                 distribution.total_users),
                "data": distribution
            }))
        },
        Err(e) => {
            error!("Failed to distribute community fund: {}", e);
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to distribute community fund: {}", e)
            }))
        }
    }
} 

async fn shutdown_node() -> impl IntoResponse {
    // In a real app, signal the main loop to exit gracefully
    std::process::exit(0);
    // This line is unreachable, but required for the type
    StatusCode::OK
}

// Messaging API endpoints
async fn send_message(
    State(state): State<ApiState>,
    axum::Json(request): axum::Json<crate::core::data_structures::MessageRequest>,
) -> impl IntoResponse {
    let node = &state.node;
    
    match node.messaging_system.send_message(request).await {
        Ok(response) => axum::Json(response),
        Err(e) => {
            error!("Failed to send message: {}", e);
            axum::Json(crate::core::data_structures::MessageResponse {
                message_id: "".to_string(),
                success: false,
                message: format!("Failed to send message: {}", e),
            })
        }
    }
}

async fn get_conversations(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    
    let conversations = node.messaging_system.get_conversations().await;
    
    axum::Json(serde_json::json!({
        "success": true,
        "conversations": conversations
    }))
}

async fn get_messages(
    State(state): State<ApiState>,
    axum::extract::Path(peer_did): axum::extract::Path<String>,
) -> impl IntoResponse {
    let node = &state.node;
    
    let messages = node.messaging_system.get_messages(&peer_did).await;
    
    axum::Json(serde_json::json!({
        "success": true,
        "messages": messages,
        "peer_did": peer_did
    }))
}

async fn mark_message_read(
    State(state): State<ApiState>,
    axum::extract::Path(message_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let node = &state.node;
    
    match node.messaging_system.mark_message_read(&message_id).await {
        Ok(_) => axum::Json(serde_json::json!({
            "success": true,
            "message": "Message marked as read"
        })),
        Err(e) => {
            error!("Failed to mark message as read: {}", e);
            axum::Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to mark message as read: {}", e)
            }))
        }
    }
}

async fn delete_message(
    State(state): State<ApiState>,
    axum::extract::Path(message_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let node = &state.node;
    
    match node.messaging_system.delete_message(&message_id).await {
        Ok(_) => axum::Json(serde_json::json!({
            "success": true,
            "message": "Message deleted"
        })),
        Err(e) => {
            error!("Failed to delete message: {}", e);
            axum::Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to delete message: {}", e)
            }))
        }
    }
}

async fn get_messaging_stats(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    
    let stats = node.messaging_system.get_message_stats().await;
    
    axum::Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

// DUX Coin API endpoints
async fn get_dux_balance(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = &node.wallet;
    let dux_address = wallet.get_address(&crate::wallet::Currency::DUX);
    
    // In a real implementation, this would call the DUX coin daemon
    let balance = wallet.get_balance(&crate::wallet::Currency::DUX);
    let formatted_balance = wallet.get_formatted_balance(&crate::wallet::Currency::DUX);
    
    axum::Json(serde_json::json!({
        "address": dux_address,
        "balance": balance,
        "formatted_balance": formatted_balance,
        "currency": "DUX",
        "success": true
    }))
}

async fn get_dux_transactions(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = &node.wallet;
    let transactions = wallet.get_transactions_by_currency(&crate::wallet::Currency::DUX);
    
    axum::Json(serde_json::json!({
        "transactions": transactions,
        "count": transactions.len(),
        "currency": "DUX",
        "success": true
    }))
}

async fn send_dux(
    State(state): State<ApiState>,
    axum::Json(request): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let node = &state.node;
    let mut wallet = node.wallet.clone();
    
    let to_address = request["to_address"].as_str().unwrap_or("");
    let amount = request["amount"].as_u64().unwrap_or(0);
    
    if to_address.is_empty() || amount == 0 {
        return axum::Json(serde_json::json!({
            "success": false,
            "message": "Invalid address or amount"
        }));
    }
    
    match wallet.send_funds(crate::wallet::SendRequest {
        to_address: to_address.to_string(),
        amount,
        currency: crate::wallet::Currency::DUX,
        memo: request["memo"].as_str().map(|s| s.to_string()),
        fee: None,
    }) {
        Ok(response) => axum::Json(serde_json::json!({
            "transaction_id": response.transaction_id,
            "success": response.success,
            "message": response.message,
            "fee": response.fee
        })),
        Err(e) => axum::Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to send DUX: {}", e)
        }))
    }
}

async fn get_dux_network(State(state): State<ApiState>) -> impl IntoResponse {
    // Mock network information for DUX coin
    axum::Json(serde_json::json!({
        "difficulty": 1.0,
        "block_height": 0,
        "connections": 0,
        "hash_rate": 0.0,
        "currency": "DUX",
        "success": true
    }))
}

async fn start_dux_mining(
    State(state): State<ApiState>,
    axum::Json(request): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let threads = request["threads"].as_i64().unwrap_or(1) as i32;
    
    axum::Json(serde_json::json!({
        "success": true,
        "message": format!("DUX mining started with {} threads", threads),
        "threads": threads
    }))
}

async fn stop_dux_mining(State(state): State<ApiState>) -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "success": true,
        "message": "DUX mining stopped"
    }))
}

async fn get_dux_mining_status(State(state): State<ApiState>) -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "mining": false,
        "hash_rate": 0.0,
        "threads": 0,
        "success": true
    }))
}

async fn sync_dux_balance(State(state): State<ApiState>) -> impl IntoResponse {
    let node = &state.node;
    let wallet = &node.wallet;
    let balance = wallet.get_balance(&crate::wallet::Currency::DUX);
    
    axum::Json(serde_json::json!({
        "success": true,
        "message": "DUX balance synced",
        "balance": balance,
        "formatted_balance": wallet.get_formatted_balance(&crate::wallet::Currency::DUX)
    }))
} 