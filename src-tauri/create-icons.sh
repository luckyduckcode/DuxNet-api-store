#!/bin/bash

# Create icons directory if it doesn't exist
mkdir -p icons

# Check if ImageMagick is installed
if ! command -v convert &> /dev/null; then
    echo "ImageMagick is required to create icons. Installing..."
    sudo apt-get update
    sudo apt-get install -y imagemagick
fi

# Convert ICO to PNG files
echo "Converting icon.ico to PNG files..."

# 32x32 icon
convert icons/icon.ico -resize 32x32 icons/32x32.png

# 128x128 icon
convert icons/icon.ico -resize 128x128 icons/128x128.png

# 128x128@2x icon (256x256 for high DPI)
convert icons/icon.ico -resize 256x256 icons/128x128@2x.png

# Create ICNS file for macOS (if needed)
convert icons/icon.ico -resize 512x512 icons/icon.icns

echo "Icons created successfully!"
ls -la icons/ 