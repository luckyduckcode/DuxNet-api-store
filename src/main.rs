mod core;
mod network;
mod api;
mod wallet;
mod frontend;

use anyhow::Result;
use tracing::{info, error, warn};
use tracing_subscriber;
use std::env;
use tokio::time::{sleep, Duration};
use crate::core::data_structures::{ReputationAttestation, ServiceId, TaskRequirements};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting DuxNet Decentralized P2P Platform");
    
    // Check if we're in test mode
    let test_mode = env::var("DUXNET_TEST_MODE").unwrap_or_default() == "1";
    
    // Create and start the DuxNet node
    let mut node = core::DuxNetNode::new(8080).await?;
    
    if test_mode {
        info!("🧪 TEST MODE ENABLED - Simulating active network with users");
        start_test_simulation(Arc::new(node.clone())).await;
    }
    
    // Start the web API server
    let api_handle = tokio::spawn(async move {
        if let Err(e) = api::start_api_server(8081).await {
            error!("API server error: {}", e);
        }
    });
    
    // Start the P2P network
    let network_handle = tokio::spawn(async move {
        if let Err(e) = node.start().await {
            error!("Network error: {}", e);
        }
    });
    
    info!("DuxNet node started successfully!");
    info!("Web API available at: http://localhost:8081");
    info!("P2P node listening on port: 8080");
    
    if test_mode {
        info!("🌐 Frontend available at: http://localhost:8081 (with simulated data)");
        info!("💡 Use 'DUXNET_TEST_MODE=1' environment variable to enable test mode");
    }
    
    // Wait for both handles
    tokio::try_join!(api_handle, network_handle)?;
    
    Ok(())
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