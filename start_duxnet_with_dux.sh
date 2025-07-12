#!/bin/bash

# DuxNet with DUX Coin Integration Startup Script
# This script starts both DuxNet and DUX coin daemon together

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DUXNET_PORT=8080
DUXNET_API_PORT=8081
DUX_COIN_RPC_PORT=8332
DUX_COIN_DATA_DIR="$HOME/.duxcoin"
DUX_COIN_CONF="$DUX_COIN_DATA_DIR/duxcoin.conf"

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}  DuxNet + DUX Coin Integration${NC}"
    echo -e "${BLUE}================================${NC}"
}

# Function to check if port is available
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Function to wait for service to be ready
wait_for_service() {
    local url=$1
    local max_attempts=30
    local attempt=1
    
    print_status "Waiting for service at $url..."
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s "$url" >/dev/null 2>&1; then
            print_status "Service is ready!"
            return 0
        fi
        
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    
    print_error "Service failed to start within expected time"
    return 1
}

# Function to setup DUX coin configuration
setup_dux_coin() {
    print_status "Setting up DUX coin configuration..."
    
    # Create data directory
    mkdir -p "$DUX_COIN_DATA_DIR"
    
    # Create configuration file
    cat > "$DUX_COIN_CONF" << EOF
# DUX Coin Configuration for DuxNet Integration
rpcuser=duxnet
rpcpassword=duxnet_secure_password_$(date +%s)
rpcport=$DUX_COIN_RPC_PORT
rpcallowip=127.0.0.1
rpcbind=127.0.0.1

# Network settings
maxconnections=50
maxuploadtarget=5000
dbcache=64
par=2
txindex=1

# Server settings
server=1
daemon=1
listen=1
bind=127.0.0.1

# Mining settings
gen=0
genproclimit=2

# Logging
debug=0
logips=0
EOF

    print_status "DUX coin configuration created at $DUX_COIN_CONF"
}

# Function to start DUX coin daemon
start_dux_coin() {
    print_status "Starting DUX coin daemon..."
    
    # Check if DUX coin daemon is already running
    if pgrep -f "duxnetd" >/dev/null; then
        print_warning "DUX coin daemon is already running"
        return 0
    fi
    
    # Check if duxnetd exists
    if [ ! -f "./duxcoin/src/duxnetd" ]; then
        print_error "DUX coin daemon not found. Please build DUX coin first."
        print_status "Run: cd duxcoin && ./autogen.sh && ./configure --enable-lightweight && make"
        return 1
    fi
    
    # Start DUX coin daemon
    ./duxcoin/src/duxnetd -conf="$DUX_COIN_CONF" &
    DUX_COIN_PID=$!
    
    # Wait for DUX coin to start
    sleep 5
    
    if kill -0 $DUX_COIN_PID 2>/dev/null; then
        print_status "DUX coin daemon started (PID: $DUX_COIN_PID)"
        echo $DUX_COIN_PID > /tmp/dux_coin.pid
    else
        print_error "Failed to start DUX coin daemon"
        return 1
    fi
}

# Function to start DuxNet
start_duxnet() {
    print_status "Starting DuxNet platform..."
    
    # Check if DuxNet is already running
    if check_port $DUXNET_API_PORT; then
        print_warning "DuxNet API is already running on port $DUXNET_API_PORT"
        return 0
    fi
    
    # Start DuxNet in background
    cargo run --release &
    DUXNET_PID=$!
    
    # Wait for DuxNet to start
    sleep 3
    
    if kill -0 $DUXNET_PID 2>/dev/null; then
        print_status "DuxNet started (PID: $DUXNET_PID)"
        echo $DUXNET_PID > /tmp/duxnet.pid
    else
        print_error "Failed to start DuxNet"
        return 1
    fi
}

# Function to check services
check_services() {
    print_status "Checking service status..."
    
    # Check DUX coin RPC
    if check_port $DUX_COIN_RPC_PORT; then
        print_status "✓ DUX coin RPC is running on port $DUX_COIN_RPC_PORT"
    else
        print_error "✗ DUX coin RPC is not running"
    fi
    
    # Check DuxNet API
    if check_port $DUXNET_API_PORT; then
        print_status "✓ DuxNet API is running on port $DUXNET_API_PORT"
    else
        print_error "✗ DuxNet API is not running"
    fi
    
    # Check DuxNet P2P
    if check_port $DUXNET_PORT; then
        print_status "✓ DuxNet P2P is running on port $DUXNET_PORT"
    else
        print_error "✗ DuxNet P2P is not running"
    fi
}

# Function to show usage information
show_usage() {
    print_status "Services are now running:"
    echo
    echo -e "${BLUE}DUX Coin:${NC}"
    echo "  RPC Endpoint: http://127.0.0.1:$DUX_COIN_RPC_PORT"
    echo "  Username: duxnet"
    echo "  Password: duxnet_secure_password_*"
    echo
    echo -e "${BLUE}DuxNet:${NC}"
    echo "  Web Interface: http://localhost:$DUXNET_API_PORT"
    echo "  API Endpoints: http://localhost:$DUXNET_API_PORT/api"
    echo "  P2P Node: localhost:$DUXNET_PORT"
    echo
    echo -e "${BLUE}DUX Coin API:${NC}"
    echo "  Balance: GET http://localhost:$DUXNET_API_PORT/api/dux/balance"
    echo "  Send: POST http://localhost:$DUXNET_API_PORT/api/dux/send"
    echo "  Network: GET http://localhost:$DUXNET_API_PORT/api/dux/network"
    echo "  Mining: POST http://localhost:$DUXNET_API_PORT/api/dux/mine/start"
    echo
    echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}"
}

# Function to cleanup on exit
cleanup() {
    print_status "Shutting down services..."
    
    # Stop DuxNet
    if [ -f /tmp/duxnet.pid ]; then
        DUXNET_PID=$(cat /tmp/duxnet.pid)
        if kill -0 $DUXNET_PID 2>/dev/null; then
            kill $DUXNET_PID
            print_status "DuxNet stopped"
        fi
        rm -f /tmp/duxnet.pid
    fi
    
    # Stop DUX coin
    if [ -f /tmp/dux_coin.pid ]; then
        DUX_COIN_PID=$(cat /tmp/dux_coin.pid)
        if kill -0 $DUX_COIN_PID 2>/dev/null; then
            kill $DUX_COIN_PID
            print_status "DUX coin daemon stopped"
        fi
        rm -f /tmp/dux_coin.pid
    fi
    
    # Stop any remaining duxnetd processes
    pkill -f "duxnetd" 2>/dev/null || true
    
    print_status "All services stopped"
    exit 0
}

# Main execution
main() {
    print_header
    
    # Set up signal handlers
    trap cleanup SIGINT SIGTERM
    
    # Check prerequisites
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo is not installed"
        exit 1
    fi
    
    if ! command -v curl &> /dev/null; then
        print_error "curl is not installed"
        exit 1
    fi
    
    # Setup and start services
    setup_dux_coin
    start_dux_coin
    start_duxnet
    
    # Wait for services to be ready
    sleep 5
    
    # Check service status
    check_services
    
    # Show usage information
    show_usage
    
    # Keep script running
    while true; do
        sleep 10
        
        # Check if services are still running
        if [ -f /tmp/duxnet.pid ] && ! kill -0 $(cat /tmp/duxnet.pid) 2>/dev/null; then
            print_error "DuxNet has stopped unexpectedly"
            cleanup
        fi
        
        if [ -f /tmp/dux_coin.pid ] && ! kill -0 $(cat /tmp/dux_coin.pid) 2>/dev/null; then
            print_error "DUX coin daemon has stopped unexpectedly"
            cleanup
        fi
    done
}

# Run main function
main "$@" 