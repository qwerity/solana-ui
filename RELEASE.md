# Simplified Release Process

This document describes the streamlined release process for Solana UI.

## Overview

The release system is now simplified:
- ✅ **Auto-updater**: Checks GitHub releases for newer versions
- ✅ **Master-only releases**: Automatic builds only on master branch
- ✅ **Manual versioning**: Version changes are done manually in Cargo.toml
- ✅ **Auto DMG creation**: GitHub Action creates and publishes DMG files

## Release Process

### 1. Prepare Release

1. **Update version in Cargo.toml** manually:
   ```toml
   [package]
   version = "0.2.0"  # Update this line
   ```

2. **Test your changes** locally:
   ```bash
   cargo build --release
   cargo run
   ```

3. **Commit your changes** to master:
   ```bash
   git add -A
   git commit -m "Release v0.2.0: Add new features"
   git push origin master
   ```

### 2. Automatic Build & Release

The GitHub Action will automatically:

1. **Detect version change** in Cargo.toml on master branch
2. **Build universal macOS binary** (Intel + Apple Silicon)  
3. **Create app bundle** with proper Info.plist
4. **Generate DMG installer** with professional layout
5. **Create GitHub release** with auto-generated changelog
6. **Upload DMG** as release asset

### 3. User Experience

Users can then:
- **Press Cmd+Shift+U** to check for updates
- **See update notification** in the Update tab  
- **One-click install** the new version
- **View update logs** in the Logs tab

## GitHub Action Details

### Trigger Conditions
- **Branch**: Only `master` branch
- **Path filter**: Only when `Cargo.toml` changes
- **Version check**: Only if version field actually changed

### Build Process
1. **Version detection** from Cargo.toml
2. **Multi-architecture build**:
   - x86_64-apple-darwin (Intel)  
   - aarch64-apple-darwin (Apple Silicon)
   - Universal binary combination
3. **App bundle creation** with synced Info.plist version
4. **DMG generation** with proper layout and icons
5. **Release creation** with automatic changelog

### Unsigned Builds

The GitHub Action builds **unsigned** applications:
- ✅ **No Apple Developer Account required**
- ✅ **No certificates or secrets needed**
- ✅ **Simple setup and maintenance**
- ⚠️ **Users may see "unidentified developer" warning**
- ⚠️ **Users need to bypass Gatekeeper** (right-click → Open)

#### Installing Unsigned Apps

Users will need to:
1. **Download the DMG** from GitHub releases
2. **Open the DMG** and drag app to Applications
3. **First launch**: Right-click the app → "Open" → "Open" (bypass Gatekeeper)
4. **Subsequent launches**: Normal double-click works

Alternatively, users can run:
```bash
xattr -d com.apple.quarantine /Applications/solana-ui.app
```

## Example Workflow

### Creating v0.2.0 Release:

1. **Edit Cargo.toml**:
   ```diff
   - version = "0.1.0"
   + version = "0.2.0"
   ```

2. **Commit to master**:
   ```bash
   git add Cargo.toml
   git commit -m "Release v0.2.0: Add update system and keyboard shortcuts"  
   git push origin master
   ```

3. **GitHub Action runs automatically**:
   - Builds universal DMG
   - Creates release at `https://github.com/qwerity/solana-ui/releases`
   - Users can now update via the app

## Auto-Updater Features

### For Users:
- **🔄 Update tab** shows current version and update status
- **Cmd+Shift+U** quick shortcut to check for updates
- **One-click installation** downloads and installs DMG
- **Progress tracking** with detailed logs

### For Developers:
- **GitHub API integration** checks releases automatically
- **Semantic versioning** comparison (v0.1.0 vs v0.2.0)
- **Comprehensive logging** for debugging update issues
- **Secure downloads** only from GitHub releases

## Troubleshooting

### Release Not Created
- ✅ Check that version in Cargo.toml actually changed
- ✅ Verify push was to master branch
- ✅ Review GitHub Actions logs for build errors

### Update Check Fails
- ✅ Verify repository name is correct in `src/updater.rs:14`
- ✅ Check GitHub repository is public or has proper access
- ✅ Ensure releases exist with DMG assets

### DMG Build Fails  
- ✅ Check all required dependencies are available
- ✅ Verify app bundle structure is correct
- ✅ Review create-dmg output for specific errors

## Key Benefits

✅ **Simple**: Just update Cargo.toml version and push to master  
✅ **Automatic**: No manual script running or complex workflows  
✅ **Reliable**: GitHub Actions handles all build complexity  
✅ **User-friendly**: Built-in update checking and installation  
✅ **No signing required**: No Apple Developer Account needed  
✅ **Universal**: Works on both Intel and Apple Silicon Macs  

The entire release process is now reduced to:
1. Update version in Cargo.toml
2. Push to master  
3. Done! 🚀