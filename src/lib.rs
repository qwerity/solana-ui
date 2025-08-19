//! # Solana Validators UI
//!
//! A GUI application for monitoring Solana validators with real-time data fetching,
//! multi-column sorting, and search functionality.
//!
//! ## Module Structure
//!
//! - [`config`] - Configuration management and persistence
//! - [`constants`] - Application constants and magic numbers
//! - [`solana`] - Solana RPC client and data models
//! - [`tabs`] - Individual tab functionality modules
//! - [`ui`] - Main application UI orchestration
//! - [`updater`] - Auto-updater for GitHub releases
//! - [`utils`] - Utility functions, sorting, and status management

pub mod config;
pub mod constants;
pub mod solana;
pub mod tabs;
pub mod ui;
pub mod updater;
pub mod utils;

pub use ui::ValidatorApp;
