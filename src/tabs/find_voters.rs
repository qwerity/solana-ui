//! Find voters tab functionality for the Solana UI application.

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::collections::{HashMap, HashSet};

use crate::constants::*;
use crate::solana::SlotVoterInfo;
use crate::utils::{create_cell_frame, create_error_frame, render_search_field};

/// Parameters for the find voters tab rendering.
pub struct FindVotersTabParams<'a> {
    pub slot_search: &'a mut String,
    pub voter_result: &'a Option<SlotVoterInfo>,
    pub search_term: &'a mut String,
    pub error_message: &'a Option<String>,
    pub is_loading: bool,
    pub should_focus_search: bool,
}

/// Render the find voters tab content.
pub fn render_find_voters_tab(
    ui: &mut egui::Ui,
    params: FindVotersTabParams,
    mut on_search_voters: impl FnMut(u64),
    mut on_clear: impl FnMut(),
    mut on_search_change: impl FnMut(),
) {
    let FindVotersTabParams {
        slot_search,
        voter_result,
        search_term,
        error_message,
        is_loading,
        should_focus_search,
    } = params;
    // Header with inline search controls
    ui.horizontal(|ui| {
        ui.heading("Find Voters in Slot");
        ui.add_space(HEADER_SPACING_LARGE);

        // Search bar near headline for filtering results
        ui.label("🔍 Filter:");
        ui.add_space(CONTENT_SPACING_SMALL);
        let _search_response = render_search_field(
            ui,
            search_term,
            "Filter voters...",
            should_focus_search,
            SMALL_SEARCH_FIELD_WIDTH,
        );
    });

    ui.add_space(HEADER_SPACING_TINY);
    ui.separator();
    ui.add_space(HEADER_SPACING_MEDIUM);

    // Slot input section
    let frame = egui::Frame::group(ui.style())
        .inner_margin(FRAME_INNER_MARGIN)
        .stroke(egui::Stroke::new(
            FRAME_STROKE_WIDTH,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("🔍 Slot Number:");
            ui.add_space(8.0);
            let slot_response = ui
                .add_sized(
                    [BUTTON_FIELD_WIDTH, SEARCH_FIELD_HEIGHT],
                    egui::TextEdit::singleline(slot_search).hint_text("Enter slot..."),
                )
                .on_hover_text("Enter a slot number to find voters");

            ui.add_space(16.0);
            if ui
                .button("🔍 Search Voters")
                .on_hover_text("Search for voters in slot (Cmd+R / Ctrl+R)")
                .clicked()
                || ui.input(|i| i.key_pressed(egui::Key::Enter) && !slot_search.is_empty())
            {
                if let Ok(slot) = slot_search.parse::<u64>() {
                    on_search_voters(slot);
                }
            }

            ui.add_space(8.0);
            if ui.button("🗑 Clear").clicked() {
                on_clear();
            }

            // Save if slot search changed
            if slot_response.changed() {
                on_search_change();
            }
        });
    });
    ui.add_space(HEADER_SPACING_SMALL);

    if let Some(error) = error_message {
        let frame = create_error_frame();

        frame.show(ui, |ui| {
            ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
        });
        ui.add_space(HEADER_SPACING_MEDIUM);
    }

    // Display results
    if let Some(result) = voter_result {
        // Apply filtering to vote transactions
        let filtered_vote_transactions =
            filter_vote_transactions(&result.vote_transactions, search_term);

        // Display results info with filtering status
        ui.horizontal(|ui| {
            if search_term.is_empty() {
                ui.label(format!(
                    "📊 Slot {}: Found {} voters",
                    result.slot, result.total_voters
                ));
            } else {
                ui.label(format!(
                    "📊 Slot {}: Showing {} of {} voters (filtered)",
                    result.slot,
                    filtered_vote_transactions.len(),
                    result.total_voters
                ));
            }
        });

        if !filtered_vote_transactions.is_empty() {
            // Create filtered voters set from transactions
            let filtered_voters: HashSet<String> = filtered_vote_transactions
                .iter()
                .map(|tx| tx.vote_account.clone())
                .collect();

            // Create filtered result
            let filtered_result = SlotVoterInfo {
                slot: result.slot,
                voters: filtered_voters,
                vote_transactions: filtered_vote_transactions,
                total_voters: result.total_voters,
            };
            render_voters_table(ui, &filtered_result);
        } else if search_term.is_empty() {
            ui.label("No voters found in this slot.");
        } else {
            ui.label(format!("No voters match the search term '{}'", search_term));
        }
    } else if !is_loading && !slot_search.is_empty() {
        ui.label("Enter a slot number and click 'Search Voters' to find voters.");
    } else if is_loading {
        ui.label("Searching for voters...");
    }
}

/// Filter vote transactions based on search term.
fn filter_vote_transactions(
    vote_transactions: &[crate::solana::VoteTransactionInfo],
    search_term: &str,
) -> Vec<crate::solana::VoteTransactionInfo> {
    if search_term.is_empty() {
        return vote_transactions.to_vec();
    }

    let search_lower = search_term.to_lowercase();
    vote_transactions
        .iter()
        .filter(|tx| {
            // Search in vote account and transaction signature
            tx.vote_account.to_lowercase().contains(&search_lower)
                || tx.signature.to_lowercase().contains(&search_lower)
        })
        .cloned()
        .collect()
}

/// Render the voters table with transaction signatures and alternating colors per vote account.
fn render_voters_table(ui: &mut egui::Ui, voter_info: &SlotVoterInfo) {
    // Define alternating colors for vote account groups - making them very distinct for testing
    let color1 = VOTER_COLOR_1;
    let color2 = VOTER_COLOR_2;

    // Sort transactions by vote account to group them together
    let mut sorted_transactions = voter_info.vote_transactions.clone();
    sorted_transactions.sort_by(|a, b| a.vote_account.cmp(&b.vote_account));

    // Create mapping of vote accounts to color indices
    let mut vote_account_colors = HashMap::new();
    let mut unique_accounts = Vec::new();

    // Collect unique vote accounts in order
    for vote_tx in &sorted_transactions {
        if !unique_accounts.contains(&vote_tx.vote_account) {
            unique_accounts.push(vote_tx.vote_account.clone());
        }
    }

    // Assign alternating colors to unique accounts
    for (index, account) in unique_accounts.iter().enumerate() {
        let color_index = index % 2;
        vote_account_colors.insert(account.clone(), color_index);
    }

    TableBuilder::new(ui)
        .striped(false) // Disable default striping since we're doing custom colors
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(COLUMN_SMALL_INDEX_WIDTH)) // Index
        .column(Column::auto().at_least(COLUMN_VOTE_ACCOUNT_WIDTH)) // Vote Account
        .column(Column::auto().at_least(COLUMN_TRANSACTION_WIDTH)) // Transaction Signature
        .header(TABLE_HEADER_HEIGHT, |mut header| {
            header.col(|ui| {
                ui.heading("#");
            });
            header.col(|ui| {
                ui.heading("Vote Account Public Key");
            });
            header.col(|ui| {
                ui.heading("Transaction Signature");
            });
        })
        .body(|mut body| {
            for (index, vote_tx) in sorted_transactions.iter().enumerate() {
                let color_index = vote_account_colors.get(&vote_tx.vote_account).unwrap_or(&0);
                let bg_color = if *color_index == 0 { color1 } else { color2 };

                body.row(TABLE_ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.scope(|ui| {
                            ui.visuals_mut().panel_fill = bg_color;
                            ui.visuals_mut().window_fill = bg_color;
                            let frame = create_cell_frame(bg_color);
                            frame.show(ui, |ui| {
                                ui.label((index + 1).to_string());
                            });
                        });
                    });
                    row.col(|ui| {
                        ui.scope(|ui| {
                            ui.visuals_mut().panel_fill = bg_color;
                            ui.visuals_mut().window_fill = bg_color;
                            let frame = create_cell_frame(bg_color);
                            frame.show(ui, |ui| {
                                ui.monospace(&vote_tx.vote_account);
                            });
                        });
                    });
                    row.col(|ui| {
                        ui.scope(|ui| {
                            ui.visuals_mut().panel_fill = bg_color;
                            ui.visuals_mut().window_fill = bg_color;
                            let frame = create_cell_frame(bg_color);
                            frame.show(ui, |ui| {
                                ui.monospace(&vote_tx.signature);
                            });
                        });
                    });
                });
            }
        });
}
