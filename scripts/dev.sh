#!/bin/bash
set -e

echo "🚀 Starting Solana Validators UI in development mode..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo is not installed. Please install from https://rustup.rs/"
    exit 1
fi

# Run in development mode with auto-reload
echo "📦 Building and running in development mode..."
echo "💡 The app will automatically rebuild when you make changes to the code."
echo "🛑 Press Ctrl+C to stop."
echo ""

cargo run