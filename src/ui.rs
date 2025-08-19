//! Main UI components and application logic for the Solana Validators UI.
//!
//! This module provides the main ValidatorApp struct and orchestrates all tabs.

use std::sync::Arc;
use std::time::Instant;

use eframe::egui;
use tokio::sync::Mutex;

use crate::config::ConfigManager;
use crate::solana::{
    GossipNodeInfo, LeaderScheduleInfo, SlotVoterInfo, SolanaClient, ValidatorInfo,
};
use crate::tabs::{
    find_voters::{self, FindVotersTabParams},
    gossip_nodes,
    leader_schedule::{self, LeaderScheduleTabParams},
    logs,
    update::UpdateTab,
    validators::{self, ValidatorsTabParams},
    AppTab,
};
use crate::utils::{Cluster, SortColumn, SortDirection, SortState, StatusManager};

/// Type alias for slot information: (current_slot, latest_slot, current_epoch)
type SlotInfo = (Option<u64>, Option<u64>, Option<u64>);

/// Constants for UI layout and behavior
mod ui_constants {
    pub const MAX_SORT_COLUMNS: usize = 3;
    pub const UI_UPDATE_INTERVAL_SECS: u64 = 1;
}

/// Main application struct managing all UI state and data.
pub struct ValidatorApp {
    // Data stores
    validators: Arc<Mutex<Vec<ValidatorInfo>>>,
    gossip_nodes: Arc<Mutex<Vec<GossipNodeInfo>>>,
    slot_voter_result: Arc<Mutex<Option<SlotVoterInfo>>>,
    leader_schedule_result: Arc<Mutex<Option<LeaderScheduleInfo>>>,
    slot_info: Arc<Mutex<SlotInfo>>,
    log_store: logs::LogStore,

    // UI state
    current_tab: AppTab,
    sort_states: Vec<SortState>,
    error_message: Option<String>,

    // Search fields
    identity_search: String,
    vote_account_search: String,
    slot_search: String,
    voter_account_search: String,
    leader_identity_search: String,
    leader_epoch_search: String,
    gossip_identity_search: String,

    // Per-tab search terms
    validators_search: String,
    gossip_nodes_search: String,
    find_voters_search: String,
    logs_search: String,

    // Search focus state
    should_focus_search: bool,

    // Tabs
    update_tab: UpdateTab,

    // Backend services
    rt: Option<tokio::runtime::Runtime>,
    status_manager: StatusManager,
    solana_client: SolanaClient,
    selected_cluster: Cluster,
    config_manager: ConfigManager,
    last_config_save: Instant,
}

impl Default for ValidatorApp {
    fn default() -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let config_manager = ConfigManager::new();
        let config = config_manager.config();

        let log_store = logs::create_log_store();

        // Add sample log entries to demonstrate functionality
        logs::log_request(
            &log_store,
            "app_startup",
            "system",
            &format!(
                "Application started with {} cluster ({})",
                config.selected_cluster.name(),
                config.selected_cluster.url()
            ),
        );
        logs::log_request(
            &log_store,
            "get_cluster_nodes",
            config.selected_cluster.url(),
            &format!("endpoint: {}", config.selected_cluster.url()),
        );
        logs::log_response(
            &log_store,
            "get_cluster_nodes",
            config.selected_cluster.url(),
            "200 nodes found",
            "200 OK",
        );
        logs::log_request(
            &log_store,
            "get_vote_accounts",
            config.selected_cluster.url(),
            &format!("endpoint: {}", config.selected_cluster.url()),
        );

        Self {
            validators: Arc::new(Mutex::new(Vec::new())),
            gossip_nodes: Arc::new(Mutex::new(Vec::new())),
            slot_voter_result: Arc::new(Mutex::new(None)),
            leader_schedule_result: Arc::new(Mutex::new(None)),
            slot_info: Arc::new(Mutex::new((None, None, None))),
            log_store: log_store.clone(),
            error_message: None,
            rt: Some(rt),
            sort_states: Vec::new(),
            identity_search: config.last_identity_search.clone(),
            vote_account_search: config.last_vote_account_search.clone(),
            slot_search: config.last_slot_search.clone(),
            voter_account_search: config.last_voter_account_search.clone(),
            leader_identity_search: config.last_leader_identity.clone(),
            leader_epoch_search: config.last_leader_epoch.clone(),
            gossip_identity_search: config.last_gossip_identity_search.clone(),
            validators_search: String::new(),
            gossip_nodes_search: String::new(),
            find_voters_search: String::new(),
            logs_search: String::new(),
            should_focus_search: false,
            update_tab: UpdateTab::new(log_store.clone()),
            status_manager: StatusManager::default(),
            solana_client: SolanaClient::new(
                config.selected_cluster.url().to_string(),
                log_store.clone(),
            ),
            selected_cluster: config.selected_cluster,
            current_tab: AppTab::from_id(&config.last_selected_tab),
            config_manager,
            last_config_save: Instant::now(),
        }
    }
}

impl ValidatorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    // Data fetching methods
    pub fn refresh_validators(&mut self) {
        if self.status_manager.validators_loading {
            return;
        }

        self.status_manager.start_validators_refresh();
        self.error_message = None;

        let validators_clone = Arc::clone(&self.validators);
        let client = self.solana_client.clone();

        if let Some(rt) = &self.rt {
            rt.spawn(async move {
                match client.fetch_validators().await {
                    Ok(new_validators) => {
                        let mut validators = validators_clone.lock().await;
                        *validators = new_validators;
                    }
                    Err(e) => {
                        eprintln!("Error fetching validators: {}", e);
                    }
                }
            });
        }
    }

    pub fn refresh_gossip_nodes(&mut self) {
        if self.status_manager.validators_loading {
            return;
        }

        self.status_manager.start_validators_refresh();
        self.error_message = None;

        let gossip_nodes_clone = Arc::clone(&self.gossip_nodes);
        let client = self.solana_client.clone();

        if let Some(rt) = &self.rt {
            rt.spawn(async move {
                match client.fetch_cluster_nodes().await {
                    Ok(new_nodes) => {
                        let mut gossip_nodes = gossip_nodes_clone.lock().await;
                        *gossip_nodes = new_nodes;
                    }
                    Err(e) => {
                        eprintln!("Error fetching gossip nodes: {}", e);
                    }
                }
            });
        }
    }

    pub fn search_voters_in_slot(&mut self, slot: u64) {
        if self.status_manager.validators_loading {
            return;
        }

        self.status_manager.start_validators_refresh();
        self.error_message = None;

        let slot_voter_result_clone = Arc::clone(&self.slot_voter_result);
        let client = self.solana_client.clone();

        if let Some(rt) = &self.rt {
            rt.spawn(async move {
                match client.find_voters_in_slot(slot).await {
                    Ok(voter_info) => {
                        let mut result = slot_voter_result_clone.lock().await;
                        *result = Some(voter_info);
                    }
                    Err(e) => {
                        eprintln!("Error finding voters in slot {}: {}", slot, e);
                    }
                }
            });
        }
    }

    pub fn fetch_leader_schedule(&mut self, identity: &str, epoch: Option<u64>) {
        if self.status_manager.validators_loading {
            return;
        }

        self.status_manager.start_validators_refresh();
        self.error_message = None;

        let leader_schedule_result_clone = Arc::clone(&self.leader_schedule_result);
        let client = self.solana_client.clone();
        let identity_clone = identity.to_string();

        if let Some(rt) = &self.rt {
            rt.spawn(async move {
                match client.fetch_leader_schedule(&identity_clone, epoch).await {
                    Ok(leader_info) => {
                        let mut result = leader_schedule_result_clone.lock().await;
                        *result = Some(leader_info);
                    }
                    Err(e) => {
                        eprintln!(
                            "Error fetching leader schedule for {}: {}",
                            identity_clone, e
                        );
                    }
                }
            });
        }
    }

    pub fn refresh_slot_info(&mut self) {
        if self.status_manager.slot_loading {
            return;
        }

        self.status_manager.start_slot_refresh();

        let slot_info_clone = Arc::clone(&self.slot_info);
        let client = self.solana_client.clone();

        if let Some(rt) = &self.rt {
            rt.spawn(async move {
                match client.fetch_slot_info().await {
                    Ok((current_slot, latest_slot, current_epoch)) => {
                        let mut slot_info = slot_info_clone.lock().await;
                        *slot_info = (Some(current_slot), Some(latest_slot), Some(current_epoch));
                    }
                    Err(e) => {
                        eprintln!("Error fetching slot info: {}", e);
                    }
                }
            });
        }
    }

    // Configuration methods
    pub fn change_cluster(&mut self, new_cluster: Cluster) {
        // Always log cluster change attempts for debugging
        logs::log_request(
            &self.log_store,
            "cluster_change_attempt",
            "system",
            &format!(
                "Attempt to change from {} to {} (current != new: {})",
                self.selected_cluster.name(),
                new_cluster.name(),
                self.selected_cluster != new_cluster
            ),
        );

        if self.selected_cluster != new_cluster {
            // Log the cluster change
            logs::log_request(
                &self.log_store,
                "cluster_change",
                "system",
                &format!(
                    "Changing from {} to {} ({})",
                    self.selected_cluster.name(),
                    new_cluster.name(),
                    new_cluster.url()
                ),
            );

            self.selected_cluster = new_cluster;
            self.solana_client =
                SolanaClient::new(new_cluster.url().to_string(), self.log_store.clone());

            // Save cluster change to config
            self.config_manager.update_cluster(new_cluster);
            self.config_manager.auto_save();

            // Refresh data for the new cluster
            self.refresh_validators();
            self.refresh_gossip_nodes();
            self.refresh_slot_info();

            logs::log_response(
                &self.log_store,
                "cluster_change",
                "system",
                &format!("Successfully switched to {} cluster", new_cluster.name()),
                "200 OK",
            );
        } else {
            logs::log_response(
                &self.log_store,
                "cluster_change_attempt",
                "system",
                &format!(
                    "No change needed - already using {} cluster",
                    new_cluster.name()
                ),
                "No Change",
            );
        }
    }

    fn save_current_state(&mut self) {
        self.config_manager
            .update_leader_schedule(&self.leader_identity_search, &self.leader_epoch_search);
        self.config_manager.update_search_filters(
            &self.identity_search,
            &self.vote_account_search,
            &self.slot_search,
            &self.voter_account_search,
            &self.gossip_identity_search,
        );
        self.config_manager
            .update_selected_tab(self.current_tab.id());
        self.config_manager.auto_save();
    }

    // Sorting methods
    pub fn handle_column_sort(&mut self, column: SortColumn, shift_pressed: bool) {
        if let Some(existing_index) = self.sort_states.iter().position(|s| s.column == column) {
            // Column is already being sorted
            let mut existing_sort = self.sort_states.remove(existing_index);
            existing_sort.direction = match existing_sort.direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };

            if shift_pressed {
                // Multi-column sort: keep existing position
                self.sort_states.insert(existing_index, existing_sort);
            } else {
                // Single column sort: make this primary
                existing_sort.priority = 0;
                self.sort_states.insert(0, existing_sort);
            }
        } else {
            // New column to sort
            let new_sort = SortState::new(column, SortDirection::Ascending, 0);
            if shift_pressed && !self.sort_states.is_empty() {
                // Add as secondary sort
                self.sort_states.push(new_sort);
            } else {
                // Replace all sorts with this new one
                self.sort_states.clear();
                self.sort_states.push(new_sort);
            }
        }

        // Update priorities
        for (i, sort_state) in self.sort_states.iter_mut().enumerate() {
            sort_state.priority = i;
        }

        // Limit the number of sort columns for performance and UI clarity
        if self.sort_states.len() > ui_constants::MAX_SORT_COLUMNS {
            self.sort_states.truncate(ui_constants::MAX_SORT_COLUMNS);
        }
    }

    // UI rendering methods
    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_space(16.0); // Left padding
                                // Left side: Slot and epoch information
            let slot_info = if let Ok(guard) = self.slot_info.try_lock() {
                *guard
            } else {
                (None, None, None)
            };

            if let (Some(current_slot), Some(latest_slot), Some(current_epoch)) = slot_info {
                ui.label(format!(
                    "🔗 Epoch: {} | Current Slot: {} | Latest Slot: {}",
                    current_epoch, current_slot, latest_slot
                ));
            } else {
                ui.label("🔗 Network info: Loading...");
            }

            // Spacer to push right content to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(16.0); // Right padding
                                    // Right side: Status and loading indicator
                ui.colored_label(
                    if self.status_manager.is_loading() {
                        egui::Color32::from_rgb(204, 102, 0) // Dark orange for loading
                    } else if self.status_manager.refresh_status == "Ready" {
                        egui::Color32::from_rgb(0, 128, 0) // Dark green for ready
                    } else {
                        egui::Color32::from_rgb(0, 102, 204) // Dark blue for status updates
                    },
                    format!("⚡ {}", self.status_manager.refresh_status),
                );

                if self.status_manager.is_loading() {
                    ui.add_space(8.0);
                    ui.spinner();
                }
            });
        });
    }

    fn handle_tab_refresh(&mut self) {
        match self.current_tab {
            AppTab::Validators => self.refresh_validators(),
            AppTab::GossipNodes => self.refresh_gossip_nodes(),
            AppTab::FindVoters => {
                if let Ok(slot) = self.slot_search.parse::<u64>() {
                    self.search_voters_in_slot(slot);
                }
            }
            AppTab::LeaderSchedule => {
                if !self.leader_identity_search.is_empty() {
                    let identity = self.leader_identity_search.clone();
                    let epoch = if self.leader_epoch_search.is_empty() {
                        None
                    } else {
                        self.leader_epoch_search.parse::<u64>().ok()
                    };
                    self.fetch_leader_schedule(&identity, epoch);
                }
            }
            AppTab::Logs => {
                // No refresh action needed for logs tab
            }
            AppTab::Update => {
                // No refresh action needed for update tab
            }
        }
        self.refresh_slot_info();
    }

    fn trigger_update_check(&mut self) {
        // Switch to update tab and log the action
        self.current_tab = AppTab::Update;
        
        // Log the update check trigger
        crate::tabs::logs::log_update(
            &self.log_store,
            "check_updates_triggered",
            "Update check triggered via keyboard shortcut (Cmd+Shift+U)",
            "Triggered"
        );
        
        // Save config due to tab change
        self.config_manager.update_selected_tab(self.current_tab.id());
        self.config_manager.auto_save();
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        // Quit shortcut (Cmd+Q)
        if ctx.input(|i| i.key_pressed(egui::Key::Q) && (i.modifiers.mac_cmd || i.modifiers.ctrl)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::R) && (i.modifiers.mac_cmd || i.modifiers.ctrl)) {
            self.handle_tab_refresh();
        }

        // Search focus shortcut (Cmd+F)
        if ctx.input(|i| i.key_pressed(egui::Key::F) && (i.modifiers.mac_cmd || i.modifiers.ctrl)) {
            self.should_focus_search = true;
        }

        // Update check shortcut (Cmd+Shift+U)
        if ctx.input(|i| i.key_pressed(egui::Key::U) && (i.modifiers.mac_cmd || i.modifiers.ctrl) && i.modifiers.shift) {
            self.trigger_update_check();
        }

        // Tab switching shortcuts (Cmd+1, Cmd+2, etc.)
        let previous_tab = self.current_tab;
        ctx.input(|i| {
            if i.modifiers.mac_cmd || i.modifiers.ctrl {
                if i.key_pressed(egui::Key::Num1) {
                    self.current_tab = AppTab::Validators;
                } else if i.key_pressed(egui::Key::Num2) {
                    self.current_tab = AppTab::GossipNodes;
                } else if i.key_pressed(egui::Key::Num3) {
                    self.current_tab = AppTab::FindVoters;
                } else if i.key_pressed(egui::Key::Num4) {
                    self.current_tab = AppTab::LeaderSchedule;
                } else if i.key_pressed(egui::Key::Num5) {
                    self.current_tab = AppTab::Logs;
                } else if i.key_pressed(egui::Key::Num6) {
                    self.current_tab = AppTab::Update;
                }
            }
        });

        // Save config if tab changed via keyboard shortcut
        if previous_tab != self.current_tab {
            self.config_manager
                .update_selected_tab(self.current_tab.id());
            self.config_manager.auto_save();
        }
    }

    fn save_config_changes(&mut self) {
        self.config_manager.update_search_filters(
            &self.identity_search,
            &self.vote_account_search,
            &self.slot_search,
            &self.voter_account_search,
            &self.gossip_identity_search,
        );
        self.config_manager
            .update_leader_schedule(&self.leader_identity_search, &self.leader_epoch_search);
        self.config_manager.auto_save();
    }

    pub fn clear_logs(&mut self) {
        if let Ok(mut logs) = self.log_store.lock() {
            logs.clear();
        }
    }
}

impl eframe::App for ValidatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(ctx);

        // Update refresh status based on elapsed time
        self.status_manager.update();

        // Top panel for cluster selection and tab selection
        egui::TopBottomPanel::top("top_panel")
            .exact_height(45.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0); // More generous left spacing

                    // Tab selection on the left
                    let previous_tab = self.current_tab;
                    ui.selectable_value(
                        &mut self.current_tab,
                        AppTab::Validators,
                        AppTab::Validators.name(),
                    )
                    .on_hover_text("Switch to Validators tab (Cmd+1)");
                    ui.add_space(8.0);
                    ui.selectable_value(
                        &mut self.current_tab,
                        AppTab::GossipNodes,
                        AppTab::GossipNodes.name(),
                    )
                    .on_hover_text("Switch to Gossip Nodes tab (Cmd+2)");
                    ui.add_space(8.0);
                    ui.selectable_value(
                        &mut self.current_tab,
                        AppTab::FindVoters,
                        AppTab::FindVoters.name(),
                    )
                    .on_hover_text("Switch to Find Voters tab (Cmd+3)");
                    ui.add_space(8.0);
                    ui.selectable_value(
                        &mut self.current_tab,
                        AppTab::LeaderSchedule,
                        AppTab::LeaderSchedule.name(),
                    )
                    .on_hover_text("Switch to Leader Schedule tab (Cmd+4)");
                    ui.add_space(8.0);
                    ui.selectable_value(&mut self.current_tab, AppTab::Logs, AppTab::Logs.name())
                        .on_hover_text("Switch to Logs tab (Cmd+5)");
                    ui.add_space(8.0);
                    ui.selectable_value(&mut self.current_tab, AppTab::Update, AppTab::Update.name())
                        .on_hover_text("Switch to Update tab (Cmd+6 or Cmd+Shift+U)");

                    // Save config if tab changed
                    if previous_tab != self.current_tab {
                        self.config_manager
                            .update_selected_tab(self.current_tab.id());
                        self.config_manager.auto_save();
                    }

                    // Push controls to the right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Cluster selection dropdown in top right
                        let mut selected_cluster = self.selected_cluster;
                        egui::ComboBox::from_label("Cluster")
                            .selected_text(self.selected_cluster.name())
                            .show_ui(ui, |ui| {
                                for &cluster in Cluster::all() {
                                    if ui
                                        .selectable_value(
                                            &mut selected_cluster,
                                            cluster,
                                            cluster.name(),
                                        )
                                        .changed()
                                    {
                                        self.change_cluster(cluster);
                                    }
                                }
                            });
                    });
                });
            });

        // Bottom status bar panel
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(50.0)
            .show(ctx, |ui| {
                self.render_status_bar(ui);
            });

        // Main content panel
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(16.0))
            .show(ctx, |ui| {
                ui.add_space(8.0);

                // Render current tab
                match self.current_tab {
                    AppTab::Validators => {
                        let all_validators = if let Ok(guard) = self.validators.try_lock() {
                            guard.clone()
                        } else {
                            Vec::new()
                        };

                        let mut sort_request: Option<(SortColumn, bool)> = None;
                        let mut refresh_requested = false;

                        let should_focus = self.should_focus_search;
                        validators::render_validators_tab(
                            ui,
                            ValidatorsTabParams {
                                validators: &all_validators,
                                sort_states: &self.sort_states,
                                search_term: &mut self.validators_search,
                                error_message: &self.error_message,
                                is_loading: self.status_manager.is_loading(),
                                should_focus_search: should_focus,
                            },
                            |column, shift| {
                                sort_request = Some((column, shift));
                            },
                            || {
                                refresh_requested = true;
                            },
                        );

                        if let Some((column, shift)) = sort_request {
                            self.handle_column_sort(column, shift);
                        }
                        if refresh_requested {
                            self.refresh_validators();
                        }
                    }
                    AppTab::GossipNodes => {
                        let all_gossip_nodes = if let Ok(guard) = self.gossip_nodes.try_lock() {
                            guard.clone()
                        } else {
                            Vec::new()
                        };

                        let mut refresh_requested = false;

                        let should_focus = self.should_focus_search;
                        gossip_nodes::render_gossip_nodes_tab(
                            ui,
                            &all_gossip_nodes,
                            &mut self.gossip_nodes_search,
                            &self.error_message,
                            self.status_manager.is_loading(),
                            should_focus,
                            || {
                                refresh_requested = true;
                            },
                        );

                        if refresh_requested {
                            self.refresh_gossip_nodes();
                        }
                    }
                    AppTab::FindVoters => {
                        let voter_result = if let Ok(guard) = self.slot_voter_result.try_lock() {
                            guard.clone()
                        } else {
                            None
                        };

                        let mut search_slot: Option<u64> = None;
                        let mut clear_needed = false;
                        let mut save_needed = false;

                        let should_focus = self.should_focus_search;
                        find_voters::render_find_voters_tab(
                            ui,
                            FindVotersTabParams {
                                slot_search: &mut self.slot_search,
                                voter_result: &voter_result,
                                search_term: &mut self.find_voters_search,
                                error_message: &self.error_message,
                                is_loading: self.status_manager.is_loading(),
                                should_focus_search: should_focus,
                            },
                            |slot| {
                                search_slot = Some(slot);
                            },
                            || {
                                clear_needed = true;
                            },
                            || {
                                save_needed = true;
                            },
                        );

                        if let Some(slot) = search_slot {
                            self.search_voters_in_slot(slot);
                        }
                        if clear_needed {
                            self.slot_search.clear();
                            self.voter_account_search.clear();
                            if let Ok(mut result) = self.slot_voter_result.try_lock() {
                                *result = None;
                            }
                            self.save_config_changes();
                        }
                        if save_needed {
                            self.save_config_changes();
                        }
                    }
                    AppTab::LeaderSchedule => {
                        let leader_result =
                            if let Ok(guard) = self.leader_schedule_result.try_lock() {
                                guard.clone()
                            } else {
                                None
                            };

                        let mut fetch_request: Option<(String, Option<u64>)> = None;
                        let mut clear_needed = false;
                        let mut save_needed = false;

                        leader_schedule::render_leader_schedule_tab(
                            ui,
                            LeaderScheduleTabParams {
                                leader_identity_search: &mut self.leader_identity_search,
                                leader_epoch_search: &mut self.leader_epoch_search,
                                leader_result: &leader_result,
                                error_message: &self.error_message,
                                is_loading: self.status_manager.is_loading(),
                            },
                            |identity, epoch| {
                                fetch_request = Some((identity.to_string(), epoch));
                            },
                            || {
                                clear_needed = true;
                            },
                            || {
                                save_needed = true;
                            },
                        );

                        if let Some((identity, epoch)) = fetch_request {
                            self.fetch_leader_schedule(&identity, epoch);
                        }
                        if clear_needed {
                            self.leader_identity_search.clear();
                            self.leader_epoch_search.clear();
                            if let Ok(mut result) = self.leader_schedule_result.try_lock() {
                                *result = None;
                            }
                            self.save_config_changes();
                        }
                        if save_needed {
                            self.save_config_changes();
                        }
                    }
                    AppTab::Logs => {
                        let mut clear_requested = false;
                        let should_focus = self.should_focus_search;
                        logs::render_logs_tab(
                            ui,
                            &self.log_store,
                            &mut self.logs_search,
                            should_focus,
                            || {
                                clear_requested = true;
                            },
                        );
                        if clear_requested {
                            self.clear_logs();
                        }
                    }
                    AppTab::Update => {
                        self.update_tab.ui(ui, ctx);
                    }
                }
            });

        // Periodic save of configuration (every 30 seconds when app is active)
        if self.last_config_save.elapsed() > std::time::Duration::from_secs(30) {
            self.save_current_state();
            self.last_config_save = Instant::now();
        }

        // Reset focus flag after UI has been rendered
        if self.should_focus_search {
            self.should_focus_search = false;
        }

        // Request repaint at regular intervals for UI updates
        ctx.request_repaint_after(std::time::Duration::from_secs(
            ui_constants::UI_UPDATE_INTERVAL_SECS,
        ));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Perform cleanup when app exits
    }
}
