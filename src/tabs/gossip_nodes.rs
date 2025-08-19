//! Gossip nodes tab functionality for the Solana UI application.

use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::constants::*;
use crate::solana::GossipNodeInfo;
use crate::utils::{create_error_frame, render_search_field};

/// Render the gossip nodes tab content.
pub fn render_gossip_nodes_tab(
    ui: &mut egui::Ui,
    gossip_nodes: &[GossipNodeInfo],
    search_term: &mut String,
    error_message: &Option<String>,
    is_loading: bool,
    should_focus_search: bool,
    mut on_refresh: impl FnMut(),
) {
    ui.horizontal(|ui| {
        ui.heading("Gossip Network Nodes");
        ui.add_space(HEADER_SPACING_LARGE);

        // Search bar near headline
        ui.label("🔍 Search:");
        ui.add_space(CONTENT_SPACING_SMALL);
        let _search_response = render_search_field(
            ui,
            search_term,
            "Search nodes...",
            should_focus_search,
            SEARCH_FIELD_WIDTH,
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .button("🔄 Refresh Nodes")
                .on_hover_text("Refresh gossip nodes data (Cmd+R / Ctrl+R)")
                .clicked()
            {
                on_refresh();
            }
        });
    });
    ui.add_space(HEADER_SPACING_SMALL);
    ui.separator();
    ui.add_space(HEADER_SPACING_MEDIUM);

    if let Some(error) = error_message {
        let frame = create_error_frame();

        frame.show(ui, |ui| {
            ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
        });
        ui.add_space(HEADER_SPACING_MEDIUM);
    }

    if gossip_nodes.is_empty() && !is_loading {
        ui.label("No gossip nodes data. Click 'Refresh Nodes' to load gossip nodes.");
    } else {
        // Apply filtering
        let filtered_nodes = filter_gossip_nodes(gossip_nodes, search_term);

        // Show filter results info
        if !search_term.is_empty() {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "🌐 Showing {} of {} gossip nodes (filtered)",
                    filtered_nodes.len(),
                    gossip_nodes.len()
                ));
            });
        } else {
            ui.horizontal(|ui| {
                ui.label(format!("🌐 Showing {} gossip nodes", filtered_nodes.len()));
            });
        }

        // Create gossip nodes table
        render_gossip_nodes_table(ui, &filtered_nodes);
    }
}

/// Render the gossip nodes table.
fn render_gossip_nodes_table(ui: &mut egui::Ui, gossip_nodes: &[GossipNodeInfo]) {
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(COLUMN_PUBKEY_WIDTH)) // Pubkey
        .column(Column::auto().at_least(COLUMN_ADDRESS_WIDTH)) // Gossip Address
        .column(Column::auto().at_least(COLUMN_ADDRESS_WIDTH)) // TPU Address
        .column(Column::auto().at_least(COLUMN_ADDRESS_WIDTH)) // RPC Address
        .column(Column::auto().at_least(COLUMN_ADDRESS_WIDTH)) // TPU QUIC Address
        .column(Column::auto().at_least(COLUMN_VERSION_WIDTH)) // Version
        .column(Column::auto().at_least(COLUMN_FEATURE_WIDTH)) // Feature Set
        .column(Column::auto().at_least(COLUMN_FEATURE_WIDTH)) // Shred Version
        .header(TABLE_HEADER_HEIGHT, |mut header| {
            header.col(|ui| {
                ui.heading("Pubkey");
            });
            header.col(|ui| {
                ui.heading("Gossip Address");
            });
            header.col(|ui| {
                ui.heading("TPU Address");
            });
            header.col(|ui| {
                ui.heading("RPC Address");
            });
            header.col(|ui| {
                ui.heading("TPU QUIC Address");
            });
            header.col(|ui| {
                ui.heading("Version");
            });
            header.col(|ui| {
                ui.heading("Feature Set");
            });
            header.col(|ui| {
                ui.heading("Shred Version");
            });
        })
        .body(|mut body| {
            for node in gossip_nodes.iter() {
                body.row(TABLE_ROW_HEIGHT, |mut row| {
                    render_gossip_node_row(&mut row, node);
                });
            }
        });
}

/// Render a single gossip node row.
fn render_gossip_node_row(row: &mut egui_extras::TableRow<'_, '_>, node: &GossipNodeInfo) {
    row.col(|ui| {
        ui.monospace(node.pubkey.to_string());
    });
    row.col(|ui| {
        ui.label(&node.gossip);
    });
    row.col(|ui| {
        ui.label(node.tpu.as_deref().unwrap_or("N/A"));
    });
    row.col(|ui| {
        ui.label(node.rpc.as_deref().unwrap_or("N/A"));
    });
    row.col(|ui| {
        ui.label(node.tpu_quic.as_deref().unwrap_or("N/A"));
    });
    row.col(|ui| {
        ui.label(node.version.as_deref().unwrap_or("Unknown"));
    });
    row.col(|ui| {
        if let Some(feature_set) = node.feature_set {
            ui.label(feature_set.to_string());
        } else {
            ui.label("N/A");
        }
    });
    row.col(|ui| {
        if let Some(shred_version) = node.shred_version {
            ui.label(shred_version.to_string());
        } else {
            ui.label("N/A");
        }
    });
}

/// Filter gossip nodes based on search term.
fn filter_gossip_nodes(nodes: &[GossipNodeInfo], search_term: &str) -> Vec<GossipNodeInfo> {
    if search_term.is_empty() {
        return nodes.to_vec();
    }

    let search_lower = search_term.to_lowercase();
    nodes
        .iter()
        .filter(|node| {
            // Search in pubkey, addresses, version, and other text fields
            node.pubkey
                .to_string()
                .to_lowercase()
                .contains(&search_lower)
                || node.gossip.to_lowercase().contains(&search_lower)
                || node
                    .tpu
                    .as_ref()
                    .is_some_and(|s| s.to_lowercase().contains(&search_lower))
                || node
                    .rpc
                    .as_ref()
                    .is_some_and(|s| s.to_lowercase().contains(&search_lower))
                || node
                    .tpu_quic
                    .as_ref()
                    .is_some_and(|s| s.to_lowercase().contains(&search_lower))
                || node
                    .version
                    .as_ref()
                    .is_some_and(|s| s.to_lowercase().contains(&search_lower))
                || node
                    .feature_set
                    .is_some_and(|f| f.to_string().contains(&search_lower))
                || node
                    .shred_version
                    .is_some_and(|s| s.to_string().contains(&search_lower))
        })
        .cloned()
        .collect()
}
