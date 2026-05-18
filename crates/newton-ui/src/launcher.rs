// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Launcher window - Configuration and ROM selection UI

use newton_core::EmulatorConfig;
use std::path::PathBuf;

/// Launcher configuration state
#[derive(Debug, Clone)]
pub struct LauncherConfig {
    pub rom_path: Option<PathBuf>,
    pub ram_mb: usize,
    pub display_width: u32,
    pub display_height: u32,
    pub color_depth: u32,
    pub enable_debug_window: bool,
    pub start_paused: bool,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            rom_path: None,
            ram_mb: 256,
            display_width: 800,
            display_height: 600,
            color_depth: 32,
            enable_debug_window: false,
            start_paused: false,
        }
    }
}

impl LauncherConfig {
    /// Convert launcher config to emulator config
    pub fn to_emulator_config(&self) -> Option<EmulatorConfig> {
        let rom_path = self.rom_path.as_ref()?;
        
        let mut config = EmulatorConfig::default();
        config.memory.ram_size_mb = self.ram_mb;
        config.memory.rom_path = Some(rom_path.to_string_lossy().to_string());
        config.display.width = self.display_width;
        config.display.height = self.display_height;
        config.display.color_depth = self.color_depth as u8;
        
        Some(config)
    }
}

/// Launcher window state
pub struct LauncherWindow {
    config: LauncherConfig,
    should_launch: bool,
    rom_file_dialog: Option<String>,
    recent_roms: Vec<PathBuf>,
}

impl LauncherWindow {
    /// Create a new launcher window
    pub fn new() -> Self {
        Self {
            config: LauncherConfig::default(),
            should_launch: false,
            rom_file_dialog: None,
            recent_roms: Self::load_recent_roms(),
        }
    }

    /// Load recent ROMs from config file
    fn load_recent_roms() -> Vec<PathBuf> {
        // TODO: Load from config file
        // For now, scan roms/ directory if it exists
        let mut roms = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir("roms") {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "rom" {
                        roms.push(entry.path());
                    }
                }
            }
        }
        
        roms.sort();
        roms
    }

    /// Update the launcher UI
    /// Call this from within an egui context, e.g.:
    /// ```
    /// ctx.run_ui(input, |ui| {
    ///     launcher.update(ui);
    /// });
    /// ```
    pub fn update(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("NewtonEmu - PowerPC Macintosh Emulator");
            ui.separator();
            
            // ROM Selection
            ui.group(|ui| {
                ui.heading("ROM File");
                
                ui.horizontal(|ui| {
                    if ui.button("📁 Browse...").clicked() {
                        // TODO: Open file dialog
                        // For now, use a text field
                        self.rom_file_dialog = Some(String::new());
                    }
                    
                    if let Some(ref path) = self.config.rom_path {
                        ui.label(format!("Selected: {}", path.display()));
                    } else {
                        ui.label("No ROM selected");
                    }
                });
                
                if !self.recent_roms.is_empty() {
                    ui.separator();
                    ui.label("Recent ROMs:");
                    
                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for rom in &self.recent_roms {
                                let name = rom.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Unknown");
                                
                                if ui.selectable_label(
                                    self.config.rom_path.as_ref() == Some(rom),
                                    name
                                ).clicked() {
                                    self.config.rom_path = Some(rom.clone());
                                }
                            }
                        });
                }
            });
            
            ui.add_space(10.0);
            
            // Memory Configuration
            ui.group(|ui| {
                ui.heading("Memory");
                
                ui.horizontal(|ui| {
                    ui.label("RAM Size:");
                    ui.add(egui::Slider::new(&mut self.config.ram_mb, 64..=2048)
                        .suffix(" MB")
                        .step_by(64.0));
                });
                
                // Common presets
                ui.horizontal(|ui| {
                    ui.label("Presets:");
                    if ui.button("128 MB").clicked() {
                        self.config.ram_mb = 128;
                    }
                    if ui.button("256 MB").clicked() {
                        self.config.ram_mb = 256;
                    }
                    if ui.button("512 MB").clicked() {
                        self.config.ram_mb = 512;
                    }
                    if ui.button("1 GB").clicked() {
                        self.config.ram_mb = 1024;
                    }
                });
            });
            
            ui.add_space(10.0);
            
            // Display Configuration
            ui.group(|ui| {
                ui.heading("Display");
                
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.add(egui::DragValue::new(&mut self.config.display_width)
                        .speed(1)
                        .range(320..=3840));
                    
                    ui.label("Height:");
                    ui.add(egui::DragValue::new(&mut self.config.display_height)
                        .speed(1)
                        .range(240..=2160));
                });
                
                // Common resolutions
                ui.horizontal(|ui| {
                    ui.label("Presets:");
                    if ui.button("640×480").clicked() {
                        self.config.display_width = 640;
                        self.config.display_height = 480;
                    }
                    if ui.button("800×600").clicked() {
                        self.config.display_width = 800;
                        self.config.display_height = 600;
                    }
                    if ui.button("1024×768").clicked() {
                        self.config.display_width = 1024;
                        self.config.display_height = 768;
                    }
                    if ui.button("1280×1024").clicked() {
                        self.config.display_width = 1280;
                        self.config.display_height = 1024;
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Color Depth:");
                    egui::ComboBox::from_id_salt("color_depth")
                        .selected_text(format!("{} bit", self.config.color_depth))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.config.color_depth, 8, "8 bit (256 colors)");
                            ui.selectable_value(&mut self.config.color_depth, 16, "16 bit (Thousands)");
                            ui.selectable_value(&mut self.config.color_depth, 32, "32 bit (Millions)");
                        });
                });
            });
            
            ui.add_space(10.0);
            
            // Debug Options
            ui.group(|ui| {
                ui.heading("Debug Options");
                
                ui.checkbox(&mut self.config.enable_debug_window, "Show debug window on startup");
                ui.checkbox(&mut self.config.start_paused, "Start emulation paused");
            });
            
            ui.add_space(20.0);
            
            // Launch Button
            ui.vertical_centered(|ui| {
                let can_launch = self.config.rom_path.is_some();
                
                ui.add_enabled_ui(can_launch, |ui| {
                    if ui.button(egui::RichText::new("🚀 Launch Emulator")
                        .size(24.0)
                        .strong())
                        .clicked() 
                    {
                        self.should_launch = true;
                    }
                });
                
                if !can_launch {
                    ui.label(egui::RichText::new("Please select a ROM file")
                        .color(egui::Color32::RED));
                }
            });
            
            ui.add_space(10.0);
            
            // Footer
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("NewtonEmu v0.1.0");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to("Documentation", "https://github.com/YourUsername/NewtonEmu");
                });
            });
        });
    }

    /// Check if user clicked launch
    pub fn should_launch(&self) -> bool {
        self.should_launch
    }

    /// Get the configuration
    pub fn config(&self) -> &LauncherConfig {
        &self.config
    }

    /// Reset launch state (call after launching)
    pub fn reset_launch(&mut self) {
        self.should_launch = false;
    }
}

impl Default for LauncherWindow {
    fn default() -> Self {
        Self::new()
    }
}
