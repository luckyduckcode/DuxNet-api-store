# DUX Coin Integration with DuxNet

## Overview

DUX Coin has been successfully integrated into the DuxNet decentralized P2P platform, providing a lightweight, ASIC-resistant cryptocurrency that can be used for transactions, mining, and API payments across the network.

## Features

### 🪙 DUX Coin Characteristics
- **ASIC-Resistant Mining**: Uses RandomX algorithm for CPU-only mining
- **Lightweight**: Optimized for minimal resource usage
- **Fast Confirmations**: 2-minute block time with 10-block maturity
- **Low Fees**: 0.005 DUX transaction fees for API usage
- **No Community Fund Tax**: DUX transactions are tax-free

### 🔗 Network Integration
- **P2P Network**: Fully integrated with DuxNet's peer-to-peer network
- **API Access**: Available through both HTTP API and Tauri desktop commands
- **Wallet Integration**: Native support in DuxNet wallet
- **Cross-Platform**: Works on both web and desktop applications

## API Endpoints

### DUX Coin API (HTTP)

#### Get DUX Balance
```http
GET /api/dux/balance
```
**Response:**
```json
{
  "address": "D1234567890abcdef...",
  "balance": 50000000000,
  "formatted_balance": "500.00000000 DUX",
  "currency": "DUX",
  "success": true
}
```

#### Get DUX Transactions
```http
GET /api/dux/transactions
```
**Response:**
```json
{
  "transactions": [...],
  "count": 5,
  "currency": "DUX",
  "success": true
}
```

#### Send DUX
```http
POST /api/dux/send
Content-Type: application/json

{
  "to_address": "D9876543210fedcba...",
  "amount": 100000000,
  "memo": "Payment for API service"
}
```

#### Get DUX Network Status
```http
GET /api/dux/network
```
**Response:**
```json
{
  "difficulty": 1.0,
  "block_height": 1234,
  "connections": 8,
  "hash_rate": 2.5,
  "currency": "DUX",
  "success": true
}
```

#### Mining Controls
```http
POST /api/dux/mine/start
{
  "threads": 4
}

POST /api/dux/mine/stop

GET /api/dux/mine/status
```

#### Sync DUX Balance
```http
POST /api/dux/sync
```

### Tauri Desktop Commands

#### Get DUX Balance
```javascript
const balance = await invoke('get_dux_balance');
```

#### Send DUX
```javascript
const result = await invoke('send_dux', {
  request: {
    to_address: "D1234567890abcdef...",
    amount: 100000000,
    memo: "Payment"
  }
});
```

#### Get DUX Network Status
```javascript
const network = await invoke('get_dux_network');
```

#### Mining Controls
```javascript
// Start mining
await invoke('start_dux_mining', {
  request: { threads: 4 }
});

// Stop mining
await invoke('stop_dux_mining');

// Get mining status
const status = await invoke('get_dux_mining_status');
```

## P2P Network Integration

### DUX Coin Daemon Integration

The DUX coin daemon runs alongside the DuxNet node and provides:

1. **RPC Interface**: Lightweight JSON-RPC API for DUX operations
2. **Network Sync**: Automatic blockchain synchronization
3. **Mining Support**: CPU-based RandomX mining
4. **Transaction Processing**: Fast transaction validation and propagation

### Configuration

The DUX coin daemon uses the following configuration:

```bash
# Start DUX coin daemon with optimized settings
./duxnetd \
  --rpcuser=duxnet \
  --rpcpassword=secure_password \
  --rpcport=8332 \
  --rpcallowip=127.0.0.1 \
  --maxconnections=50 \
  --maxuploadtarget=5000 \
  --dbcache=64 \
  --par=2 \
  --txindex=1 \
  --server=1 \
  --daemon=1
```

### Network Parameters

- **Block Time**: 2 minutes (120 seconds)
- **Difficulty Adjustment**: Every 10 blocks
- **Maturity**: 10 blocks
- **Max Supply**: 100 billion DUX
- **Initial Block Reward**: 1000 DUX
- **Halving**: Every 210,000 blocks

## Wallet Integration

### DUX Currency Support

The DuxNet wallet now supports DUX coin with:

- **Address Generation**: DUX addresses start with 'D'
- **Balance Tracking**: Real-time balance updates
- **Transaction History**: Complete transaction records
- **Fee Calculation**: Low 0.005 DUX fees
- **Initial Balance**: 50,000 DUX for new wallets

### Currency Properties

```rust
Currency::DUX => {
    symbol: "DUX",
    name: "DUX Coin",
    decimals: 8,
    initial_balance: 50000,
    address_prefix: "D"
}
```

## Usage Examples

### 1. Send DUX via API

```bash
curl -X POST http://localhost:8081/api/dux/send \
  -H "Content-Type: application/json" \
  -d '{
    "to_address": "D1234567890abcdef...",
    "amount": 100000000,
    "memo": "Payment for service"
  }'
```

### 2. Get DUX Balance via Tauri

```javascript
// In the desktop app
const balance = await invoke('get_dux_balance');
console.log(`DUX Balance: ${balance.formatted_balance}`);
```

### 3. Start DUX Mining

```bash
curl -X POST http://localhost:8081/api/dux/mine/start \
  -H "Content-Type: application/json" \
  -d '{"threads": 4}'
```

### 4. Monitor DUX Network

```bash
curl http://localhost:8081/api/dux/network
```

## Development

### Building DUX Coin

```bash
cd duxcoin
./autogen.sh
./configure --enable-lightweight
make -j$(nproc)
```

### Running DUX Coin Daemon

```bash
# Start the daemon
./src/duxnetd -daemon

# Check status
./src/duxnet-cli getinfo

# Generate new address
./src/duxnet-cli getnewaddress
```

### Integration Testing

```bash
# Test DUX API endpoints
curl http://localhost:8081/api/dux/balance
curl http://localhost:8081/api/dux/network

# Test mining
curl -X POST http://localhost:8081/api/dux/mine/start \
  -H "Content-Type: application/json" \
  -d '{"threads": 2}'
```

## Security Considerations

1. **RPC Security**: Always use strong passwords for RPC access
2. **Network Security**: DUX coin uses the same security model as Bitcoin
3. **Wallet Security**: Private keys are encrypted and stored securely
4. **Mining Security**: RandomX prevents ASIC mining attacks

## Performance Optimizations

### Lightweight Configuration

- **Memory Usage**: Reduced to 64MB cache
- **CPU Usage**: Optimized for 2-4 threads
- **Network**: Limited to 50 connections
- **Storage**: Minimal blockchain storage

### API Performance

- **Caching**: 30-second cache for network data
- **Connection Pooling**: Reusable HTTP connections
- **Async Processing**: Non-blocking API calls
- **Error Handling**: Graceful degradation

## Troubleshooting

### Common Issues

1. **RPC Connection Failed**
   - Check if DUX daemon is running
   - Verify RPC credentials
   - Check firewall settings

2. **Mining Not Starting**
   - Ensure sufficient CPU resources
   - Check mining thread count
   - Verify network connectivity

3. **Balance Not Syncing**
   - Check blockchain synchronization
   - Verify address format
   - Check transaction confirmations

### Debug Commands

```bash
# Check DUX daemon status
./src/duxnet-cli getinfo

# Check blockchain sync
./src/duxnet-cli getblockchaininfo

# Check mining status
./src/duxnet-cli getmininginfo

# Check network connections
./src/duxnet-cli getconnectioncount
```

## Future Enhancements

1. **Smart Contracts**: DUX coin smart contract support
2. **Privacy Features**: Optional privacy enhancements
3. **Cross-Chain**: Interoperability with other blockchains
4. **DeFi Integration**: Decentralized finance features
5. **Mobile Support**: Mobile wallet applications

## Contributing

To contribute to DUX coin integration:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

DUX coin integration is licensed under the MIT License, same as DuxNet.

## Support

For support with DUX coin integration:

- **Documentation**: Check this README and code comments
- **Issues**: Report bugs on GitHub
- **Discussions**: Join community discussions
- **Development**: Contribute to the project

---

**DUX Coin**: ASIC-resistant, lightweight, and integrated with DuxNet for seamless P2P transactions and API payments. 