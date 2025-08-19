# Scripts Directory

This directory contains utility scripts for developing, building, and releasing the Solana Validators UI application.

## Available Scripts

### 🔧 `setup.sh`
Sets up the development environment by installing dependencies and tools.

```bash
./scripts/setup.sh
```

**What it does:**
- Installs Rust if not present
- Updates Rust toolchain
- Installs Homebrew (macOS)
- Installs macOS build dependencies (`create-dmg`, `librsvg`)
- Installs additional Rust tools (`cargo-watch`)
- Builds project dependencies

### 🚀 `dev.sh`
Starts the application in development mode.

```bash
./scripts/dev.sh
```

**What it does:**
- Runs `cargo run` to start the application
- Useful for development and testing

### 🧪 `test.sh`
Runs all tests and code quality checks.

```bash
./scripts/test.sh
```

**What it does:**
- Runs `cargo check` for compilation errors
- Runs `cargo test` for unit tests
- Runs `cargo clippy` for linting
- Runs `cargo fmt --check` for code formatting

### 🏗️ `build-macos.sh`
Builds a complete macOS installer (`.dmg` file).

```bash
./scripts/build-macos.sh
```

**What it does:**
- Builds release binary with `cargo build --release`
- Creates macOS app bundle (`solana-ui.app`)
- Uses pre-generated icons from `assets/icons/` (generates if missing)
- Copies pre-made Info.plist with app metadata
- Generates DMG installer with `create-dmg`

**Requirements:**
- macOS
- `create-dmg` (installed via `brew install create-dmg`)

### 🎨 `generate-icons.sh`
Generates all icon formats from the SVG logo.

```bash
./scripts/generate-icons.sh
```

**What it does:**
- Converts `solana-logo.svg` to various PNG sizes
- Creates `AppIcon.iconset` directory with all required sizes
- Generates `assets/icons/AppIcon.icns` for macOS
- Only needs to be run once or when the logo changes

**Requirements:**
- macOS
- `librsvg` (installed via `brew install librsvg`)

### 🧹 `clean.sh`
Cleans up build artifacts and temporary files.

```bash
./scripts/clean.sh
```

**What it does:**
- Runs `cargo clean`
- Removes app bundle and DMG files
- Removes icon conversion artifacts
- Cleans up temporary files (`.DS_Store`, `*.tmp`)

### 🚀 `release.sh`
Prepares a new release version.

```bash
./scripts/release.sh
```

**What it does:**
- Checks for clean git working directory
- Prompts for version number
- Updates version in `Cargo.toml`
- Runs full test suite
- Creates git commit and tag
- Provides instructions for publishing

## GitHub Actions

The project includes a GitHub Action (`.github/workflows/build-macos.yml`) that automatically builds the macOS installer when you push a git tag:

```bash
git tag v1.0.0
git push origin v1.0.0
```

This will trigger the CI/CD pipeline to:
- Build the macOS app
- Create the DMG installer
- Publish as a GitHub release

## Quick Start

1. **First time setup:**
   ```bash
   ./scripts/setup.sh
   ```

2. **Development:**
   ```bash
   ./scripts/dev.sh
   ```

3. **Testing:**
   ```bash
   ./scripts/test.sh
   ```

4. **Build installer:**
   ```bash
   ./scripts/build-macos.sh
   ```

5. **Release:**
   ```bash
   ./scripts/release.sh
   ```