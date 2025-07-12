use crate::core::data_structures::*;
use crate::wallet::{Currency, Wallet, MultiSigWallet};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn};
use serde::{Deserialize, Serialize};

pub struct CommunityFundManager {
    funds: Arc<RwLock<HashMap<Currency, CommunityFund>>>,
    dht: Arc<crate::core::dht::DHT>,
    distribution_interval: u64, // 12 hours in seconds
    trusted_nodes: Vec<String>, // DIDs of trusted nodes for signing distributions
}

impl CommunityFundManager {
    pub fn new(dht: Arc<crate::core::dht::DHT>) -> Self {
        let mut funds = HashMap::new();
        
        // Initialize community funds for all supported currencies (excluding DUX)
        for currency in [Currency::BTC, Currency::ETH, Currency::USDC, Currency::LTC, Currency::XMR, Currency::DOGE] {
            funds.insert(currency, CommunityFund {
                currency,
                balance: 0,
                last_distribution: 0,
                total_distributed: 0,
                distribution_count: 0,
            });
        }
        
        Self {
            funds: Arc::new(RwLock::new(funds)),
            dht,
            distribution_interval: 12 * 60 * 60, // 12 hours
            trusted_nodes: vec![
                "did:duxnet:trusted-node-1".to_string(),
                "did:duxnet:trusted-node-2".to_string(),
                "did:duxnet:trusted-node-3".to_string(),
            ],
        }
    }

    /// Calculate the 5% tax amount for a transaction
    pub fn calculate_tax_amount(&self, amount: u64) -> u64 {
        (amount * 5) / 100 // 5% tax
    }

    /// Add tax to community fund for a specific currency
    pub async fn add_tax_to_fund(&self, currency: Currency, tax_amount: u64) -> Result<()> {
        let mut funds = self.funds.write().await;
        
        if let Some(fund) = funds.get_mut(&currency) {
            fund.balance += tax_amount;
            info!("Added {} {} tax to community fund. New balance: {}", 
                  tax_amount, currency.symbol(), fund.balance);
            
            // Store updated balance in DHT
            self.store_fund_balance(&currency, fund.balance).await?;
        }
        
        Ok(())
    }

    /// Get community fund balance for a currency
    pub async fn get_fund_balance(&self, currency: &Currency) -> u64 {
        let funds = self.funds.read().await;
        funds.get(currency).map(|f| f.balance).unwrap_or(0)
    }

    /// Get all community fund balances
    pub async fn get_all_fund_balances(&self) -> HashMap<Currency, CommunityFund> {
        let funds = self.funds.read().await;
        funds.clone()
    }

    /// Check if community fund should be distributed
    pub async fn should_distribute(&self, currency: &Currency) -> bool {
        // First check if the fund exists and has balance
        let fund_balance = {
            let funds = self.funds.read().await;
            if let Some(fund) = funds.get(currency) {
                let now = get_current_timestamp();
                let time_since_last = now.saturating_sub(fund.last_distribution);
                
                // Check if enough time has passed
                if time_since_last < self.distribution_interval {
                    return false;
                }
                
                // Check if there are funds to distribute
                if fund.balance == 0 {
                    info!("Community fund for {} has no balance to distribute", currency.symbol());
                    return false;
                }
                
                fund.balance
            } else {
                return false;
            }
        };
        
        // Now check if there are active users to distribute to
        match self.get_active_dids().await {
            Ok(active_dids) => {
                if active_dids.is_empty() {
                    info!("No active users found for {} community fund distribution", currency.symbol());
                    return false;
                }
                
                // Check if the amount per user would be meaningful
                let amount_per_user = fund_balance / active_dids.len() as u64;
                if amount_per_user == 0 {
                    info!("{} community fund balance ({}) too low for {} users ({} per user)", 
                          currency.symbol(), fund_balance, active_dids.len(), amount_per_user);
                    return false;
                }
                
                true
            }
            Err(e) => {
                warn!("Failed to get active DIDs for {} distribution check: {}", currency.symbol(), e);
                false
            }
        }
    }

    /// Distribute community fund to all users
    pub async fn distribute_fund(&self, currency: Currency) -> Result<CommunityFundDistribution> {
        let mut funds = self.funds.write().await;
        
        let fund = funds.get_mut(&currency).ok_or_else(|| {
            anyhow::anyhow!("Community fund not found for currency: {}", currency.symbol())
        })?;

        let now = get_current_timestamp();
        
        // Check if enough time has passed
        let time_since_last = now.saturating_sub(fund.last_distribution);
        if time_since_last < self.distribution_interval {
            return Err(anyhow::anyhow!("Distribution not yet due. Next distribution in {} seconds", 
                                      self.distribution_interval - time_since_last));
        }

        // Check if there are funds to distribute
        if fund.balance == 0 {
            return Err(anyhow::anyhow!("No funds available for distribution"));
        }

        // Get all active DIDs from DHT
        let active_dids = self.get_active_dids().await?;
        
        if active_dids.is_empty() {
            warn!("No active DIDs found for community fund distribution");
            return Err(anyhow::anyhow!("No active users to distribute to"));
        }

        // Calculate amount per user
        let amount_per_user = fund.balance / active_dids.len() as u64;
        
        if amount_per_user == 0 {
            warn!("Community fund balance too low for distribution: {} {} to {} users", 
                  fund.balance, currency.symbol(), active_dids.len());
            return Err(anyhow::anyhow!("Insufficient balance for meaningful distribution"));
        }

        info!("Starting distribution of {} {} to {} users ({} per user)", 
              fund.balance, currency.symbol(), active_dids.len(), amount_per_user);

        // Create distribution transactions
        let mut transaction_ids = Vec::new();
        let mut successful_distributions = 0;
        
        for did in &active_dids {
            match self.create_distribution_transaction(&currency, did, amount_per_user).await {
                Ok(tx_id) => {
                    transaction_ids.push(tx_id.clone());
                    successful_distributions += 1;
                    info!("Created distribution transaction {} for DID: {}", tx_id, did);
                }
                Err(e) => {
                    error!("Failed to create distribution transaction for DID {}: {}", did, e);
                }
            }
        }

        // Only update fund state if we had successful distributions
        if successful_distributions > 0 {
            let total_distributed = amount_per_user * successful_distributions as u64;
            
            // Update fund state
            fund.balance = fund.balance.saturating_sub(total_distributed);
            fund.last_distribution = now;
            fund.total_distributed += total_distributed;
            fund.distribution_count += 1;

            // Store updated state in DHT
            self.store_fund_state(&currency, fund).await?;

            let distribution = CommunityFundDistribution {
                currency,
                amount_per_user,
                total_users: successful_distributions,
                distribution_timestamp: now,
                transaction_ids,
            };

            info!("Successfully distributed {} {} to {} users ({} per user). Remaining balance: {}", 
                  total_distributed, currency.symbol(), successful_distributions, amount_per_user, fund.balance);

            Ok(distribution)
        } else {
            Err(anyhow::anyhow!("Failed to create any distribution transactions"))
        }
    }

    /// Get community fund statistics
    pub async fn get_stats(&self) -> Result<CommunityFundStats> {
        let funds = self.funds.read().await;
        let mut currencies = Vec::new();
        let mut total_balance_usd = 0.0;

        for (currency, fund) in funds.iter() {
            let now = get_current_timestamp();
            let next_distribution = fund.last_distribution + self.distribution_interval;
            let next_distribution_in = if next_distribution > now {
                next_distribution - now
            } else {
                0
            };

            let balance_info = CommunityFundBalance {
                currency: currency.symbol().to_string(),
                balance: fund.balance,
                formatted_balance: currency.format_amount(fund.balance),
                last_distribution: fund.last_distribution,
                next_distribution: next_distribution_in,
                total_distributed: fund.total_distributed,
                distribution_count: fund.distribution_count,
            };

            currencies.push(balance_info);
            
            // Convert to USD for total (simplified conversion)
            let usd_value = match currency {
                Currency::BTC => fund.balance as f64 * 50000.0 / 10f64.powi(8), // $50k per BTC
                Currency::ETH => fund.balance as f64 * 3000.0 / 10f64.powi(18), // $3k per ETH
                Currency::USDC => fund.balance as f64 / 10f64.powi(6), // 1:1 USD
                Currency::LTC => fund.balance as f64 * 100.0 / 10f64.powi(8), // $100 per LTC
                Currency::XMR => fund.balance as f64 * 200.0 / 10f64.powi(12), // $200 per XMR
                Currency::DOGE => fund.balance as f64 * 0.1 / 10f64.powi(8), // $0.1 per DOGE
            };
            
            total_balance_usd += usd_value;
        }

        let total_distributed_all_time = funds.values().map(|f| f.total_distributed).sum();

        Ok(CommunityFundStats {
            total_balance_usd,
            currencies: currencies.clone(),
            next_distribution_in: currencies.iter().map(|c| c.next_distribution).min().unwrap_or(0),
            total_distributed_all_time,
        })
    }

    /// Get active DIDs from DHT
    async fn get_active_dids(&self) -> Result<Vec<String>> {
        Ok(self.dht.get_active_dids().await)
    }

    /// Create a distribution transaction
    async fn create_distribution_transaction(&self, currency: &Currency, recipient_did: &str, amount: u64) -> Result<String> {
        // In a real implementation, this would create an actual blockchain transaction
        // For now, generate a mock transaction ID
        let tx_id = format!("cf-dist-{}-{}-{}", currency.symbol().to_lowercase(), 
                           recipient_did.replace(":", "-"), 
                           get_current_timestamp());
        
        // Store transaction in DHT for tracking
        self.dht.store_community_fund_transaction(&tx_id, currency, recipient_did, amount).await?;
        
        Ok(tx_id)
    }

    /// Store fund balance in DHT
    async fn store_fund_balance(&self, currency: &Currency, balance: u64) -> Result<()> {
        let key = format!("community_fund_balance_{}", currency.symbol());
        let value = serde_json::to_string(&balance)?;
        self.dht.as_ref().store(key, value.into_bytes(), 86400).await?;
        Ok(())
    }

    /// Store fund state in DHT
    async fn store_fund_state(&self, currency: &Currency, fund: &CommunityFund) -> Result<()> {
        let key = format!("community_fund_state_{}", currency.symbol());
        let value = serde_json::to_string(fund)?;
        self.dht.as_ref().store(key, value.into_bytes(), 86400).await?;
        Ok(())
    }

    /// Load fund state from DHT
    pub async fn load_fund_state(&self, currency: &Currency) -> Result<()> {
        let key = format!("community_fund_state_{}", currency.symbol());
        
        if let Some(data) = self.dht.get(&key).await {
            if let Ok(fund) = serde_json::from_slice::<CommunityFund>(&data) {
                let mut funds = self.funds.write().await;
                funds.insert(currency.clone(), fund.clone());
                info!("Loaded community fund state for {}: balance {}", 
                      currency.symbol(), fund.balance);
            }
        }
        
        Ok(())
    }

    /// Load all fund states from DHT
    pub async fn load_all_fund_states(&self) -> Result<()> {
        for currency in [Currency::BTC, Currency::ETH, Currency::USDC, Currency::LTC, Currency::XMR, Currency::DOGE] {
            self.load_fund_state(&currency).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::dht::DHT;
    use crate::core::data_structures::NodeId;

    #[tokio::test]
    async fn test_tax_calculation() {
        let dht = Arc::new(DHT::new(NodeId("test-node".to_string())));
        let cf_manager = CommunityFundManager::new(dht);
        
        // Test 5% tax calculation
        assert_eq!(cf_manager.calculate_tax_amount(100), 5);
        assert_eq!(cf_manager.calculate_tax_amount(1000), 50);
        assert_eq!(cf_manager.calculate_tax_amount(10000), 500);
    }

    #[tokio::test]
    async fn test_add_tax_to_fund() {
        let dht = Arc::new(DHT::new(NodeId("test-node".to_string())));
        let cf_manager = CommunityFundManager::new(dht);
        
        // Add tax to BTC fund
        cf_manager.add_tax_to_fund(Currency::BTC, 1000000).await.unwrap();
        
        // Check balance
        let balance = cf_manager.get_fund_balance(&Currency::BTC).await;
        assert_eq!(balance, 1000000);
        
        // Add more tax
        cf_manager.add_tax_to_fund(Currency::BTC, 500000).await.unwrap();
        let balance = cf_manager.get_fund_balance(&Currency::BTC).await;
        assert_eq!(balance, 1500000);
    }

    #[tokio::test]
    async fn test_should_distribute() {
        let dht = Arc::new(DHT::new(NodeId("test-node".to_string())));
        let cf_manager = CommunityFundManager::new(dht);
        
        // Initially should not distribute (no time has passed)
        assert!(!cf_manager.should_distribute(&Currency::BTC).await);
        
        // Add some funds
        cf_manager.add_tax_to_fund(Currency::BTC, 1000000).await.unwrap();
        
        // Still should not distribute (not enough time passed)
        assert!(!cf_manager.should_distribute(&Currency::BTC).await);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let dht = Arc::new(DHT::new(NodeId("test-node".to_string())));
        let cf_manager = CommunityFundManager::new(dht);
        
        // Add funds to multiple currencies
        cf_manager.add_tax_to_fund(Currency::BTC, 1000000).await.unwrap();
        cf_manager.add_tax_to_fund(Currency::USDC, 5000000).await.unwrap();
        
        let stats = cf_manager.get_stats().await.unwrap();
        
        // Should have 6 currencies (all supported)
        assert_eq!(stats.currencies.len(), 6);
        
        // Check that BTC and USDC have balances
        let btc_currency = stats.currencies.iter().find(|c| c.currency == "BTC").unwrap();
        let usdc_currency = stats.currencies.iter().find(|c| c.currency == "USDC").unwrap();
        
        assert_eq!(btc_currency.balance, 1000000);
        assert_eq!(usdc_currency.balance, 5000000);
    }
} 