//! Utility functions, enums, and status management for the Solana UI application.
//!
//! This module provides:
//! - Sorting functionality with multi-column support
//! - Network cluster definitions and URLs  
//! - Status management for async operations
//! - Formatting utilities for display

use eframe::egui;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::constants::*;

/// Direction for sorting table columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Solana network clusters supported by the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cluster {
    Testnet,
    Mainnet,
}

impl Cluster {
    /// Get the RPC URL for this cluster.
    pub const fn url(self) -> &'static str {
        match self {
            Self::Testnet => "https://api.testnet.solana.com",
            Self::Mainnet => "https://api.mainnet-beta.solana.com",
        }
    }

    /// Get the display name for this cluster.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Testnet => "Testnet",
            Self::Mainnet => "Mainnet",
        }
    }

    /// Get all available clusters.
    pub const fn all() -> &'static [Self] {
        &[Self::Testnet, Self::Mainnet]
    }
}

/// State for a single column's sorting configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortState {
    pub column: SortColumn,
    pub direction: SortDirection,
    pub priority: usize,
}

impl SortState {
    /// Create a new sort state for a column.
    pub const fn new(column: SortColumn, direction: SortDirection, priority: usize) -> Self {
        Self {
            column,
            direction,
            priority,
        }
    }
}

/// Available columns for sorting validator data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Identity,
    VoteAccount,
    Commission,
    LastVote,
    RootSlot,
    VoteCredits,
    ActivatedStake,
    Version,
    SkipRate,
}

/// Manages loading states and status messages for async operations.
pub struct StatusManager {
    pub refresh_status: String,
    pub validators_loading: bool,
    pub slot_loading: bool,
    pub last_validators_fetch: Option<Instant>,
    pub last_slot_fetch: Option<Instant>,
}

/// Timeouts for different operations in seconds.
mod timeouts {
    pub const VALIDATORS_TIMEOUT: u64 = 5;
    pub const SLOT_TIMEOUT: u64 = 3;
    pub const STATUS_DISPLAY: u64 = 7;
}

/// Status messages used throughout the application.
mod status_messages {
    pub const READY: &str = "Ready";
    pub const FETCHING_VALIDATORS: &str = "Fetching validators...";
    pub const UPDATING_SLOT: &str = "Updating slot info...";
    pub const VALIDATORS_UPDATED: &str = "Validators updated";
}

impl Default for StatusManager {
    fn default() -> Self {
        Self {
            refresh_status: status_messages::READY.to_string(),
            validators_loading: false,
            slot_loading: false,
            last_validators_fetch: None,
            last_slot_fetch: None,
        }
    }
}

impl StatusManager {
    /// Start tracking a validators refresh operation.
    pub fn start_validators_refresh(&mut self) {
        self.validators_loading = true;
        self.refresh_status = status_messages::FETCHING_VALIDATORS.to_string();
        self.last_validators_fetch = Some(Instant::now());
    }

    /// Start tracking a slot info refresh operation.
    pub fn start_slot_refresh(&mut self) {
        self.slot_loading = true;
        if !self.validators_loading {
            self.refresh_status = status_messages::UPDATING_SLOT.to_string();
        }
        self.last_slot_fetch = Some(Instant::now());
    }

    /// Update loading states based on elapsed time.
    /// Should be called regularly from the UI update loop.
    pub fn update(&mut self) {
        let now = Instant::now();

        self.check_validators_timeout(now);
        self.check_slot_timeout(now);
        self.auto_reset_status(now);
    }

    /// Check if validators fetch operation has timed out.
    fn check_validators_timeout(&mut self, now: Instant) {
        if self.validators_loading {
            if let Some(start_time) = self.last_validators_fetch {
                if now.duration_since(start_time)
                    > Duration::from_secs(timeouts::VALIDATORS_TIMEOUT)
                {
                    self.validators_loading = false;
                    self.refresh_status = status_messages::VALIDATORS_UPDATED.to_string();
                }
            }
        }
    }

    /// Check if slot fetch operation has timed out.
    fn check_slot_timeout(&mut self, now: Instant) {
        if self.slot_loading {
            if let Some(start_time) = self.last_slot_fetch {
                if now.duration_since(start_time) > Duration::from_secs(timeouts::SLOT_TIMEOUT) {
                    self.slot_loading = false;
                    if !self.validators_loading
                        && self.refresh_status == status_messages::UPDATING_SLOT
                    {
                        self.refresh_status = status_messages::READY.to_string();
                    }
                }
            }
        }
    }

    /// Auto-reset status to Ready after showing completion message for a while.
    fn auto_reset_status(&mut self, now: Instant) {
        if !self.is_loading() && self.refresh_status == status_messages::VALIDATORS_UPDATED {
            if let Some(start_time) = self.last_validators_fetch {
                if now.duration_since(start_time) > Duration::from_secs(timeouts::STATUS_DISPLAY) {
                    self.refresh_status = status_messages::READY.to_string();
                }
            }
        }
    }

    /// Check if any operations are currently loading.
    pub const fn is_loading(&self) -> bool {
        self.validators_loading || self.slot_loading
    }
}

/// Formatting constants for display values.
mod formatting {
    pub const LAMPORTS_PER_SOL: f64 = 1_000_000_000.0;
    pub const DECIMAL_PLACES: usize = 2;
}

/// Format stake amount from lamports to SOL with appropriate precision.
pub fn format_stake(stake: u64) -> String {
    let sol_amount = stake as f64 / formatting::LAMPORTS_PER_SOL;
    format!(
        "{:.precision$} SOL",
        sol_amount,
        precision = formatting::DECIMAL_PLACES
    )
}

/// Format skip rate as a percentage with appropriate precision.
pub fn format_skip_rate(skip_rate: f64) -> String {
    format!(
        "{:.precision$}%",
        skip_rate,
        precision = formatting::DECIMAL_PLACES
    )
}

/// Create a standard error frame with consistent styling.
pub fn create_error_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(ERROR_BACKGROUND)
        .inner_margin(FRAME_INNER_MARGIN)
        .corner_radius(FRAME_CORNER_RADIUS)
        .stroke(egui::Stroke::new(FRAME_STROKE_WIDTH, egui::Color32::RED))
}

/// Create a standard info frame with consistent styling.
pub fn create_info_frame(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::new()
        .fill(ui.visuals().widgets.noninteractive.weak_bg_fill)
        .inner_margin(FRAME_INNER_MARGIN_SMALL)
        .corner_radius(FRAME_CORNER_RADIUS)
}

/// Create a colored cell frame for table rows.
pub fn create_cell_frame(color: egui::Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(color)
        .inner_margin(FRAME_INNER_MARGIN_TINY)
}

/// Render a standard search field with consistent sizing.
pub fn render_search_field(
    ui: &mut egui::Ui,
    search_term: &mut String,
    hint_text: &str,
    should_focus: bool,
    width: f32,
) -> egui::Response {
    let response = ui.add_sized(
        [width, SEARCH_FIELD_HEIGHT],
        egui::TextEdit::singleline(search_term).hint_text(hint_text),
    );

    if should_focus {
        response.request_focus();
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_stake() {
        assert_eq!(format_stake(1_000_000_000), "1.00 SOL");
        assert_eq!(format_stake(500_000_000), "0.50 SOL");
        assert_eq!(format_stake(0), "0.00 SOL");
    }

    #[test]
    fn test_format_skip_rate() {
        assert_eq!(format_skip_rate(5.25), "5.25%");
        assert_eq!(format_skip_rate(0.0), "0.00%");
        assert_eq!(format_skip_rate(100.0), "100.00%");
    }

    #[test]
    fn test_cluster_urls() {
        assert_eq!(Cluster::Testnet.url(), "https://api.testnet.solana.com");
        assert_eq!(
            Cluster::Mainnet.url(),
            "https://api.mainnet-beta.solana.com"
        );
    }
}
