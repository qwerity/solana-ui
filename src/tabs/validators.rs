//! Validators tab functionality for the Solana UI application.

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::cmp::Ordering;

use crate::constants::*;
use crate::solana::ValidatorInfo;
use crate::utils::{
    create_error_frame, create_info_frame, format_skip_rate, format_stake, render_search_field,
    SortColumn, SortDirection, SortState,
};

/// Parameters for the validators tab rendering.
pub struct ValidatorsTabParams<'a> {
    pub validators: &'a [ValidatorInfo],
    pub sort_states: &'a [SortState],
    pub search_term: &'a mut String,
    pub error_message: &'a Option<String>,
    pub is_loading: bool,
    pub should_focus_search: bool,
}

/// Render the validators tab content.
pub fn render_validators_tab(
    ui: &mut egui::Ui,
    params: ValidatorsTabParams,
    mut on_sort: impl FnMut(SortColumn, bool),
    mut on_refresh: impl FnMut(),
) {
    let ValidatorsTabParams {
        validators,
        sort_states,
        search_term,
        error_message,
        is_loading,
        should_focus_search,
    } = params;
    ui.horizontal(|ui| {
        ui.heading("Solana Validators");
        ui.add_space(HEADER_SPACING_LARGE);

        // Search bar near headline
        ui.label("🔍 Search:");
        ui.add_space(CONTENT_SPACING_SMALL);
        let _search_response = render_search_field(
            ui,
            search_term,
            "Search validators...",
            should_focus_search,
            SEARCH_FIELD_WIDTH,
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let button = ui
                .button("🔄 Refresh Validators")
                .on_hover_text("Refresh validators data (Cmd+R / Ctrl+R)");
            if button.clicked() {
                on_refresh();
            }
        });
    });
    ui.add_space(HEADER_SPACING_SMALL);
    ui.separator();
    ui.add_space(HEADER_SPACING_MEDIUM);

    // Instructions for multi-column sorting
    render_sort_info(ui, sort_states);

    if let Some(error) = error_message {
        let frame = create_error_frame();

        frame.show(ui, |ui| {
            ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
        });
        ui.add_space(HEADER_SPACING_MEDIUM);
    }

    if validators.is_empty() && !is_loading {
        ui.label("No validators data. Click 'Refresh Validators' to load validators.");
        return;
    }

    // Apply filtering
    let filtered_validators = filter_validators(validators, search_term);

    // Show filter results info
    if !search_term.is_empty() {
        ui.horizontal(|ui| {
            ui.label(format!(
                "📊 Showing {} of {} validators (filtered)",
                filtered_validators.len(),
                validators.len()
            ));
        });
    } else {
        ui.horizontal(|ui| {
            ui.label(format!(
                "📊 Showing {} validators",
                filtered_validators.len()
            ));
        });
    }

    // Apply sorting
    let mut sorted_validators = filtered_validators;
    sort_validators(&mut sorted_validators, sort_states);

    // Create table
    render_validators_table(ui, &sorted_validators, sort_states, on_sort);
}

/// Render sorting information.
fn render_sort_info(ui: &mut egui::Ui, sort_states: &[SortState]) {
    let frame = create_info_frame(ui);

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("💡 Tip: Click column headers to sort. Hold Shift + click to add secondary/tertiary sorts.");
            if !sort_states.is_empty() {
                ui.add_space(HEADER_SPACING_MEDIUM);
                ui.colored_label(
                    ui.visuals().widgets.active.text_color(),
                    format!("📊 Sorting by {} column(s)", sort_states.len())
                );
            }
        });
    });
    ui.add_space(HEADER_SPACING_MEDIUM);
}

/// Filter validators based on search term.
fn filter_validators(validators: &[ValidatorInfo], search_term: &str) -> Vec<ValidatorInfo> {
    if search_term.is_empty() {
        return validators.to_vec();
    }

    let search_lower = search_term.to_lowercase();
    validators
        .iter()
        .filter(|validator| {
            // Search in identity, vote account, version, and other text fields
            validator
                .identity
                .to_string()
                .to_lowercase()
                .contains(&search_lower)
                || validator
                    .vote_account
                    .to_string()
                    .to_lowercase()
                    .contains(&search_lower)
                || validator.version.to_lowercase().contains(&search_lower)
                || validator.commission.to_string().contains(&search_lower)
                || validator.last_vote.to_string().contains(&search_lower)
                || validator.root_slot.to_string().contains(&search_lower)
                || validator.vote_credits.to_string().contains(&search_lower)
        })
        .cloned()
        .collect()
}

/// Sort validators based on sort states.
pub fn sort_validators(validators: &mut [ValidatorInfo], sort_states: &[SortState]) {
    validators.sort_by(|a, b| {
        for sort_state in sort_states {
            let comparison = match sort_state.column {
                SortColumn::Identity => a.identity.cmp(&b.identity),
                SortColumn::VoteAccount => a.vote_account.cmp(&b.vote_account),
                SortColumn::Commission => a.commission.cmp(&b.commission),
                SortColumn::LastVote => a.last_vote.cmp(&b.last_vote),
                SortColumn::RootSlot => a.root_slot.cmp(&b.root_slot),
                SortColumn::VoteCredits => a.vote_credits.cmp(&b.vote_credits),
                SortColumn::ActivatedStake => a.activated_stake.cmp(&b.activated_stake),
                SortColumn::Version => a.version.cmp(&b.version),
                SortColumn::SkipRate => a
                    .skip_rate
                    .partial_cmp(&b.skip_rate)
                    .unwrap_or(Ordering::Equal),
            };

            let final_comparison = match sort_state.direction {
                SortDirection::Ascending => comparison,
                SortDirection::Descending => comparison.reverse(),
            };

            if final_comparison != Ordering::Equal {
                return final_comparison;
            }
        }
        Ordering::Equal
    });
}

/// Get sort indicator for a column.
pub fn get_sort_indicator(sort_states: &[SortState], column: SortColumn) -> String {
    if let Some(sort_state) = sort_states.iter().find(|s| s.column == column) {
        let arrow = match sort_state.direction {
            SortDirection::Ascending => "▲",
            SortDirection::Descending => "▼",
        };
        if sort_states.len() == 1 || sort_state.priority == 0 {
            format!(" {}", arrow)
        } else {
            format!(" {}({})", arrow, sort_state.priority + 1)
        }
    } else {
        String::new()
    }
}

/// Render the validators table.
fn render_validators_table(
    ui: &mut egui::Ui,
    validators: &[ValidatorInfo],
    sort_states: &[SortState],
    mut on_sort: impl FnMut(SortColumn, bool),
) {
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(COLUMN_PUBKEY_WIDTH)) // Identity (full base58)
        .column(Column::auto().at_least(COLUMN_PUBKEY_WIDTH)) // Vote Account (full base58)
        .column(Column::auto().at_least(COLUMN_COMMISSION_WIDTH)) // Commission
        .column(Column::auto().at_least(COLUMN_FEATURE_WIDTH)) // Last Vote Slot
        .column(Column::auto().at_least(COLUMN_FEATURE_WIDTH)) // Root Slot
        .column(Column::auto().at_least(COLUMN_VOTE_CREDITS_WIDTH)) // Vote Credits
        .column(Column::auto().at_least(COLUMN_FEATURE_WIDTH)) // Skip Rate
        .column(Column::auto().at_least(COLUMN_VOTE_CREDITS_WIDTH)) // Activated Stake
        .column(Column::auto().at_least(COLUMN_FEATURE_WIDTH)) // Version
        .header(TABLE_HEADER_HEIGHT, |mut header| {
            render_table_headers(&mut header, sort_states, on_sort);
        })
        .body(|mut body| {
            for validator in validators.iter() {
                body.row(TABLE_ROW_HEIGHT, |mut row| {
                    render_validator_row(&mut row, validator);
                });
            }
        });
}

/// Render table headers with sorting.
fn render_table_headers(
    header: &mut egui_extras::TableRow<'_, '_>,
    sort_states: &[SortState],
    mut on_sort: impl FnMut(SortColumn, bool),
) {
    let headers = [
        (SortColumn::Identity, "Identity"),
        (SortColumn::VoteAccount, "Vote Account"),
        (SortColumn::Commission, "Commission"),
        (SortColumn::LastVote, "Last Vote Slot"),
        (SortColumn::RootSlot, "Root Slot"),
        (SortColumn::VoteCredits, "Vote Credits"),
        (SortColumn::SkipRate, "Skip Rate"),
        (SortColumn::ActivatedStake, "Activated Stake"),
        (SortColumn::Version, "Version"),
    ];

    for (sort_column, title) in headers {
        header.col(|ui| {
            let text = format!("{}{}", title, get_sort_indicator(sort_states, sort_column));
            let response = ui.button(text);
            if response.clicked() {
                let shift_pressed = ui.input(|i| i.modifiers.shift);
                on_sort(sort_column, shift_pressed);
            }
        });
    }
}

/// Render a single validator row.
fn render_validator_row(row: &mut egui_extras::TableRow<'_, '_>, validator: &ValidatorInfo) {
    row.col(|ui| {
        ui.monospace(validator.identity.to_string());
    });
    row.col(|ui| {
        ui.monospace(validator.vote_account.to_string());
    });
    row.col(|ui| {
        ui.label(format!("{}%", validator.commission));
    });
    row.col(|ui| {
        ui.label(validator.last_vote.to_string());
    });
    row.col(|ui| {
        ui.label(validator.root_slot.to_string());
    });
    row.col(|ui| {
        ui.label(validator.vote_credits.to_string());
    });
    row.col(|ui| {
        ui.label(format_skip_rate(validator.skip_rate));
    });
    row.col(|ui| {
        ui.label(format_stake(validator.activated_stake));
    });
    row.col(|ui| {
        ui.label(&validator.version);
    });
}
