# DuxNet Community Fund Implementation

## Overview

The DuxNet Community Fund is a decentralized system that automatically collects a 5% transaction tax from all cryptocurrency transactions (excluding DUX) and distributes the collected funds equally to all active users every 12 hours. This implementation ensures complete decentralization, transparency, and fair distribution.

## Key Features

### 1. Automatic Tax Collection
- **5% Transaction Tax**: Every transaction in supported currencies (BTC, ETH, USDC, LTC, XMR, DOGE) automatically deducts 5% for the community fund
- **DUX Exclusion**: DUX cryptocurrency transactions are exempt from the tax
- **Transparent Process**: Tax amounts are clearly displayed in transaction details

### 2. Decentralized Distribution
- **12-Hour Schedule**: Funds are distributed every 12 hours automatically
- **Equal Distribution**: All collected funds are split equally among active users
- **Multi-Signature Security**: Distributions require multiple trusted node signatures
- **DHT Storage**: All fund data is stored in the Distributed Hash Table for redundancy

### 3. Real-Time Monitoring
- **Live Statistics**: View current fund balances, distribution history, and next distribution countdown
- **Currency-Specific Tracking**: Separate tracking for each supported cryptocurrency
- **USD Value Conversion**: Automatic conversion to USD for total fund value display

## Architecture

### Core Components

#### 1. Community Fund Manager (`src/core/community_fund.rs`)
```rust
pub struct CommunityFundManager {
    funds: Arc<RwLock<HashMap<Currency, CommunityFund>>>,
    dht: Arc<crate::core::dht::DHT>,
    distribution_interval: u64, // 12 hours in seconds
    trusted_nodes: Vec<String>, // DIDs of trusted nodes
}
```

**Key Methods:**
- `calculate_tax_amount(amount: u64) -> u64`: Calculates 5% tax
- `add_tax_to_fund(currency: Currency, tax_amount: u64)`: Adds tax to fund
- `distribute_fund(currency: Currency)`: Distributes funds to all users
- `get_stats() -> CommunityFundStats`: Returns comprehensive statistics

#### 2. Enhanced Wallet (`src/wallet/mod.rs`)
The wallet now automatically:
- Calculates 5% tax on all outgoing transactions
- Creates separate tax transactions
- Displays tax information in transaction details
- Excludes DUX from tax calculations

#### 3. Task Engine Integration (`src/core/tasks.rs`)
- Automatically checks for distribution eligibility every task cycle
- Schedules distributions when 12-hour intervals are reached
- Handles distribution across all supported currencies

#### 4. DHT Storage (`src/core/dht.rs`)
- Stores fund balances and state
- Tracks distribution timestamps
- Maintains active user lists
- Records distribution transactions

### Data Structures

#### Community Fund
```rust
pub struct CommunityFund {
    pub currency: Currency,
    pub balance: u64,
    pub last_distribution: u64,
    pub total_distributed: u64,
    pub distribution_count: u64,
}
```

#### Community Fund Stats
```rust
pub struct CommunityFundStats {
    pub total_balance_usd: f64,
    pub currencies: Vec<CommunityFundBalance>,
    pub next_distribution_in: u64,
    pub total_distributed_all_time: u64,
}
```

## API Endpoints

### 1. Get Community Fund Statistics
```http
GET /api/community_fund/stats
```

**Response:**
```json
{
  "success": true,
  "data": {
    "total_balance_usd": 1250.50,
    "currencies": [
      {
        "currency": "BTC",
        "balance": 50000000,
        "formatted_balance": "0.5 BTC",
        "last_distribution": 1640995200,
        "next_distribution": 43200,
        "total_distributed": 100000000,
        "distribution_count": 5
      }
    ],
    "next_distribution_in": 43200,
    "total_distributed_all_time": 500000000
  }
}
```

### 2. Get Currency-Specific Balance
```http
GET /api/community_fund/balance/{currency}
```

**Response:**
```json
{
  "success": true,
  "currency": "BTC",
  "balance": 50000000,
  "formatted_balance": "0.5 BTC"
}
```

### 3. Manual Distribution (Admin)
```http
POST /api/community_fund/distribute/{currency}
```

**Response:**
```json
{
  "success": true,
  "message": "Distributed 0.1 BTC to 5 users",
  "data": {
    "currency": "BTC",
    "amount_per_user": 20000000,
    "total_users": 5,
    "distribution_timestamp": 1640995200,
    "transaction_ids": ["cf-dist-btc-user1-1640995200"]
  }
}
```

## Frontend Integration

### Community Fund Dashboard
The frontend includes a dedicated Community Fund section with:

1. **Overview Tab**: Displays all currency balances and statistics
2. **Distribution Tab**: Allows manual distribution (admin only)

### Real-Time Updates
- Automatic refresh of fund statistics
- Live countdown to next distribution
- Transaction history with tax information
- USD value conversion for total fund value

### Key JavaScript Functions

```javascript
// Refresh community fund statistics
async function refreshCommunityFundStats() {
    const response = await fetch('/api/community_fund/stats');
    const data = await response.json();
    if (data.success) {
        displayCommunityFundStats(data.data);
    }
}

// Manual distribution
async function distributeCommunityFund() {
    const currency = document.getElementById('distributionCurrency').value;
    const response = await fetch(`/api/community_fund/distribute/${currency}`, {
        method: 'POST'
    });
    const data = await response.json();
    // Handle response...
}
```

## Usage Examples

### 1. Sending Funds (Automatic Tax Collection)
When a user sends 100 USDC:
- **Amount Sent**: 95 USDC (to recipient)
- **Tax Collected**: 5 USDC (to community fund)
- **Total Deducted**: 100 USDC

### 2. Distribution Process
Every 12 hours, the system:
1. Checks if distribution is due for each currency
2. Retrieves active user list from DHT
3. Calculates amount per user: `total_fund / number_of_users`
4. Creates distribution transactions
5. Updates fund state and timestamps
6. Stores transaction records in DHT

### 3. Monitoring Fund Status
```bash
# Check current fund balances
curl http://localhost:8081/api/community_fund/stats

# Check specific currency balance
curl http://localhost:8081/api/community_fund/balance/BTC

# Manual distribution (if needed)
curl -X POST http://localhost:8081/api/community_fund/distribute/USDC
```

## Security Features

### 1. Multi-Signature Distribution
- Distributions require signatures from trusted nodes
- Prevents single-point-of-failure attacks
- Ensures consensus on fund distributions

### 2. DHT Redundancy
- All fund data stored in Distributed Hash Table
- Multiple copies across network nodes
- Automatic recovery from node failures

### 3. Transaction Verification
- All tax transactions are cryptographically signed
- Verification of fund transfers
- Audit trail for all distributions

### 4. Rate Limiting
- Distribution can only occur every 12 hours
- Prevents spam or abuse of distribution system
- Automatic scheduling prevents conflicts

## Configuration

### Distribution Interval
```rust
// In src/core/community_fund.rs
distribution_interval: 12 * 60 * 60, // 12 hours in seconds
```

### Trusted Nodes
```rust
trusted_nodes: vec![
    "did:duxnet:trusted-node-1".to_string(),
    "did:duxnet:trusted-node-2".to_string(),
    "did:duxnet:trusted-node-3".to_string(),
],
```

### Supported Currencies
```rust
// All currencies except DUX are taxed
[Currency::BTC, Currency::ETH, Currency::USDC, Currency::LTC, Currency::XMR, Currency::DOGE]
```

## Testing

### Unit Tests
```bash
# Run all tests
cargo test

# Run community fund specific tests
cargo test community_fund
```

### Integration Tests
```bash
# Test API endpoints
curl -X POST http://localhost:8081/api/wallet/send \
  -H "Content-Type: application/json" \
  -d '{"to_address":"test","amount":1000000,"currency":"USDC"}'

# Verify tax was collected
curl http://localhost:8081/api/community_fund/balance/USDC
```

## Monitoring and Maintenance

### 1. Fund Health Checks
- Monitor fund balances across all currencies
- Track distribution success rates
- Alert on failed distributions

### 2. Performance Metrics
- Distribution processing time
- DHT storage efficiency
- Network consensus speed

### 3. Audit Logs
- All tax collections logged
- Distribution transactions recorded
- User participation tracked

## Future Enhancements

### 1. Dynamic Tax Rates
- Adjustable tax rates based on network conditions
- Community voting on tax rate changes
- Automatic rate adjustments

### 2. Advanced Distribution
- Weighted distributions based on user activity
- Merit-based fund allocation
- Staking requirements for distributions

### 3. Enhanced Analytics
- Historical fund performance
- User participation analytics
- Economic impact analysis

## Troubleshooting

### Common Issues

1. **Distribution Not Triggering**
   - Check if 12 hours have passed since last distribution
   - Verify trusted nodes are online
   - Check DHT connectivity

2. **Tax Not Being Collected**
   - Ensure transaction is not in DUX currency
   - Verify wallet integration is working
   - Check transaction amount is sufficient

3. **API Endpoints Not Responding**
   - Verify API server is running on port 8081
   - Check CORS configuration
   - Ensure proper authentication

### Debug Commands
```bash
# Check node status
curl http://localhost:8081/api/status

# View transaction history
curl http://localhost:8081/api/wallet/transactions

# Check DHT health
curl http://localhost:8081/api/stats
```

## Conclusion

The DuxNet Community Fund implementation provides a robust, decentralized system for automatic tax collection and fair distribution. The system ensures transparency, security, and equal participation for all network users while maintaining the decentralized ethos of the DuxNet platform.

For more information, refer to the main DuxNet documentation and API reference. 