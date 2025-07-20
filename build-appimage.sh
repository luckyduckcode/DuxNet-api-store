#!/bin/bash

# DuxNet API Store AppImage Build Script
# This script builds the API server and creates a desktop AppImage

set -e

echo "🚀 Building DuxNet API Store AppImage..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

# Check if Tauri CLI is available locally
if [ ! -f "node_modules/.bin/tauri" ]; then
    print_warning "Tauri CLI not found. Installing locally..."
    npm install @tauri-apps/cli@latest
fi

# Clean previous builds
print_status "Cleaning previous builds..."
cargo clean
rm -rf src-tauri/target

# Build the API server
print_status "Building API server..."
cargo build --release

if [ $? -eq 0 ]; then
    print_success "API server built successfully!"
else
    print_error "Failed to build API server"
    exit 1
fi

# Check if static files exist
if [ ! -f "static/index.html" ]; then
    print_error "Static files not found. Please ensure static/index.html exists."
    exit 1
fi

# Build the Tauri AppImage
print_status "Building Tauri AppImage..."
cd src-tauri

# Build for Linux AppImage
../node_modules/.bin/tauri build --target appimage

if [ $? -eq 0 ]; then
    print_success "AppImage built successfully!"
    
    # Find the AppImage file
    APPIMAGE_PATH=$(find target/appimage -name "*.AppImage" 2>/dev/null | head -n 1)
    
    if [ -n "$APPIMAGE_PATH" ]; then
        print_success "AppImage created at: $APPIMAGE_PATH"
        
        # Make it executable
        chmod +x "$APPIMAGE_PATH"
        
        # Copy to project root for easy access
        cp "$APPIMAGE_PATH" "../DuxNet-API-Store.AppImage"
        print_success "AppImage copied to: DuxNet-API-Store.AppImage"
        
        # Show file size
        FILE_SIZE=$(du -h "../DuxNet-API-Store.AppImage" | cut -f1)
        print_status "AppImage size: $FILE_SIZE"
        
    else
        print_warning "AppImage file not found in expected location"
    fi
else
    print_error "Failed to build AppImage"
    exit 1
fi

cd ..

# Create a launcher script
print_status "Creating launcher script..."
cat > launch-duxnet.sh << 'EOF'
#!/bin/bash

# DuxNet API Store Launcher
# This script starts both the API server and the desktop app

echo "🚀 Starting DuxNet API Store..."

# Start the API server in the background
echo "📡 Starting API server..."
./target/release/duxnet-api-store &
API_PID=$!

# Wait a moment for the API to start
sleep 3

# Check if API is running
if curl -s http://localhost:8081/api/status > /dev/null; then
    echo "✅ API server is running on http://localhost:8081"
    
    # Launch the desktop app
    echo "🖥️  Launching desktop application..."
    ./DuxNet-API-Store.AppImage &
    
    echo "🎉 DuxNet API Store is now running!"
    echo "   - API Server: http://localhost:8081"
    echo "   - Desktop App: Running in background"
    echo ""
    echo "Press Ctrl+C to stop both services"
    
    # Wait for user to stop
    wait $API_PID
else
    echo "❌ Failed to start API server"
    kill $API_PID 2>/dev/null
    exit 1
fi
EOF

chmod +x launch-duxnet.sh
print_success "Launcher script created: launch-duxnet.sh"

# Create installation instructions
print_status "Creating installation instructions..."
cat > INSTALL.md << 'EOF'
# DuxNet API Store - Installation Guide

## Quick Start

1. **Run the launcher script:**
   ```bash
   ./launch-duxnet.sh
   ```

2. **Or run manually:**
   ```bash
   # Start API server
   ./target/release/duxnet-api-store &
   
   # Launch desktop app
   ./DuxNet-API-Store.AppImage
   ```

## Features

- 🚀 **API Server**: RESTful API for service management
- 🖥️ **Desktop GUI**: Modern interface for service registration and discovery
- 📊 **Analytics**: Usage tracking and performance monitoring
- 🔍 **Service Discovery**: Advanced search and filtering
- 🔑 **API Key Management**: Secure authentication system

## API Endpoints

- `GET /api/status` - Server status
- `GET /api/version` - API version info
- `POST /api/services/register` - Register new service
- `POST /api/services/search` - Search services
- `GET /api/analytics/usage` - Usage analytics
- `GET /api/developer/dashboard` - Developer portal

## Demo API Keys

- `demo-api-key-123` (Demo User)
- `admin-api-key-456` (Admin)
- `service-api-key-789` (Service Provider)

## System Requirements

- Linux x86_64
- 2GB RAM minimum
- 100MB disk space

## Troubleshooting

If the AppImage doesn't run:
```bash
chmod +x DuxNet-API-Store.AppImage
./DuxNet-API-Store.AppImage
```

For API issues:
```bash
curl http://localhost:8081/api/status
```
EOF

print_success "Installation guide created: INSTALL.md"

# Final summary
echo ""
print_success "🎉 DuxNet API Store AppImage build completed!"
echo ""
echo "📁 Files created:"
echo "   - DuxNet-API-Store.AppImage (Desktop application)"
echo "   - launch-duxnet.sh (Launcher script)"
echo "   - INSTALL.md (Installation guide)"
echo ""
echo "🚀 To start the application:"
echo "   ./launch-duxnet.sh"
echo ""
echo "📡 API will be available at: http://localhost:8081"
echo "🖥️  Desktop app will launch automatically"
echo ""
print_success "Build completed successfully!" 