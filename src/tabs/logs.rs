//! Logs tab functionality for the Solana UI application.

use chrono::{DateTime, Local};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::constants::*;
use crate::utils::render_search_field;

/// A single log entry for RPC requests/responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp when the log was created
    pub timestamp: DateTime<Local>,
    /// Type of log entry (Request, Response, Error)
    pub entry_type: LogEntryType,
    /// The operation being performed (e.g., "get_block", "get_leader_schedule")
    pub operation: String,
    /// The request URL or endpoint
    pub url: String,
    /// Request parameters or response body
    pub content: String,
    /// Status or error code
    pub status: String,
}

/// Type of log entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogEntryType {
    Request,
    Response,
    Error,
    Update,
}

impl LogEntryType {
    pub fn icon(&self) -> &'static str {
        match self {
            LogEntryType::Request => "⬆️",
            LogEntryType::Response => "⬇️",
            LogEntryType::Error => "❌",
            LogEntryType::Update => "🔄",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            LogEntryType::Request => LOG_REQUEST_COLOR,
            LogEntryType::Response => LOG_RESPONSE_COLOR,
            LogEntryType::Error => LOG_ERROR_COLOR,
            LogEntryType::Update => egui::Color32::from_rgb(100, 149, 237),
        }
    }
}

/// Global log storage.
pub type LogStore = Arc<Mutex<Vec<LogEntry>>>;

/// Create a new log store.
pub fn create_log_store() -> LogStore {
    Arc::new(Mutex::new(Vec::new()))
}

/// Add a log entry to the store.
pub fn add_log_entry(store: &LogStore, entry: LogEntry) {
    if let Ok(mut logs) = store.lock() {
        logs.push(entry);
        // Keep only the last entries to prevent memory issues
        if logs.len() > LOG_MAX_ENTRIES {
            let len = logs.len();
            logs.drain(0..len - LOG_MAX_ENTRIES);
        }
    }
}

/// Log an RPC request.
pub fn log_request(store: &LogStore, operation: &str, url: &str, params: &str) {
    let entry = LogEntry {
        timestamp: Local::now(),
        entry_type: LogEntryType::Request,
        operation: operation.to_string(),
        url: url.to_string(),
        content: params.to_string(),
        status: "Sent".to_string(),
    };
    add_log_entry(store, entry);
}

/// Log an RPC response.
pub fn log_response(store: &LogStore, operation: &str, url: &str, response: &str, status: &str) {
    let entry = LogEntry {
        timestamp: Local::now(),
        entry_type: LogEntryType::Response,
        operation: operation.to_string(),
        url: url.to_string(),
        content: response.to_string(),
        status: status.to_string(),
    };
    add_log_entry(store, entry);
}

/// Log an RPC error.
pub fn log_error(store: &LogStore, operation: &str, url: &str, error: &str) {
    let entry = LogEntry {
        timestamp: Local::now(),
        entry_type: LogEntryType::Error,
        operation: operation.to_string(),
        url: url.to_string(),
        content: error.to_string(),
        status: "Error".to_string(),
    };
    add_log_entry(store, entry);
}

/// Log an update-related event.
pub fn log_update(store: &LogStore, operation: &str, message: &str, status: &str) {
    let entry = LogEntry {
        timestamp: Local::now(),
        entry_type: LogEntryType::Update,
        operation: operation.to_string(),
        url: "updater".to_string(),
        content: message.to_string(),
        status: status.to_string(),
    };
    add_log_entry(store, entry);
}

/// Render the logs tab content.
pub fn render_logs_tab(
    ui: &mut egui::Ui,
    log_store: &LogStore,
    search_term: &mut String,
    should_focus_search: bool,
    mut on_clear_logs: impl FnMut(),
) {
    ui.horizontal(|ui| {
        ui.heading("RPC Logs");
        ui.add_space(HEADER_SPACING_LARGE);

        // Search bar near headline
        ui.label("🔍 Search:");
        ui.add_space(CONTENT_SPACING_SMALL);
        let _search_response = render_search_field(
            ui,
            search_term,
            "Search logs...",
            should_focus_search,
            SEARCH_FIELD_WIDTH,
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("🗑 Clear Logs").clicked() {
                on_clear_logs();
            }
        });
    });
    ui.add_space(HEADER_SPACING_TINY);
    ui.separator();
    ui.add_space(HEADER_SPACING_SMALL);

    // Get current logs
    let logs = if let Ok(guard) = log_store.lock() {
        guard.clone()
    } else {
        Vec::new()
    };

    if logs.is_empty() {
        ui.label("No logs yet. RPC requests and responses will appear here.");
        return;
    }

    // Apply filtering
    let filtered_logs = filter_logs(&logs, search_term);

    ui.horizontal(|ui| {
        if search_term.is_empty() {
            ui.label(format!("📊 Showing {} log entries", logs.len()));
        } else {
            ui.label(format!(
                "📊 Showing {} of {} log entries (filtered)",
                filtered_logs.len(),
                logs.len()
            ));
        }
    });

    // Create logs table with auto-scroll to bottom
    egui::ScrollArea::vertical()
        .auto_shrink(SCROLL_AUTO_SHRINK)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            render_logs_table(ui, &filtered_logs);
        });
}

/// Render the logs table.
fn render_logs_table(ui: &mut egui::Ui, logs: &[LogEntry]) {
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(COLUMN_LOG_TYPE_WIDTH)) // Type icon
        .column(Column::auto().at_least(COLUMN_LOG_TIMESTAMP_WIDTH)) // Timestamp
        .column(Column::auto().at_least(COLUMN_LOG_OPERATION_WIDTH)) // Operation
        .column(Column::auto().at_least(COLUMN_LOG_STATUS_WIDTH)) // Status
        .column(Column::auto().at_least(COLUMN_LOG_URL_WIDTH)) // URL
        .column(Column::remainder().at_least(COLUMN_LOG_CONTENT_WIDTH)) // Content
        .header(TABLE_ROW_HEIGHT_LOGS, |mut header| {
            header.col(|ui| {
                ui.heading("Type");
            });
            header.col(|ui| {
                ui.heading("Time");
            });
            header.col(|ui| {
                ui.heading("Operation");
            });
            header.col(|ui| {
                ui.heading("Status");
            });
            header.col(|ui| {
                ui.heading("URL");
            });
            header.col(|ui| {
                ui.heading("Content");
            });
        })
        .body(|mut body| {
            // Show logs in reverse order (newest first)
            for log_entry in logs.iter().rev() {
                body.row(TABLE_ROW_HEIGHT_SMALL, |mut row| {
                    render_log_row(&mut row, log_entry);
                });
            }
        });
}

/// Render a single log row.
fn render_log_row(row: &mut egui_extras::TableRow<'_, '_>, entry: &LogEntry) {
    row.col(|ui| {
        ui.colored_label(entry.entry_type.color(), entry.entry_type.icon());
    });
    row.col(|ui| {
        ui.label(entry.timestamp.format("%H:%M:%S").to_string());
    });
    row.col(|ui| {
        ui.label(&entry.operation);
    });
    row.col(|ui| {
        ui.colored_label(entry.entry_type.color(), &entry.status);
    });
    row.col(|ui| {
        ui.monospace(&entry.url);
    });
    row.col(|ui| {
        // Truncate very long content for display
        let display_content = if entry.content.len() > LOG_CONTENT_TRUNCATE_LENGTH {
            format!("{}...", &entry.content[..LOG_CONTENT_DISPLAY_LENGTH])
        } else {
            entry.content.clone()
        };
        ui.label(display_content);
    });
}

/// Filter logs based on search term.
fn filter_logs(logs: &[LogEntry], search_term: &str) -> Vec<LogEntry> {
    if search_term.is_empty() {
        return logs.to_vec();
    }

    let search_lower = search_term.to_lowercase();
    logs.iter()
        .filter(|log| {
            log.operation.to_lowercase().contains(&search_lower)
                || log.url.to_lowercase().contains(&search_lower)
                || log.content.to_lowercase().contains(&search_lower)
                || log.status.to_lowercase().contains(&search_lower)
        })
        .cloned()
        .collect()
}
