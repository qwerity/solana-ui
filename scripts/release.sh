#!/bin/bash
set -e

echo "🚀 Preparing release for Solana Validators UI..."

# Check if we're on main/master branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [[ "$CURRENT_BRANCH" != "main" && "$CURRENT_BRANCH" != "master" ]]; then
    echo "⚠️  Warning: You're not on the main/master branch (currently on: $CURRENT_BRANCH)"
    read -p "Do you want to continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check if working directory is clean
if [[ -n $(git status --porcelain) ]]; then
    echo "❌ Working directory is not clean. Please commit or stash your changes."
    git status --short
    exit 1
fi

# Get version from user
read -p "Enter version number (e.g., 1.0.0): " VERSION
if [[ -z "$VERSION" ]]; then
    echo "❌ Version number is required"
    exit 1
fi

# Validate version format
if [[ ! $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "❌ Invalid version format. Use semantic versioning (e.g., 1.0.0)"
    exit 1
fi

# Update version in Cargo.toml
echo "📝 Updating version in Cargo.toml..."
sed -i '' "s/^version = .*/version = \"$VERSION\"/" Cargo.toml

# Run tests
echo "🧪 Running tests..."
./scripts/test.sh

# Build release
echo "🔨 Building release..."
cargo build --release

# Create git tag
echo "🏷️  Creating git tag..."
git add Cargo.toml Cargo.lock
git commit -m "Release v$VERSION" || true
git tag "v$VERSION"

echo "✅ Release v$VERSION prepared!"
echo ""
echo "🚀 Next steps:"
echo "   git push origin main"
echo "   git push origin v$VERSION"
echo ""
echo "This will trigger the GitHub Action to build and release the macOS installer."