#!/bin/bash
set -e

echo "🎨 Generating icons for Solana UI..."

# Check if rsvg-convert is available
if ! command -v rsvg-convert &> /dev/null; then
    echo "⚠️  rsvg-convert not found. Install with: brew install librsvg"
    exit 1
fi

# Check if iconutil is available (should be on macOS)
if ! command -v iconutil &> /dev/null; then
    echo "⚠️  iconutil not found. This script requires macOS."
    exit 1
fi

# Create assets directory if it doesn't exist
mkdir -p assets/icons

# Create iconset directory
rm -rf assets/icons/AppIcon.iconset
mkdir assets/icons/AppIcon.iconset

echo "🔄 Converting SVG to different PNG sizes..."

# Convert SVG to different sizes for iconset
rsvg-convert -w 16 -h 16 solana-logo.svg -o assets/icons/AppIcon.iconset/icon_16x16.png
rsvg-convert -w 32 -h 32 solana-logo.svg -o assets/icons/AppIcon.iconset/icon_16x16@2x.png
rsvg-convert -w 32 -h 32 solana-logo.svg -o assets/icons/AppIcon.iconset/icon_32x32.png
rsvg-convert -w 64 -h 64 solana-logo.svg -o assets/icons/AppIcon.iconset/icon_32x32@2x.png
rsvg-convert -w 128 -h 128 solana-logo.svg -o assets/icons/AppIcon.iconset/icon_128x128.png
rsvg-convert -w 256 -h 256 solana-logo.svg -o assets/icons/AppIcon.iconset/icon_128x128@2x.png
rsvg-convert -w 256 -h 256 solana-logo.svg -o assets/icons/AppIcon.iconset/icon_256x256.png
rsvg-convert -w 512 -h 512 solana-logo.svg -o assets/icons/AppIcon.iconset/icon_256x256@2x.png
rsvg-convert -w 512 -h 512 solana-logo.svg -o assets/icons/AppIcon.iconset/icon_512x512.png
rsvg-convert -w 1024 -h 1024 solana-logo.svg -o assets/icons/AppIcon.iconset/icon_512x512@2x.png

echo "🏗️ Creating ICNS file..."

# Convert to ICNS
iconutil -c icns assets/icons/AppIcon.iconset -o assets/icons/AppIcon.icns

echo "✅ Icons generated successfully!"
echo "📁 Files created:"
echo "   assets/icons/AppIcon.iconset/ - Individual PNG files"
echo "   assets/icons/AppIcon.icns - macOS icon file"
