// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Main application UI

use newton_core::{Emulator, EmulatorConfig};
use newton_utils::Result;
use crate::debugger::DebuggerWindow;

/// Main emulator application
pub struct EmulatorApp {
    emulator: Emulator,
    debugger: DebuggerWindow,
    show_debugger: bool,
    show_config: bool,
}

impl EmulatorApp {
    /// Create a new emulator application
    pub fn new(config: EmulatorConfig) -> Result<Self> {
        let emulator = Emulator::new(config)?;
        
        Ok(Self {
            emulator,
            debugger: DebuggerWindow::new(),
            show_debugger: false,
            show_config: false,
        })
    }

    /// Update the UI
    pub fn update(&mut self, ctx: &egui::Context) {
        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load ROM...").clicked() {
                        // TODO: File dialog
                    }
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
                
                ui.menu_button("Emulation", |ui| {
                    if ui.button("Start").clicked() {
                        self.emulator.start();
                    }
                    if ui.button("Stop").clicked() {
                        self.emulator.stop();
                    }
                    if ui.button("Reset").clicked() {
                        self.emulator.reset();
                    }
                    ui.separator();
                    if ui.button("Step").clicked() {
                        let _ = self.emulator.step();
                    }
                });
                
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_debugger, "Debugger");
                    ui.checkbox(&mut self.show_config, "Configuration");
                });
            });
        });

        // Main display area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("NewtonEmu - PowerPC Emulator");
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Status:");
                if self.emulator.is_running() {
                    ui.colored_label(egui::Color32::GREEN, "Running");
                } else {
                    ui.colored_label(egui::Color32::RED, "Stopped");
                }
            });
            
            ui.horizontal(|ui| {
                ui.label(format!("PC: 0x{:08X}", self.emulator.cpu().registers.pc));
            });
            
            ui.separator();
            ui.label("Display output will appear here");
        });

        // Debugger window
        if self.show_debugger {
            self.debugger.show(ctx, &mut self.emulator, &mut self.show_debugger);
        }

        // Configuration window
        if self.show_config {
            egui::Window::new("Configuration")
                .open(&mut self.show_config)
                .show(ctx, |ui| {
                    ui.label("Configuration options will appear here");
                });
        }
    }

    /// Get emulator reference
    pub fn emulator(&self) -> &Emulator {
        &self.emulator
    }

    /// Get mutable emulator reference
    pub fn emulator_mut(&mut self) -> &mut Emulator {
        &mut self.emulator
    }
}
