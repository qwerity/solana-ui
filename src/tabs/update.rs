//! Update tab for checking and installing application updates.

use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::updater::{UpdateStatus, Updater, ReleaseInfo};
use crate::tabs::logs::LogStore;

pub struct UpdateTab {
    updater: Option<Updater>,
    update_status: Arc<Mutex<Option<UpdateStatus>>>,
    is_checking: bool,
    is_downloading: bool,
    download_progress: f32,
    error_message: Option<String>,
    success_message: Option<String>,
    download_status: Arc<Mutex<Option<Result<PathBuf, String>>>>,
}

impl Default for UpdateTab {
    fn default() -> Self {
        Self {
            updater: None, // Will be initialized later with log store
            update_status: Arc::new(Mutex::new(None)),
            is_checking: false,
            is_downloading: false,
            download_progress: 0.0,
            error_message: None,
            success_message: None,
            download_status: Arc::new(Mutex::new(None)),
        }
    }
}

impl UpdateTab {
    pub fn new(log_store: LogStore) -> Self {
        Self {
            updater: Some(Updater::new(log_store)),
            update_status: Arc::new(Mutex::new(None)),
            is_checking: false,
            is_downloading: false,
            download_progress: 0.0,
            error_message: None,
            success_message: None,
            download_status: Arc::new(Mutex::new(None)),
        }
    }


    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("🔄 Application Updates");
        ui.separator();
        
        // Show keyboard shortcut info
        ui.horizontal(|ui| {
            ui.label("💡 Tip:");
            ui.colored_label(
                egui::Color32::from_rgb(100, 149, 237),
                "Press Cmd+Shift+U to quickly access this tab"
            );
        });
        ui.add_space(10.0);

        // Current version info
        if let Some(ref updater) = self.updater {
            ui.horizontal(|ui| {
                ui.label("Current version:");
                ui.colored_label(egui::Color32::from_rgb(100, 149, 237), updater.current_version());
            });
        }

        ui.add_space(10.0);

        // Check for updates button
        ui.horizontal(|ui| {
            let check_button = ui.add_enabled(
                !self.is_checking && !self.is_downloading,
                egui::Button::new(if self.is_checking {
                    "🔄 Checking..."
                } else {
                    "🔍 Check for Updates"
                })
            );

            if check_button.clicked() {
                self.check_for_updates(ctx);
            }

            if self.is_checking {
                ui.spinner();
            }
        });

        ui.add_space(10.0);

        // Display update status and reset checking state when done
        let update_status = self.update_status.clone();
        if let Ok(status) = update_status.try_lock() {
            if let Some(ref status) = *status {
                // Reset checking state once we have results
                if self.is_checking {
                    self.is_checking = false;
                }
                
                match status {
                    UpdateStatus::UpToDate => {
                        ui.colored_label(
                            egui::Color32::from_rgb(34, 139, 34),
                            "✅ You're running the latest version!"
                        );
                    }
                    UpdateStatus::UpdateAvailable(release) => {
                        self.show_update_available_ui(ui, release, ctx);
                    }
                    UpdateStatus::CheckFailed(error) => {
                        ui.colored_label(
                            egui::Color32::from_rgb(220, 20, 60),
                            format!("❌ Check failed: {}", error)
                        );
                    }
                }
            }
        }

        // Show error messages
        if let Some(ref error) = self.error_message {
            ui.add_space(10.0);
            ui.colored_label(egui::Color32::from_rgb(220, 20, 60), format!("❌ {}", error));
        }

        // Show success messages
        if let Some(ref success) = self.success_message {
            ui.add_space(10.0);
            ui.colored_label(egui::Color32::from_rgb(34, 139, 34), format!("✅ {}", success));
        }

        // Check download status and update UI accordingly
        let download_status = self.download_status.clone();
        if let Ok(status) = download_status.try_lock() {
            if let Some(ref result) = *status {
                // Reset downloading state when we have a result
                if self.is_downloading {
                    self.is_downloading = false;
                }
                
                match result {
                    Ok(dmg_path) => {
                        ui.add_space(10.0);
                        ui.colored_label(
                            egui::Color32::from_rgb(34, 139, 34),
                            format!("✅ Downloaded to: {}", dmg_path.display())
                        );
                        ui.label("📂 The Downloads folder should have opened automatically");
                        ui.label("Double-click the DMG to install the update");
                    }
                    Err(error) => {
                        ui.add_space(10.0);
                        ui.colored_label(
                            egui::Color32::from_rgb(220, 20, 60),
                            format!("❌ Download failed: {}", error)
                        );
                    }
                }
            }
        }

        // Download progress
        if self.is_downloading {
            ui.add_space(10.0);
            ui.label("📥 Downloading update...");
            let progress_bar = egui::ProgressBar::new(self.download_progress)
                .show_percentage()
                .animate(true);
            ui.add(progress_bar);
            ui.label("The DMG will be saved to your Downloads folder");
        }

        ui.add_space(20.0);

        // Auto-update settings
        ui.group(|ui| {
            ui.heading("⚙️ Update Settings");
            ui.checkbox(&mut false, "Check for updates automatically on startup");
            ui.checkbox(&mut false, "Include pre-release versions");
            ui.add_space(5.0);
            ui.label("🔒 Updates are downloaded from GitHub releases and verified before installation.");
        });
    }

    fn show_update_available_ui(&mut self, ui: &mut egui::Ui, release: &ReleaseInfo, ctx: &egui::Context) {
        ui.group(|ui| {
            ui.heading("🎉 Update Available!");
            
            ui.horizontal(|ui| {
                ui.label("New version:");
                ui.colored_label(
                    egui::Color32::from_rgb(34, 139, 34),
                    &release.tag_name
                );
            });

            ui.horizontal(|ui| {
                ui.label("Release name:");
                ui.label(&release.name);
            });

            ui.horizontal(|ui| {
                ui.label("Published:");
                ui.label(&release.published_at);
            });

            if release.prerelease {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 165, 0),
                    "⚠️ This is a pre-release version"
                );
            }

            ui.add_space(10.0);

            // Release notes
            ui.label("📝 Release Notes:");
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    ui.label(&release.body);
                });

            ui.add_space(10.0);

            // Install update button
            ui.horizontal(|ui| {
                let install_button = ui.add_enabled(
                    !self.is_downloading,
                    egui::Button::new(if self.is_downloading {
                        "📥 Downloading..."
                    } else {
                        "🚀 Install Update"
                    })
                );

                if install_button.clicked() {
                    self.install_update(release.clone(), ctx);
                }

                ui.label("(The app will restart after installation)");
            });
        });
    }

    fn check_for_updates(&mut self, ctx: &egui::Context) {
        if let Some(updater) = &self.updater {
            if !self.is_checking {
                self.is_checking = true;
                self.error_message = None;
                self.success_message = None;
                
                let updater_clone = updater.clone();
                let status_clone = self.update_status.clone();
                let ctx_clone = ctx.clone();
                
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async move {
                        let status = updater_clone.check_for_updates().await;
                        {
                            let mut status_guard = status_clone.lock().await;
                            *status_guard = Some(status);
                        }
                        ctx_clone.request_repaint();
                    });
                });
            }
        }
    }

    fn install_update(&mut self, release: ReleaseInfo, ctx: &egui::Context) {
        if let Some(updater) = &self.updater {
            if !self.is_downloading {
                self.is_downloading = true;
                self.download_progress = 0.0;
                self.error_message = None;
                self.success_message = None;
                
                let updater_clone = updater.clone();
                let ctx_clone = ctx.clone();
                let download_status_clone = self.download_status.clone();
                
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async move {
                        let result = match updater_clone.download_update(&release).await {
                            Ok(dmg_path) => {
                                // Open the Downloads folder to show the DMG
                                if let Err(e) = std::process::Command::new("open")
                                    .arg("-R")
                                    .arg(&dmg_path)
                                    .spawn()
                                {
                                    eprintln!("Failed to open Downloads folder: {}", e);
                                }
                                Ok(dmg_path)
                            }
                            Err(e) => {
                                eprintln!("Download failed: {}", e);
                                Err(e.to_string())
                            }
                        };
                        
                        {
                            let mut status_guard = download_status_clone.lock().await;
                            *status_guard = Some(result);
                        }
                        ctx_clone.request_repaint();
                    });
                });
            }
        }
    }
}