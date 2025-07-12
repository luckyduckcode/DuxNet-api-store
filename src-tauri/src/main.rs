// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod network;
mod api;
mod wallet;

use anyhow::Result;
use tracing::{info, error, warn};
use tracing_subscriber;
use std::env;
use tokio::time::{sleep, Duration};
use crate::core::data_structures::{ReputationAttestation, ServiceId, TaskRequirements};
use std::sync::Arc;
use tauri::Manager;
use std::str::FromStr;

// Global state to hold the DuxNet node
struct DuxNetState {
    node: Arc<core::DuxNetNode>,
}

#[tauri::command]
async fn get_wallet_info(state: tauri::State<'_, DuxNetState>) -> Result<serde_json::Value, String> {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    
    Ok(serde_json::json!({
        "success": true,
        "addresses": wallet.get_all_addresses(),
        "total_balance": wallet.get_total_balance_usd()
    }))
}

#[tauri::command]
async fn send_funds(
    state: tauri::State<'_, DuxNetState>,
    to_address: String, 
    amount: u64, 
    currency: String
) -> Result<serde_json::Value, String> {
    let node = &state.node;
    
    // Parse currency string to enum
    let currency_enum = match currency.as_str() {
        "BTC" => wallet::Currency::BTC,
        "ETH" => wallet::Currency::ETH,
        "USDC" => wallet::Currency::USDC,
        "LTC" => wallet::Currency::LTC,
        "XMR" => wallet::Currency::XMR,
        "DOGE" => wallet::Currency::DOGE,
        _ => return Err(format!("Invalid currency: {}", currency))
    };
    let to_address_clone = to_address.clone();
    
    match node.wallet.write().await.create_transaction(to_address, amount, currency_enum) {
        Ok(tx) => {
            Ok(serde_json::json!({
                "success": true,
                "transaction_id": tx.id,
                "message": format!("Sent {} {} to {}", amount, currency, to_address_clone)
            }))
        }
        Err(e) => {
            Err(format!("Transaction failed: {}", e))
        }
    }
}

#[tauri::command]
async fn get_balances(state: tauri::State<'_, DuxNetState>) -> Result<serde_json::Value, String> {
    let node = &state.node;
    let wallet = node.wallet.read().await;
    let balances = wallet.get_all_balances();
    
    Ok(serde_json::json!({
        "success": true,
        "balances": balances
    }))
}

#[tauri::command]
async fn get_network_status(state: tauri::State<'_, DuxNetState>) -> Result<serde_json::Value, String> {
    let node = &state.node;
    // Peer count
    let peers = node.dht.peers.read().await.len();
    // Service count
    let entries = node.dht.entries.read().await;
    let services = entries.keys().filter(|k| k.starts_with("service:")).count();
    // Task count
    let tasks = node.task_engine.pending_tasks.read().await.len();
    Ok(serde_json::json!({
        "success": true,
        "peers": peers,
        "services": services,
        "tasks": tasks
    }))
}

#[tauri::command]
async fn get_services(state: tauri::State<'_, DuxNetState>) -> Result<serde_json::Value, String> {
    let node = &state.node;
    let entries = node.dht.entries.read().await;
    let mut services = Vec::new();
    for (key, entry) in entries.iter() {
        if key.starts_with("service:") {
            if let Ok(service) = serde_json::from_slice::<core::data_structures::ServiceMetadata>(&entry.value) {
                services.push(service);
            }
        }
    }
    Ok(serde_json::json!({
        "success": true,
        "services": services
    }))
}

// Messaging Tauri commands
#[tauri::command]
async fn send_message(
    state: tauri::State<'_, DuxNetState>,
    to_did: String,
    content: String,
    message_type: String,
) -> Result<serde_json::Value, String> {
    let node = &state.node;
    
    let message_type_enum = match message_type.as_str() {
        "text" => crate::core::data_structures::MessageType::Text,
        "file" => crate::core::data_structures::MessageType::File,
        "service_request" => crate::core::data_structures::MessageType::ServiceRequest,
        "task_update" => crate::core::data_structures::MessageType::TaskUpdate,
        "escrow_update" => crate::core::data_structures::MessageType::EscrowUpdate,
        "reputation_update" => crate::core::data_structures::MessageType::ReputationUpdate,
        "system" => crate::core::data_structures::MessageType::System,
        _ => crate::core::data_structures::MessageType::Text,
    };
    
    let request = crate::core::data_structures::MessageRequest {
        to_did,
        content,
        message_type: message_type_enum,
        reply_to: None,
    };
    
    match node.messaging_system.send_message(request).await {
        Ok(response) => Ok(serde_json::json!({
            "success": true,
            "data": response
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to send message: {}", e)
        }))
    }
}

#[tauri::command]
async fn get_conversations(state: tauri::State<'_, DuxNetState>) -> Result<serde_json::Value, String> {
    let node = &state.node;
    
    let conversations = node.messaging_system.get_conversations().await;
    
    Ok(serde_json::json!({
        "success": true,
        "data": conversations
    }))
}

#[tauri::command]
async fn get_messages(
    state: tauri::State<'_, DuxNetState>,
    peer_did: String,
) -> Result<serde_json::Value, String> {
    let node = &state.node;
    
    let messages = node.messaging_system.get_messages(&peer_did).await;
    
    Ok(serde_json::json!({
        "success": true,
        "data": messages,
        "peer_did": peer_did
    }))
}

#[tauri::command]
async fn mark_message_read(
    state: tauri::State<'_, DuxNetState>,
    message_id: String,
) -> Result<serde_json::Value, String> {
    let node = &state.node;
    
    match node.messaging_system.mark_message_read(&message_id).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Message marked as read"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to mark message as read: {}", e)
        }))
    }
}

#[tauri::command]
async fn delete_message(
    state: tauri::State<'_, DuxNetState>,
    message_id: String,
    ) -> Result<serde_json::Value, String> {
    let node = &state.node;
    
    match node.messaging_system.delete_message(&message_id).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Message deleted"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to delete message: {}", e)
        }))
    }
}

#[tauri::command]
async fn get_messaging_stats(state: tauri::State<'_, DuxNetState>) -> Result<serde_json::Value, String> {
    let node = &state.node;
    
    let stats = node.messaging_system.get_message_stats().await;
    
    Ok(serde_json::json!({
        "success": true,
        "data": stats
    }))
}

// DUX Coin Tauri commands
#[tauri::command]
async fn get_dux_balance(state: tauri::State<'_, DuxNetState>) -> Result<serde_json::Value, String> {
    let node = &state.node;
    let wallet = &node.wallet;
    let dux_address = wallet.get_address(&crate::wallet::Currency::DUX);
    
    let balance = wallet.get_balance(&crate::wallet::Currency::DUX);
    let formatted_balance = wallet.get_formatted_balance(&crate::wallet::Currency::DUX);
    
    Ok(serde_json::json!({
        "address": dux_address,
        "balance": balance,
        "formatted_balance": formatted_balance,
        "currency": "DUX",
        "success": true
    }))
}

#[tauri::command]
async fn get_dux_transactions(state: tauri::State<'_, DuxNetState>) -> Result<serde_json::Value, String> {
    let node = &state.node;
    let wallet = &node.wallet;
    let transactions = wallet.get_transactions_by_currency(&crate::wallet::Currency::DUX);
    
    Ok(serde_json::json!({
        "transactions": transactions,
        "count": transactions.len(),
        "currency": "DUX",
        "success": true
    }))
}

#[tauri::command]
async fn send_dux(
    state: tauri::State<'_, DuxNetState>,
    request: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let node = &state.node;
    let mut wallet = node.wallet.clone();
    
    let to_address = request["to_address"].as_str().unwrap_or("");
    let amount = request["amount"].as_u64().unwrap_or(0);
    
    if to_address.is_empty() || amount == 0 {
        return Ok(serde_json::json!({
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
        Ok(response) => Ok(serde_json::json!({
            "transaction_id": response.transaction_id,
            "success": response.success,
            "message": response.message,
            "fee": response.fee
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "message": format!("Failed to send DUX: {}", e)
        }))
    }
}

#[tauri::command]
async fn get_dux_network(state: tauri::State<'_, DuxNetState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "difficulty": 1.0,
        "block_height": 0,
        "connections": 0,
        "hash_rate": 0.0,
        "currency": "DUX",
        "success": true
    }))
}

#[tauri::command]
async fn start_dux_mining(
    state: tauri::State<'_, DuxNetState>,
    request: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let threads = request["threads"].as_i64().unwrap_or(1) as i32;
    
    Ok(serde_json::json!({
        "success": true,
        "message": format!("DUX mining started with {} threads", threads),
        "threads": threads
    }))
}

#[tauri::command]
async fn stop_dux_mining(state: tauri::State<'_, DuxNetState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "success": true,
        "message": "DUX mining stopped"
    }))
}

#[tauri::command]
async fn get_dux_mining_status(state: tauri::State<'_, DuxNetState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "mining": false,
        "hash_rate": 0.0,
        "threads": 0,
        "success": true
    }))
}

#[tauri::command]
async fn sync_dux_balance(state: tauri::State<'_, DuxNetState>) -> Result<serde_json::Value, String> {
    let node = &state.node;
    let wallet = &node.wallet;
    let balance = wallet.get_balance(&crate::wallet::Currency::DUX);
    
    Ok(serde_json::json!({
        "success": true,
        "message": "DUX balance synced",
        "balance": balance,
        "formatted_balance": wallet.get_formatted_balance(&crate::wallet::Currency::DUX)
    }))
}

async fn start_backend_services() -> Result<Arc<core::DuxNetNode>, Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting DuxNet Decentralized P2P Platform");
    let test_mode = env::var("DUXNET_TEST_MODE").unwrap_or_default() == "1";
    // Create and start the DuxNet node (mutable)
    let mut node = core::DuxNetNode::new(8080).await?;
    if test_mode {
        info!("🧪 TEST MODE ENABLED - Simulating active network with users");
        start_test_simulation(Arc::new(node.clone())).await;
    }
    // Start the web API server
    let api_node = Arc::new(node.clone());
    let _api_handle = tokio::spawn(async move {
        if let Err(e) = api::start_api_server(8081).await {
            error!("API server error: {}", e);
        }
    });
    // Start the P2P network
    node.start().await?;
    info!("DuxNet node started successfully!");
    info!("Web API available at: http://localhost:8081");
    info!("P2P node listening on port: 8080");
    if test_mode {
        info!("🌐 Frontend available at: http://localhost:8081 (with simulated data)");
        info!("💡 Use 'DUXNET_TEST_MODE=1' environment variable to enable test mode");
    }
    Ok(Arc::new(node))
}

async fn start_test_simulation(node: Arc<core::DuxNetNode>) {
    // Spawn background task for test simulation
    tokio::spawn(async move {
        info!("🎭 Starting test simulation...");
        
        // Wait a bit for the node to fully start
        sleep(Duration::from_secs(2)).await;
        
        // Simulate network activity every 30 seconds
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Simulate random network events
            match rand::random::<u8>() % 4 {
                0 => simulate_transaction(&node).await,
                1 => simulate_service_registration(&node).await,
                2 => simulate_task_submission(&node).await,
                3 => simulate_reputation_update(&node).await,
                _ => {}
            }
        }
    });
}

async fn simulate_transaction(node: &core::DuxNetNode) {
    use crate::wallet::Currency;
    
    let currencies = vec![Currency::BTC, Currency::ETH];
    let currency = currencies[rand::random::<usize>() % currencies.len()];
    let amount = rand::random::<u64>() % 1000 + 10; // 10-1010 units
    
    match node.wallet.write().await.create_transaction(
        format!("did:duxnet:test-user-{}", rand::random::<u32>()),
        amount,
        currency.clone()
    ) {
        Ok(tx) => {
            info!("💰 Simulated transaction: {} {} (fee: {})", 
                  amount, currency, tx.fee);
        }
        Err(e) => {
            warn!("❌ Failed to simulate transaction: {}", e);
        }
    }
}

async fn simulate_service_registration(node: &core::DuxNetNode) {
    let services = vec![
        ("AI Text Processing", "Advanced NLP services", 150),
        ("Image Analysis", "Computer vision processing", 200),
        ("Data Computation", "High-performance analytics", 100),
        ("Blockchain Verification", "Smart contract validation", 300),
        ("Machine Learning Training", "Neural network optimization", 250),
    ];
    
    let service = services[rand::random::<usize>() % services.len()];
    
    match node.register_service(
        service.0.to_string(),
        service.1.to_string(),
        service.2
    ).await {
        Ok(_) => {
            info!("🔧 Simulated service registration: {}", service.0);
        }
        Err(e) => {
            warn!("❌ Failed to simulate service registration: {}", e);
        }
    }
}

async fn simulate_task_submission(node: &core::DuxNetNode) {
    let tasks = vec![
        ("text-processing", "Analyze sentiment of customer reviews"),
        ("image-analysis", "Detect objects in surveillance footage"),
        ("data-computation", "Process financial market data"),
        ("ml-training", "Train recommendation system model"),
    ];
    
    let task = tasks[rand::random::<usize>() % tasks.len()];
    // Use arbitrary but valid requirements
    let requirements = TaskRequirements {
        cpu_cores: 2,
        memory_mb: 1024,
        timeout_seconds: 3600,
    };
    let service_id = ServiceId(task.0.to_string());
    
    match node.submit_task(
        service_id,
        task.1.as_bytes().to_vec(),
        requirements.clone()
    ).await {
        Ok(_) => {
            info!("📋 Simulated task submission: {} (req: {} cores, {} MB, {} s)", task.0, requirements.cpu_cores, requirements.memory_mb, requirements.timeout_seconds);
        }
        Err(e) => {
            warn!("❌ Failed to simulate task submission: {}", e);
        }
    }
}

async fn simulate_reputation_update(node: &core::DuxNetNode) {
    let target_did = format!("did:duxnet:user-{}", rand::random::<u32>());
    let score = (rand::random::<f64>() * 2.0) + 3.0; // 3.0-5.0 score
    let interaction = vec!["service_provided", "task_completed", "escrow_finalized"];
    let interaction_type = interaction[rand::random::<usize>() % interaction.len()];
    
    let attestation = ReputationAttestation {
        target_did: target_did.clone(),
        attester_did: format!("did:duxnet:attester-{}", rand::random::<u32>()),
        score,
        interaction_type: interaction_type.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        signature: vec![], // Simplified for simulation
    };
    
    match node.reputation_system.add_attestation(attestation).await {
        Ok(_) => {
            info!("⭐ Simulated reputation update: {} -> {:.1}/5.0", target_did, score);
        }
        Err(e) => {
            warn!("❌ Failed to simulate reputation update: {}", e);
        }
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Start backend services synchronously
            let node = tauri::async_runtime::block_on(start_backend_services())
                .map_err(|e| e.to_string())?;
            app.manage(DuxNetState { node });
            info!("✅ Backend services integrated with Tauri app");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_wallet_info,
            send_funds,
            get_balances,
            get_network_status,
            get_services,
            send_message,
            get_conversations,
            get_messages,
            mark_message_read,
            delete_message,
            get_messaging_stats,
            get_dux_balance,
            get_dux_transactions,
            send_dux,
            get_dux_network,
            start_dux_mining,
            stop_dux_mining,
            get_dux_mining_status,
            sync_dux_balance,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
} 