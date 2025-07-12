use crate::core::data_structures::*;
use crate::wallet::Currency;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error};

/// DUX Coin API client for DuxNet integration
pub struct DuxCoinAPI {
    rpc_url: String,
    rpc_user: String,
    rpc_password: String,
    client: reqwest::Client,
    cache: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

/// DUX Coin balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuxBalance {
    pub confirmed: f64,
    pub unconfirmed: f64,
    pub total: f64,
    pub address: String,
}

/// DUX Coin transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuxTransaction {
    pub txid: String,
    pub amount: f64,
    pub confirmations: i32,
    pub address: String,
    pub time: i64,
    pub category: String,
}

/// DUX Coin network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuxNetworkInfo {
    pub difficulty: f64,
    pub block_height: i64,
    pub connections: i32,
    pub hash_rate: f64,
}

/// DUX Coin API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuxAPIResponse<T> {
    pub result: Option<T>,
    pub error: Option<serde_json::Value>,
    pub id: String,
}

impl DuxCoinAPI {
    /// Create a new DUX Coin API client
    pub fn new(rpc_url: String, rpc_user: String, rpc_password: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            rpc_url,
            rpc_user,
            rpc_password,
            client,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Make an RPC call to the DUX coin daemon
    async fn rpc_call<T>(&self, method: &str, params: Vec<serde_json::Value>) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let request_body = serde_json::json!({
            "jsonrpc": "1.0",
            "id": "duxnet",
            "method": method,
            "params": params
        });

        let response = self
            .client
            .post(&self.rpc_url)
            .basic_auth(&self.rpc_user, Some(&self.rpc_password))
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("RPC call failed: {}", response.status()));
        }

        let response_text = response.text().await?;
        let api_response: DuxAPIResponse<T> = serde_json::from_str(&response_text)?;

        if let Some(error) = api_response.error {
            return Err(anyhow::anyhow!("RPC error: {:?}", error));
        }

        api_response.result.ok_or_else(|| anyhow::anyhow!("No result in response"))
    }

    /// Get DUX balance for an address
    pub async fn get_balance(&self, address: &str) -> Result<DuxBalance> {
        let cache_key = format!("balance:{}", address);
        
        // Check cache first
        {
            let cache = self.cache.lock().await;
            if let Some(cached) = cache.get(&cache_key) {
                if let Ok(balance) = serde_json::from_value::<DuxBalance>(cached.clone()) {
                    return Ok(balance);
                }
            }
        }

        let result: serde_json::Value = self.rpc_call("getreceivedbyaddress", vec![
            serde_json::Value::String(address.to_string()),
        ]).await?;

        let confirmed = result.as_f64().unwrap_or(0.0);
        
        let balance = DuxBalance {
            confirmed,
            unconfirmed: 0.0, // DUX coin doesn't have unconfirmed in this context
            total: confirmed,
            address: address.to_string(),
        };

        // Cache the result
        {
            let mut cache = self.cache.lock().await;
            cache.insert(cache_key, serde_json::to_value(&balance)?);
        }

        Ok(balance)
    }

    /// Get DUX transactions for an address
    pub async fn get_transactions(&self, address: &str, limit: i32) -> Result<Vec<DuxTransaction>> {
        let cache_key = format!("transactions:{}:{}", address, limit);
        
        // Check cache first
        {
            let cache = self.cache.lock().await;
            if let Some(cached) = cache.get(&cache_key) {
                if let Ok(txs) = serde_json::from_value::<Vec<DuxTransaction>>(cached.clone()) {
                    return Ok(txs);
                }
            }
        }

        let result: Vec<serde_json::Value> = self.rpc_call("listtransactions", vec![
            serde_json::Value::String(address.to_string()),
            serde_json::Value::Number(limit.into()),
        ]).await?;

        let mut transactions = Vec::new();
        for tx_data in result {
            if let Ok(tx) = serde_json::from_value::<DuxTransaction>(tx_data) {
                transactions.push(tx);
            }
        }

        // Cache the result
        {
            let mut cache = self.cache.lock().await;
            cache.insert(cache_key, serde_json::to_value(&transactions)?);
        }

        Ok(transactions)
    }

    /// Send DUX coins
    pub async fn send_dux(&self, from_address: &str, to_address: &str, amount: f64) -> Result<String> {
        let result: serde_json::Value = self.rpc_call("sendtoaddress", vec![
            serde_json::Value::String(to_address.to_string()),
            serde_json::Value::Number(serde_json::Number::from_f64(amount).unwrap()),
        ]).await?;

        let txid = result.as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid transaction ID"))?
            .to_string();

        info!("Sent {} DUX from {} to {}: {}", amount, from_address, to_address, txid);
        Ok(txid)
    }

    /// Get DUX network information
    pub async fn get_network_info(&self) -> Result<DuxNetworkInfo> {
        let cache_key = "network_info".to_string();
        
        // Check cache first
        {
            let cache = self.cache.lock().await;
            if let Some(cached) = cache.get(&cache_key) {
                if let Ok(info) = serde_json::from_value::<DuxNetworkInfo>(cached.clone()) {
                    return Ok(info);
                }
            }
        }

        let difficulty: f64 = self.rpc_call("getdifficulty", vec![]).await?;
        let block_height: i64 = self.rpc_call("getblockcount", vec![]).await?;
        let connections: i32 = self.rpc_call("getconnectioncount", vec![]).await?;
        
        // Estimate hash rate (simplified)
        let hash_rate = difficulty * 2.0; // Rough estimate

        let network_info = DuxNetworkInfo {
            difficulty,
            block_height,
            connections,
            hash_rate,
        };

        // Cache the result for 30 seconds
        {
            let mut cache = self.cache.lock().await;
            cache.insert(cache_key, serde_json::to_value(&network_info)?);
        }

        Ok(network_info)
    }

    /// Validate DUX address
    pub async fn validate_address(&self, address: &str) -> Result<bool> {
        let result: serde_json::Value = self.rpc_call("validateaddress", vec![
            serde_json::Value::String(address.to_string()),
        ]).await?;

        Ok(result["isvalid"].as_bool().unwrap_or(false))
    }

    /// Generate new DUX address
    pub async fn generate_address(&self) -> Result<String> {
        let result: serde_json::Value = self.rpc_call("getnewaddress", vec![]).await?;
        
        result.as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid address"))
            .map(|s| s.to_string())
    }

    /// Estimate DUX transaction fee
    pub async fn estimate_fee(&self, blocks: i32) -> Result<f64> {
        let result: f64 = self.rpc_call("estimatesmartfee", vec![
            serde_json::Value::Number(blocks.into()),
        ]).await?;

        Ok(result)
    }

    /// Start DUX mining
    pub async fn start_mining(&self, threads: i32) -> Result<bool> {
        let _: serde_json::Value = self.rpc_call("setgenerate", vec![
            serde_json::Value::Bool(true),
            serde_json::Value::Number(threads.into()),
        ]).await?;

        info!("Started DUX mining with {} threads", threads);
        Ok(true)
    }

    /// Stop DUX mining
    pub async fn stop_mining(&self) -> Result<bool> {
        let _: serde_json::Value = self.rpc_call("setgenerate", vec![
            serde_json::Value::Bool(false),
        ]).await?;

        info!("Stopped DUX mining");
        Ok(true)
    }

    /// Get DUX mining hash rate
    pub async fn get_hash_rate(&self) -> Result<f64> {
        let result: f64 = self.rpc_call("gethashespersec", vec![]).await?;
        Ok(result)
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
        info!("DUX Coin API cache cleared");
    }
}

/// DUX Coin integration for DuxNet wallet
pub struct DuxNetDuxIntegration {
    api: DuxCoinAPI,
    wallet: Arc<Mutex<crate::wallet::Wallet>>,
}

impl DuxNetDuxIntegration {
    /// Create new DUX integration
    pub fn new(api: DuxCoinAPI, wallet: crate::wallet::Wallet) -> Self {
        Self {
            api,
            wallet: Arc::new(Mutex::new(wallet)),
        }
    }

    /// Sync DUX balance with DuxNet wallet
    pub async fn sync_balance(&self) -> Result<()> {
        let mut wallet = self.wallet.lock().await;
        let dux_address = wallet.get_address(&Currency::DUX);
        
        match self.api.get_balance(&dux_address).await {
            Ok(balance) => {
                let amount = (balance.total * 100_000_000.0) as u64; // Convert to satoshis
                wallet.add_funds(Currency::DUX, amount);
                info!("Synced DUX balance: {} DUX", balance.total);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to sync DUX balance: {}", e);
                Err(e)
            }
        }
    }

    /// Send DUX from DuxNet wallet
    pub async fn send_dux(&self, to_address: &str, amount: u64) -> Result<String> {
        let mut wallet = self.wallet.lock().await;
        let from_address = wallet.get_address(&Currency::DUX);
        
        // Remove from wallet first
        wallet.remove_funds(&Currency::DUX, amount)?;
        
        // Send via DUX coin daemon
        let amount_f64 = amount as f64 / 100_000_000.0; // Convert from satoshis
        let txid = self.api.send_dux(&from_address, to_address, amount_f64).await?;
        
        info!("Sent {} DUX to {}: {}", amount_f64, to_address, txid);
        Ok(txid)
    }

    /// Get DUX transaction history
    pub async fn get_transaction_history(&self, limit: i32) -> Result<Vec<DuxTransaction>> {
        let wallet = self.wallet.lock().await;
        let dux_address = wallet.get_address(&Currency::DUX);
        
        self.api.get_transactions(&dux_address, limit).await
    }

    /// Get DUX network status
    pub async fn get_network_status(&self) -> Result<DuxNetworkInfo> {
        self.api.get_network_info().await
    }
} 