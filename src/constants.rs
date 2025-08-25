//! Constants used throughout the Solana UI application.

#![allow(dead_code)]

use eframe::egui;

// UI Layout Constants
pub const HEADER_SPACING_LARGE: f32 = 24.0;
pub const HEADER_SPACING_MEDIUM: f32 = 16.0;
pub const HEADER_SPACING_SMALL: f32 = 12.0;
pub const HEADER_SPACING_TINY: f32 = 8.0;
pub const CONTENT_SPACING_SMALL: f32 = 4.0;

// Search Field Constants
pub const SEARCH_FIELD_WIDTH: f32 = 350.0;
pub const SEARCH_FIELD_HEIGHT: f32 = 20.0;
pub const SMALL_SEARCH_FIELD_WIDTH: f32 = 300.0;
pub const EPOCH_FIELD_WIDTH: f32 = 60.0;
pub const BUTTON_FIELD_WIDTH: f32 = 150.0;

// Table Column Widths
pub const COLUMN_PUBKEY_WIDTH: f32 = 350.0;
pub const COLUMN_ADDRESS_WIDTH: f32 = 150.0;
pub const COLUMN_VERSION_WIDTH: f32 = 100.0;
pub const COLUMN_FEATURE_WIDTH: f32 = 80.0;
pub const COLUMN_COMMISSION_WIDTH: f32 = 60.0;
pub const COLUMN_VOTE_CREDITS_WIDTH: f32 = 100.0;
pub const COLUMN_TIME_WIDTH: f32 = 220.0;
pub const COLUMN_SLOT_WIDTH: f32 = 120.0;
pub const COLUMN_EPOCH_WIDTH: f32 = 80.0;
pub const COLUMN_SMALL_INDEX_WIDTH: f32 = 50.0;
pub const COLUMN_VOTE_ACCOUNT_WIDTH: f32 = 350.0;
pub const COLUMN_TRANSACTION_WIDTH: f32 = 450.0;
pub const COLUMN_LOG_TYPE_WIDTH: f32 = 40.0;
pub const COLUMN_LOG_TIMESTAMP_WIDTH: f32 = 150.0;
pub const COLUMN_LOG_OPERATION_WIDTH: f32 = 120.0;
pub const COLUMN_LOG_STATUS_WIDTH: f32 = 80.0;
pub const COLUMN_LOG_URL_WIDTH: f32 = 200.0;
pub const COLUMN_LOG_CONTENT_WIDTH: f32 = 300.0;

// Table Row Heights
pub const TABLE_HEADER_HEIGHT: f32 = 28.0;
pub const TABLE_ROW_HEIGHT: f32 = 24.0;
pub const TABLE_ROW_HEIGHT_LARGE: f32 = 26.0;
pub const TABLE_ROW_HEIGHT_SMALL: f32 = 18.0;
pub const TABLE_ROW_HEIGHT_LOGS: f32 = 20.0;

// Frame and Border Constants
pub const FRAME_INNER_MARGIN: f32 = 12.0;
pub const FRAME_INNER_MARGIN_SMALL: f32 = 8.0;
pub const FRAME_INNER_MARGIN_TINY: f32 = 4.0;
pub const FRAME_CORNER_RADIUS: f32 = 4.0;
pub const FRAME_STROKE_WIDTH: f32 = 1.0;

// Content Display Constants
pub const LOG_MAX_ENTRIES: usize = 1000;
pub const LOG_CONTENT_TRUNCATE_LENGTH: usize = 100;
pub const LOG_CONTENT_DISPLAY_LENGTH: usize = 97;

// Colors
pub const ERROR_BACKGROUND: egui::Color32 =
    egui::Color32::from_rgba_premultiplied(255, 200, 200, 50);
pub const SUCCESS_COLOR: egui::Color32 = egui::Color32::from_rgb(0, 128, 0);

// Log Entry Type Colors
pub const LOG_REQUEST_COLOR: egui::Color32 = egui::Color32::from_rgb(0, 102, 204); // Blue
pub const LOG_RESPONSE_COLOR: egui::Color32 = egui::Color32::from_rgb(0, 128, 0); // Green
pub const LOG_ERROR_COLOR: egui::Color32 = egui::Color32::from_rgb(204, 0, 0); // Red

// Find Voters Colors
pub const VOTER_COLOR_1: egui::Color32 = egui::Color32::from_rgb(200, 230, 255); // Light blue
pub const VOTER_COLOR_2: egui::Color32 = egui::Color32::from_rgb(255, 230, 200); // Light orange

// Sort Priority Constants
pub const PRIMARY_SORT_INDEX: usize = 0;
pub const SORT_PRIORITY_OFFSET: usize = 1;

// Auto-shrink array for scroll areas
pub const SCROLL_AUTO_SHRINK: [bool; 2] = [false; 2];
