#!/bin/bash
set -e

echo "🧪 Running tests for Solana Validators UI..."

# Run cargo check first
echo "🔍 Checking code..."
cargo check

# Run tests
echo "🧪 Running tests..."
cargo test

# Run clippy for linting
echo "📎 Running clippy..."
cargo clippy -- -D warnings

# Check formatting
echo "🎨 Checking code formatting..."
cargo fmt -- --check

echo "✅ All tests and checks passed!"