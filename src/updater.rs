//! Auto-updater module for checking and installing updates from GitHub releases.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::tabs::logs::{LogStore, log_update};

const GITHUB_API_BASE: &str = "https://api.github.com/repos";
const REPO_OWNER: &str = "qwerity"; // Replace with actual username
const REPO_NAME: &str = "solana-ui";
const USER_AGENT: &str = concat!("solana-ui/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub published_at: String,
    pub prerelease: bool,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub content_type: String,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable(ReleaseInfo),
    CheckFailed(String),
}

#[derive(Clone)]
pub struct Updater {
    current_version: String,
    client: reqwest::Client,
    log_store: LogStore,
}

impl Updater {
    pub fn new(log_store: LogStore) -> Self {
        let current_version = env!("CARGO_PKG_VERSION").to_string();
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to create HTTP client");

        log_update(&log_store, "updater_initialized", &format!("Updater initialized with version {}", current_version), "OK");

        Self {
            current_version,
            client,
            log_store,
        }
    }

    /// Check if a new version is available on GitHub releases
    pub async fn check_for_updates(&self) -> UpdateStatus {
        log_update(&self.log_store, "check_updates_started", "Checking for updates...", "Starting");
        
        match self.fetch_latest_release().await {
            Ok(release) => {
                if self.is_newer_version(&release.tag_name) {
                    log_update(&self.log_store, "update_available", &format!("Update available: {} -> {}", self.current_version, release.tag_name), "Available");
                    UpdateStatus::UpdateAvailable(release)
                } else {
                    log_update(&self.log_store, "up_to_date", &format!("Already running latest version: {}", self.current_version), "Up to date");
                    UpdateStatus::UpToDate
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to check for updates: {}", e);
                log_update(&self.log_store, "check_updates_failed", &error_msg, "Error");
                UpdateStatus::CheckFailed(error_msg)
            }
        }
    }

    /// Fetch the latest release information from GitHub API
    async fn fetch_latest_release(&self) -> Result<ReleaseInfo> {
        let url = format!("{}/{}/{}/releases/latest", GITHUB_API_BASE, REPO_OWNER, REPO_NAME);
        
        log_update(&self.log_store, "fetch_release", &format!("Fetching latest release from: {}", url), "Request");
        
        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?;

        let release: ReleaseInfo = response.json().await?;
        
        log_update(&self.log_store, "fetch_release", &format!("Found release: {} ({})", release.name, release.tag_name), "Success");
        
        Ok(release)
    }

    /// Compare version strings to determine if the remote version is newer
    fn is_newer_version(&self, remote_version: &str) -> bool {
        // Remove 'v' prefix if present
        let remote = remote_version.trim_start_matches('v');
        let current = self.current_version.trim_start_matches('v');
        
        // Simple semantic version comparison
        match (self.parse_version(current), self.parse_version(remote)) {
            (Ok(current_parts), Ok(remote_parts)) => {
                for i in 0..3 {
                    match remote_parts[i].cmp(&current_parts[i]) {
                        std::cmp::Ordering::Greater => return true,
                        std::cmp::Ordering::Less => return false,
                        std::cmp::Ordering::Equal => continue,
                    }
                }
                false
            }
            _ => false, // If parsing fails, assume no update needed
        }
    }

    /// Parse a semantic version string into [major, minor, patch]
    fn parse_version(&self, version: &str) -> Result<[u32; 3]> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Invalid version format: {}", version));
        }

        Ok([
            parts[0].parse()?,
            parts[1].parse()?,
            parts[2].parse()?,
        ])
    }


    /// Download the DMG file for the given release to Downloads folder
    pub async fn download_update(&self, release: &ReleaseInfo) -> Result<PathBuf> {
        log_update(&self.log_store, "download_started", &format!("Starting download of {}", release.tag_name), "Starting");
        
        // Find the macOS asset (DMG file)
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name.ends_with(".dmg"))
            .ok_or_else(|| anyhow!("No macOS installer found in release"))?;

        log_update(&self.log_store, "dmg_found", &format!("Found DMG: {} ({} bytes)", asset.name, asset.size), "Found");

        // Get Downloads directory
        let downloads_dir = dirs::download_dir()
            .ok_or_else(|| anyhow!("Could not find Downloads directory"))?;
        
        let dmg_path = downloads_dir.join(&asset.name);
        
        // Check if file already exists
        if dmg_path.exists() {
            log_update(&self.log_store, "file_exists", &format!("DMG already exists: {}", dmg_path.display()), "Exists");
            return Ok(dmg_path);
        }

        log_update(&self.log_store, "download_start", &format!("Downloading to: {}", dmg_path.display()), "Downloading");

        // Download the file
        let response = self
            .client
            .get(&asset.browser_download_url)
            .send()
            .await?
            .error_for_status()?;

        let mut file = fs::File::create(&dmg_path).await?;
        
        // Get the response bytes directly
        let bytes = response.bytes().await?;
        file.write_all(&bytes).await?;
        let downloaded = bytes.len() as u64;
        
        log_update(&self.log_store, "download_progress", &format!("Downloaded {} bytes", downloaded), "Progress");

        file.flush().await?;
        log_update(&self.log_store, "download_complete", &format!("Download complete: {}", dmg_path.display()), "Complete");

        Ok(dmg_path)
    }

    /// Get the current version
    pub fn current_version(&self) -> &str {
        &self.current_version
    }
}
