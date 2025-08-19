#!/bin/bash
set -e

echo "🔨 Building Solana UI for macOS..."

# Build the release binary
echo "📦 Building release binary..."
cargo build --release

# Create app bundle structure
echo "🏗️ Creating app bundle..."
rm -rf "solana-ui.app"
mkdir -p "solana-ui.app/Contents/MacOS"
mkdir -p "solana-ui.app/Contents/Resources"

# Copy binary
echo "📋 Copying binary to app bundle..."
cp target/release/solana-ui "solana-ui.app/Contents/MacOS/solana-ui"
chmod +x "solana-ui.app/Contents/MacOS/solana-ui"

# Copy Info.plist
echo "📄 Copying Info.plist..."
cp Info.plist "solana-ui.app/Contents/Info.plist"

# Check if create-dmg is installed
if ! command -v create-dmg &> /dev/null; then
    echo "⚠️  create-dmg not found. Install with: brew install create-dmg"
    echo "   Skipping DMG creation..."
    exit 0
fi

# Check if icons exist, generate them if not
if [ ! -f "assets/icons/AppIcon.icns" ]; then
    echo "🎨 Icons not found, generating them..."
    ./scripts/generate-icons.sh
fi

# Copy icon to app bundle
echo "🎨 Copying icon to app bundle..."
cp assets/icons/AppIcon.icns "solana-ui.app/Contents/Resources/"

# Create DMG
echo "💿 Creating DMG installer..."
rm -f solana-ui.dmg

create-dmg \
  --volname "Solana UI" \
  --volicon "assets/icons/AppIcon.icns" \
  --window-pos 200 120 \
  --window-size 800 400 \
  --icon-size 100 \
  --icon "solana-ui.app" 200 190 \
  --hide-extension "solana-ui.app" \
  --app-drop-link 600 185 \
  --disk-image-size 200 \
  "solana-ui.dmg" \
  "solana-ui.app"

echo "✅ macOS installer created successfully: solana-ui.dmg"
echo "📦 App bundle created: solana-ui.app"
echo ""
echo "🚀 To test the app bundle, run:"
echo "   open 'solana-ui.app'"
echo ""
echo "💿 To test the DMG installer, run:"
echo "   open solana-ui.dmg"