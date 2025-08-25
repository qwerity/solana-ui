//! Solana Validators UI - Main application entry point.
//!
//! A GUI application for monitoring Solana validators, network nodes, and analyzing
//! voting activity across different slots and clusters.

mod config;
mod constants;
mod solana;
mod tabs;
mod ui;
mod updater;
mod utils;

use ui::ValidatorApp;

/// Application configuration constants.
mod app_config {
    pub const WINDOW_SIZE: [f32; 2] = [1800.0, 1000.0];
    pub const WINDOW_TITLE: &str = "Solana UI";
    pub const APP_NAME: &str = "solana-ui";
}

/// Load the official Solana logo.
fn load_solana_icon() -> Option<egui::IconData> {
    let svg_data = include_bytes!("../solana-logo.svg");

    // Parse SVG and render to PNG at 32x32
    match resvg::usvg::Tree::from_data(svg_data, &resvg::usvg::Options::default()) {
        Ok(tree) => {
            let size = 512;
            let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size)?;

            // Calculate scale to fit the logo in 32x32
            let svg_size = tree.size();
            let scale_x = size as f32 / svg_size.width();
            let scale_y = size as f32 / svg_size.height();
            let scale = scale_x.min(scale_y);

            let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
            resvg::render(&tree, transform, &mut pixmap.as_mut());

            // Convert to RGBA format
            let rgba_data: Vec<u8> = pixmap
                .pixels()
                .iter()
                .flat_map(|pixel| [pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()])
                .collect();

            Some(egui::IconData {
                rgba: rgba_data,
                width: size,
                height: size,
            })
        }
        Err(e) => {
            eprintln!("Failed to parse Solana SVG logo: {}", e);
            None
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    // Load the configuration to get window settings
    let config_manager = config::ConfigManager::new();

    // Load the official Solana icon
    let icon = load_solana_icon();

    // Get window settings from config or use defaults
    let window_size = config_manager
        .get_window_size()
        .unwrap_or(app_config::WINDOW_SIZE.into());
    let window_position = config_manager.get_window_position();

    // Configure native window options with saved geometry
    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([window_size.0, window_size.1])
        .with_title(app_config::WINDOW_TITLE);

    // Set position if available
    if let Some(pos) = window_position {
        viewport_builder = viewport_builder.with_position([pos.0, pos.1]);
    }

    let options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };

    // Add icon if successfully loaded
    let options = if let Some(icon_data) = icon {
        eframe::NativeOptions {
            viewport: options.viewport.with_icon(icon_data),
            ..options
        }
    } else {
        options
    };

    // Launch the application
    eframe::run_native(
        app_config::APP_NAME,
        options,
        Box::new(|cc| Ok(Box::new(ValidatorApp::new(cc)))),
    )
}
