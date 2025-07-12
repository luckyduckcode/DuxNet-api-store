# DuxNet - Decentralized P2P Platform

A powerful, decentralized peer-to-peer platform built with Rust, featuring a Tauri desktop application, P2P networking, digital identity management, and cryptocurrency wallet functionality.

## 🚀 Features

### Core Platform
- **Decentralized P2P Network**: Peer-to-peer communication without central servers
- **Digital Identity Management**: DID-based identity system with Ed25519 cryptography
- **Reputation System**: Trust-based reputation scoring and attestations
- **Escrow Contracts**: Secure multi-signature escrow system
- **Task Engine**: Distributed task processing and execution

### Wallet & Cryptocurrency
- **Multi-Currency Support**: Native DUX tokens, DUX Coin (ASIC-resistant), and multi-signature wallets
- **DUX Coin Integration**: RandomX mining, lightweight blockchain, fast confirmations
- **Transaction Management**: Secure transaction creation, signing, and verification
- **Key Management**: Private key import/export with base64 encoding
- **Balance Tracking**: Real-time balance monitoring across currencies

### Desktop Application
- **Tauri Framework**: Cross-platform desktop app with web technologies
- **Modern UI**: Beautiful, responsive user interface
- **Native Performance**: Rust backend with web frontend

### API & Networking
- **RESTful API**: Comprehensive HTTP API on port 8081
- **P2P Node**: Network node running on port 8080
- **Web Interface**: Accessible at http://localhost:8081

## 🛠️ Technology Stack

- **Backend**: Rust with Tokio async runtime
- **Frontend**: Tauri (Web technologies + Rust)
- **Cryptography**: Ed25519 for digital signatures
- **Networking**: Custom P2P protocol
- **Database**: In-memory storage with persistence
- **API**: Axum web framework

## 📋 Prerequisites

- **Rust**: Latest stable version (1.70+)
- **Node.js**: Version 16+ (for Tauri development)
- **Git**: For version control
- **Windows**: Visual Studio Build Tools (for Windows builds)

## 🚀 Quick Start

### 1. Clone the Repository
```bash
git clone <your-new-repo-url>
cd duxnet-platform
```

### 2. Build and Run
```bash
# Build the application
cargo build --release

# Run the application
cargo run --release
```

### 3. Access the Platform
- **Web Interface**: http://localhost:8081
- **API Documentation**: http://localhost:8081/api
- **P2P Node**: Port 8080

## 📦 Installation

### Windows
1. Install Rust: https://rustup.rs/
2. Install Visual Studio Build Tools
3. Clone and build the project

### Linux/macOS
1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Install system dependencies
3. Clone and build the project

## 🔧 Development

### Project Structure
```
src/
├── core/           # Core platform functionality
│   ├── identity.rs # Digital identity management
│   ├── dht.rs      # Distributed hash table
│   ├── reputation.rs # Reputation system
│   ├── escrow.rs   # Escrow contracts
│   └── tasks.rs    # Task engine
├── wallet/         # Cryptocurrency wallet
├── network/        # P2P networking
├── api/           # REST API endpoints
└── frontend/      # Web interface

src-tauri/         # Tauri desktop app
static/            # Static web assets
```

### Building for Development
```bash
# Development build
cargo build

# Run with logging
RUST_LOG=debug cargo run

# Tauri development
npm run dev
```

### Building for Production
```bash
# Release build
cargo build --release

# Tauri production build
npm run build
```

## 🔌 API Endpoints

### Identity Management
- `POST /api/identity/create` - Create new DID
- `GET /api/identity/{did}` - Get identity info
- `POST /api/identity/attest` - Create reputation attestation

### Wallet Operations
- `GET /api/wallet/balance` - Get wallet balance
- `POST /api/wallet/send` - Send transaction
- `GET /api/wallet/transactions` - Get transaction history

### P2P Network
- `GET /api/network/peers` - List connected peers
- `POST /api/network/connect` - Connect to peer
- `GET /api/network/status` - Network status

### Escrow Contracts
- `POST /api/escrow/create` - Create escrow contract
- `POST /api/escrow/sign` - Sign escrow contract
- `GET /api/escrow/{id}` - Get escrow details

## 🔐 Security Features

- **Ed25519 Cryptography**: State-of-the-art digital signatures
- **DID-based Identity**: Decentralized identifiers for users
- **Multi-signature Wallets**: Enhanced security for transactions
- **Reputation System**: Trust-based peer verification
- **Escrow Contracts**: Secure multi-party agreements

## 🌐 P2P Network

The DuxNet platform operates on a decentralized P2P network where:
- Nodes communicate directly without central servers
- Data is distributed across the network
- Reputation ensures trust between peers
- Escrow contracts provide secure transactions

## 📊 Monitoring

### Logs
The application provides detailed logging:
- Identity creation and management
- Network connections and peer discovery
- Transaction processing
- API requests and responses

### Metrics
- Active peer connections
- Transaction volume
- Network latency
- System performance

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🆘 Support

- **Issues**: Report bugs and feature requests on GitHub
- **Documentation**: Check the `/docs` folder for detailed guides
- **Community**: Join our community discussions

## 🔮 Roadmap

- [x] DUX Coin integration with ASIC-resistant mining
- [ ] Mobile application support
- [ ] Advanced consensus mechanisms
- [ ] Cross-chain interoperability
- [ ] Enhanced privacy features
- [ ] Developer SDK
- [ ] Plugin system

---

**DuxNet** - Building the future of decentralized applications with Rust and P2P technology. 