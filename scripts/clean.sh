#!/bin/bash
set -e

echo "🧹 Cleaning up build artifacts..."

# Clean Cargo build artifacts
echo "🗑️  Removing Cargo build artifacts..."
cargo clean

# Clean macOS build artifacts
echo "🗑️  Removing macOS build artifacts..."
rm -rf "solana-ui.app"
rm -f "solana-ui.dmg"

# Clean temporary files
echo "🗑️  Removing temporary files..."
find . -name ".DS_Store" -delete || true
find . -name "*.tmp" -delete || true

echo "✅ Cleanup complete!"