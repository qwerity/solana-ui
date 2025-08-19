#!/bin/bash
set -e

echo "🔧 Setting up development environment for Solana Validators UI..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
else
    echo "✅ Rust is already installed"
fi

# Update Rust
echo "🔄 Updating Rust toolchain..."
rustup update stable

# Check if Homebrew is installed (macOS only)
if [[ "$OSTYPE" == "darwin"* ]]; then
    if ! command -v brew &> /dev/null; then
        echo "🍺 Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    else
        echo "✅ Homebrew is already installed"
    fi
    
    # Install dependencies for building macOS installer
    echo "📦 Installing macOS build dependencies..."
    brew install create-dmg librsvg
fi

# Install additional Rust tools
echo "🔧 Installing additional Rust tools..."
cargo install cargo-watch || true

# Build dependencies
echo "📦 Building dependencies..."
cargo build

# Generate icons if on macOS
if [[ "$OSTYPE" == "darwin"* ]] && command -v rsvg-convert &> /dev/null; then
    echo "🎨 Generating application icons..."
    ./scripts/generate-icons.sh
fi

echo "✅ Development environment setup complete!"
echo ""
echo "🚀 You can now run:"
echo "   ./scripts/dev.sh     - Start development server"
echo "   ./scripts/test.sh    - Run tests and checks"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "   ./scripts/build-macos.sh - Build macOS installer"
fi