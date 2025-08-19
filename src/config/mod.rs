//! Configuration management for the Solana UI application.
//!
//! This module provides:
//! - Application configuration persistence
//! - Settings management
//! - Config file handling

use crate::utils::Cluster;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration that persists between sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Last entered validator identity for leader schedule
    pub last_leader_identity: String,
    /// Last entered epoch for leader schedule
    pub last_leader_epoch: String,
    /// Last selected cluster
    pub selected_cluster: Cluster,
    /// Last entered identity search filter
    pub last_identity_search: String,
    /// Last entered vote account search filter
    pub last_vote_account_search: String,
    /// Last entered slot for voter search
    pub last_slot_search: String,
    /// Last entered voter account search filter
    pub last_voter_account_search: String,
    /// Last entered gossip identity search filter  
    pub last_gossip_identity_search: String,
    /// Last selected tab
    pub last_selected_tab: String,
    /// Window size (width, height)
    pub window_size: Option<(f32, f32)>,
    /// Window position (x, y)
    pub window_position: Option<(f32, f32)>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            last_leader_identity: String::new(),
            last_leader_epoch: String::new(),
            selected_cluster: Cluster::Mainnet,
            last_identity_search: String::new(),
            last_vote_account_search: String::new(),
            last_slot_search: String::new(),
            last_voter_account_search: String::new(),
            last_gossip_identity_search: String::new(),
            last_selected_tab: "Validators".to_string(),
            window_size: None,
            window_position: None,
        }
    }
}

/// Configuration manager for the Solana UI application.
pub struct ConfigManager {
    config_path: PathBuf,
    config: AppConfig,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigManager {
    /// Create a new configuration manager and load existing config.
    pub fn new() -> Self {
        let config_path = Self::get_config_path();
        let config = Self::load_config(&config_path).unwrap_or_default();

        Self {
            config_path,
            config,
        }
    }

    /// Get the configuration file path.
    fn get_config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let app_config_dir = config_dir.join("solana-ui");

        // Create the config directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&app_config_dir) {
            eprintln!("Warning: Failed to create config directory: {}", e);
        }

        app_config_dir.join("config.json")
    }

    /// Load configuration from file.
    fn load_config(path: &PathBuf) -> Result<AppConfig, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to file.
    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }

    /// Get the current configuration.
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Update leader schedule settings.
    pub fn update_leader_schedule(&mut self, identity: &str, epoch: &str) {
        self.config.last_leader_identity = identity.to_string();
        self.config.last_leader_epoch = epoch.to_string();
    }

    /// Update search filters.
    pub fn update_search_filters(
        &mut self,
        identity_search: &str,
        vote_account_search: &str,
        slot_search: &str,
        voter_account_search: &str,
        gossip_identity_search: &str,
    ) {
        self.config.last_identity_search = identity_search.to_string();
        self.config.last_vote_account_search = vote_account_search.to_string();
        self.config.last_slot_search = slot_search.to_string();
        self.config.last_voter_account_search = voter_account_search.to_string();
        self.config.last_gossip_identity_search = gossip_identity_search.to_string();
    }

    /// Update selected cluster.
    pub fn update_cluster(&mut self, cluster: Cluster) {
        self.config.selected_cluster = cluster;
    }

    /// Update selected tab.
    pub fn update_selected_tab(&mut self, tab: &str) {
        self.config.last_selected_tab = tab.to_string();
    }

    /// Auto-save configuration (with error handling).
    pub fn auto_save(&self) {
        if let Err(e) = self.save_config() {
            eprintln!("Warning: Failed to save configuration: {}", e);
        }
    }
}
