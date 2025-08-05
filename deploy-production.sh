#!/bin/bash

# DuxNet Production Deployment Script
# This script helps deploy DuxNet in a production environment

set -e

echo "🚀 DuxNet Production Deployment Script"
echo "======================================"

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   echo "❌ This script should not be run as root for security reasons"
   exit 1
fi

# Create production directories
echo "📁 Creating production directories..."
sudo mkdir -p /etc/duxnet
sudo mkdir -p /var/log/duxnet
sudo mkdir -p /var/lib/duxnet
sudo mkdir -p /var/backups/duxnet

# Copy configuration
echo "⚙️ Setting up configuration..."
if [ ! -f "/etc/duxnet/duxnet.env" ]; then
    sudo cp production.env /etc/duxnet/duxnet.env
    echo "✅ Configuration template copied to /etc/duxnet/duxnet.env"
    echo "⚠️ Please edit /etc/duxnet/duxnet.env with your production values"
else
    echo "ℹ️ Configuration already exists at /etc/duxnet/duxnet.env"
fi

# Set proper permissions
echo "🔐 Setting permissions..."
sudo chown -R $USER:$USER /var/lib/duxnet
sudo chown -R $USER:$USER /var/backups/duxnet
sudo chmod 750 /etc/duxnet
sudo chmod 640 /etc/duxnet/duxnet.env

# Build optimized release binary
echo "🔨 Building optimized release binary..."
cargo build --release

# Create systemd service
echo "🔧 Creating systemd service..."
sudo tee /etc/systemd/system/duxnet.service > /dev/null <<EOF
[Unit]
Description=DuxNet Decentralized API Marketplace
After=network.target
Wants=network.target

[Service]
Type=simple
User=$USER
Group=$USER
WorkingDirectory=$(pwd)
ExecStart=$(pwd)/target/release/duxnet
EnvironmentFile=/etc/duxnet/duxnet.env
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=duxnet

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/duxnet /var/log/duxnet /var/backups/duxnet

[Install]
WantedBy=multi-user.target
EOF

# Setup log rotation
echo "📋 Setting up log rotation..."
sudo tee /etc/logrotate.d/duxnet > /dev/null <<EOF
/var/log/duxnet/*.log {
    daily
    missingok
    rotate 52
    compress
    delaycompress
    notifempty
    create 644 $USER $USER
    postrotate
        systemctl reload duxnet || true
    endscript
}
EOF

# Setup firewall rules (if ufw is available)
if command -v ufw &> /dev/null; then
    echo "🔥 Setting up firewall rules..."
    sudo ufw allow 8081/tcp comment "DuxNet API"
    sudo ufw allow 9000/tcp comment "DuxNet P2P"
    echo "✅ Firewall rules added"
fi

# Enable and start service
echo "🏃 Enabling and starting DuxNet service..."
sudo systemctl daemon-reload
sudo systemctl enable duxnet
sudo systemctl start duxnet

# Check service status
echo "📊 Checking service status..."
sleep 3
if sudo systemctl is-active --quiet duxnet; then
    echo "✅ DuxNet service is running successfully!"
    echo "🌐 API available at: http://localhost:8081"
    echo "📊 Service status: $(sudo systemctl is-active duxnet)"
else
    echo "❌ DuxNet service failed to start"
    echo "📋 Checking logs..."
    sudo journalctl -u duxnet --no-pager -n 20
    exit 1
fi

echo ""
echo "🎉 DuxNet Production Deployment Complete!"
echo "======================================"
echo "📖 Next steps:"
echo "   1. Edit /etc/duxnet/duxnet.env with your production configuration"
echo "   2. Configure your DuxCoin node and update RPC settings"
echo "   3. Set up SSL/TLS certificates if needed"
echo "   4. Configure your reverse proxy (nginx/apache)"
echo "   5. Set up monitoring and backups"
echo ""
echo "🔧 Management commands:"
echo "   sudo systemctl status duxnet     # Check status"
echo "   sudo systemctl restart duxnet    # Restart service"
echo "   sudo systemctl stop duxnet       # Stop service"
echo "   sudo journalctl -u duxnet -f     # View live logs"
echo ""
echo "🌐 Default API endpoint: http://localhost:8081"
echo "📚 API documentation: http://localhost:8081/api/status"
