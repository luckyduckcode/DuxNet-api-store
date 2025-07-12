#!/bin/bash

# DuxNet Wallet AppImage Build Script
set -e

echo "🚀 Starting DuxNet Wallet AppImage build..."

# Check if we're on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "❌ This script is designed for Linux systems only."
    exit 1
fi

# Check for required dependencies
echo "📋 Checking dependencies..."

# Check for Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is required but not installed."
    echo "Please install Node.js from https://nodejs.org/"
    exit 1
fi

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is required but not installed."
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check for Tauri CLI
if ! command -v tauri &> /dev/null; then
    echo "📦 Installing Tauri CLI locally..."
    npm install @tauri-apps/cli@next
fi

# Install project dependencies
echo "📦 Installing project dependencies..."
npm install

# Create icons if they don't exist
if [ ! -f "src-tauri/icons/32x32.png" ]; then
    echo "🎨 Creating application icons..."
    cd src-tauri
    chmod +x create-icons.sh
    ./create-icons.sh
    cd ..
fi

# Build the AppImage
echo "🔨 Building AppImage..."
npx tauri build --bundles appimage

# Check if build was successful
if [ -f "src-tauri/target/release/bundle/appimage/duxnet-wallet_*.AppImage" ]; then
    echo "✅ AppImage built successfully!"
    
    # List the created AppImage
    echo "📁 Created AppImage:"
    ls -la src-tauri/target/release/bundle/appimage/*.AppImage
    
    # Make it executable
    chmod +x src-tauri/target/release/bundle/appimage/*.AppImage
    
    echo ""
    echo "🎉 AppImage creation completed!"
    echo "You can find your AppImage in: src-tauri/target/release/bundle/appimage/"
    echo ""
    echo "To run the AppImage:"
    echo "./src-tauri/target/release/bundle/appimage/duxnet-wallet_*.AppImage"
else
    echo "❌ AppImage build failed!"
    exit 1
fi 