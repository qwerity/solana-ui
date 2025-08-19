//! Leader schedule tab functionality for the Solana UI application.

use chrono::Utc;
use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::constants::*;
use crate::solana::{LeaderScheduleInfo, SolanaClient};
use crate::utils::create_error_frame;

/// Parameters for the leader schedule tab rendering.
#[allow(dead_code)]
pub struct LeaderScheduleTabParams<'a> {
    pub leader_identity_search: &'a mut String,
    pub leader_epoch_search: &'a mut String,
    pub leader_result: &'a Option<LeaderScheduleInfo>,
    pub error_message: &'a Option<String>,
    pub is_loading: bool,
}

/// Render the leader schedule tab content.
pub fn render_leader_schedule_tab(
    ui: &mut egui::Ui,
    params: LeaderScheduleTabParams,
    mut on_fetch_schedule: impl FnMut(&str, Option<u64>),
    mut on_clear: impl FnMut(),
    mut on_search_change: impl FnMut(),
) {
    let LeaderScheduleTabParams {
        leader_identity_search,
        leader_epoch_search,
        leader_result,
        error_message,
        is_loading,
    } = params;
    // Header with inline search controls
    ui.horizontal(|ui| {
        ui.heading("Leader Schedule");
        ui.add_space(HEADER_SPACING_LARGE);

        // Compact search controls right next to the heading
        ui.label("🔑 Identity:");
        ui.add_space(CONTENT_SPACING_SMALL);
        let identity_response = ui
            .add_sized(
                [SEARCH_FIELD_WIDTH, SEARCH_FIELD_HEIGHT],
                egui::TextEdit::singleline(leader_identity_search)
                    .hint_text("Enter validator identity..."),
            )
            .on_hover_text("Enter validator identity public key (base58)");

        ui.add_space(HEADER_SPACING_TINY);
        ui.label("📅 Epoch:");
        ui.add_space(CONTENT_SPACING_SMALL);
        let epoch_response = ui
            .add_sized(
                [EPOCH_FIELD_WIDTH, SEARCH_FIELD_HEIGHT],
                egui::TextEdit::singleline(leader_epoch_search).hint_text("Current"),
            )
            .on_hover_text("Leave empty for current epoch");

        ui.add_space(HEADER_SPACING_TINY);
        if (ui
            .button("🔍 Fetch")
            .on_hover_text("Fetch leader schedule (Cmd+R / Ctrl+R)")
            .clicked()
            || ui.input(|i| i.key_pressed(egui::Key::Enter) && !leader_identity_search.is_empty()))
            && !leader_identity_search.trim().is_empty()
        {
            let epoch = if leader_epoch_search.is_empty() {
                None
            } else {
                leader_epoch_search.parse::<u64>().ok()
            };
            on_fetch_schedule(leader_identity_search.trim(), epoch);
        }

        ui.add_space(CONTENT_SPACING_SMALL);
        if ui.button("🗑").on_hover_text("Clear all").clicked() {
            on_clear();
        }

        // Save if leader schedule fields changed
        if identity_response.changed() || epoch_response.changed() {
            on_search_change();
        }
    });

    ui.add_space(HEADER_SPACING_TINY);
    ui.separator();
    ui.add_space(HEADER_SPACING_SMALL);

    if let Some(error) = error_message {
        let frame = create_error_frame();

        frame.show(ui, |ui| {
            ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
        });
        ui.add_space(HEADER_SPACING_MEDIUM);
    }

    // Display results
    if let Some(result) = leader_result {
        // Display results info
        ui.horizontal(|ui| {
            ui.label(format!(
                "📊 Validator {}: {} leader slots in epoch {}",
                result.validator_identity, result.total_slots, result.target_epoch
            ));

            // Show next upcoming slot info
            if let Some(next_slot) = &result.next_leader_slot {
                ui.separator();
                // Recalculate time difference for current timestamp
                let current_timestamp = Utc::now().timestamp();
                let updated_time_diff = SolanaClient::format_time_difference(
                    current_timestamp,
                    next_slot.time_local.timestamp(),
                );
                ui.colored_label(
                    SUCCESS_COLOR,
                    format!("⏰ Next: Slot {} in {}", next_slot.slot, updated_time_diff),
                );
            }
        });

        if !result.leader_slots.is_empty() {
            render_leader_schedule_table(ui, result);
        } else {
            ui.label(format!(
                "No leader slots found for validator {} in epoch {}",
                result.validator_identity, result.target_epoch
            ));
        }
    } else if !is_loading && !leader_identity_search.is_empty() {
        ui.label("Enter a validator identity and click 'Fetch Schedule' to get leader slots.");
    } else if is_loading {
        ui.label("Fetching leader schedule...");
    }
}

/// Render the leader schedule table with auto-scroll functionality.
fn render_leader_schedule_table(ui: &mut egui::Ui, leader_info: &LeaderScheduleInfo) {
    let current_timestamp = Utc::now().timestamp();
    let mut next_upcoming_index = None;

    // Find the index of the next upcoming slot
    for (index, leader_slot) in leader_info.leader_slots.iter().enumerate() {
        if leader_slot.time_local.timestamp() > current_timestamp {
            next_upcoming_index = Some(index);
            break;
        }
    }

    egui::ScrollArea::vertical()
        .auto_shrink(SCROLL_AUTO_SHRINK)
        .show(ui, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto().at_least(COLUMN_EPOCH_WIDTH)) // Epoch
                .column(Column::auto().at_least(COLUMN_SLOT_WIDTH)) // Slot
                .column(Column::auto().at_least(COLUMN_TIME_WIDTH)) // Time (Local)
                .column(Column::auto().at_least(COLUMN_SLOT_WIDTH)) // Time Diff
                .header(TABLE_HEADER_HEIGHT, |mut header| {
                    header.col(|ui| {
                        ui.heading("Epoch");
                    });
                    header.col(|ui| {
                        ui.heading("Slot");
                    });
                    header.col(|ui| {
                        ui.heading("Time (Local)");
                    });
                    header.col(|ui| {
                        ui.heading("Time Diff");
                    });
                })
                .body(|mut body| {
                    for (index, leader_slot) in leader_info.leader_slots.iter().enumerate() {
                        let is_next_upcoming = next_upcoming_index == Some(index);
                        let row_height = if is_next_upcoming {
                            TABLE_ROW_HEIGHT_LARGE
                        } else {
                            TABLE_ROW_HEIGHT
                        };

                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                if is_next_upcoming {
                                    ui.colored_label(
                                        SUCCESS_COLOR,
                                        format!("➤ {}", leader_slot.epoch),
                                    );
                                } else {
                                    ui.label(leader_slot.epoch.to_string());
                                }
                            });
                            row.col(|ui| {
                                if is_next_upcoming {
                                    ui.colored_label(SUCCESS_COLOR, leader_slot.slot.to_string());
                                } else {
                                    ui.monospace(leader_slot.slot.to_string());
                                }
                            });
                            row.col(|ui| {
                                let time_str = leader_slot
                                    .time_local
                                    .format("%Y-%m-%d %H:%M:%S %:z")
                                    .to_string();
                                if is_next_upcoming {
                                    ui.colored_label(SUCCESS_COLOR, time_str);
                                } else {
                                    ui.label(time_str);
                                }
                            });
                            row.col(|ui| {
                                // Recalculate time difference for current timestamp
                                let updated_time_diff = SolanaClient::format_time_difference(
                                    current_timestamp,
                                    leader_slot.time_local.timestamp(),
                                );
                                if is_next_upcoming {
                                    ui.colored_label(
                                        SUCCESS_COLOR,
                                        format!("⏰ {}", updated_time_diff),
                                    );
                                } else {
                                    ui.label(updated_time_diff);
                                }
                            });

                            // Auto-scroll to next upcoming row (after all columns are set)
                            if is_next_upcoming {
                                row.response().scroll_to_me(None);
                            }
                        });
                    }
                });
        });
}
