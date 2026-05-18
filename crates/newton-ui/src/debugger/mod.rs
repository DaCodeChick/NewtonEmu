// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Debugger UI components

pub mod register_view;
pub mod memory_view;
pub mod disassembly_view;

use newton_core::Emulator;

/// Main debugger window
pub struct DebuggerWindow {
    // Debugger state
}

impl DebuggerWindow {
    /// Create a new debugger window
    pub fn new() -> Self {
        Self {}
    }

    /// Show the debugger window
    pub fn show(&mut self, ctx: &egui::Context, emulator: &mut Emulator, open: &mut bool) {
        egui::Window::new("Debugger")
            .open(open)
            .default_width(600.0)
            .show(ctx, |ui| {
                ui.heading("CPU Debugger");
                ui.separator();
                
                // Registers
                ui.collapsing("Registers", |ui| {
                    if let Some(regs) = emulator.registers() {
                        egui::Grid::new("registers_grid")
                            .num_columns(4)
                            .show(ui, |ui| {
                                for i in 0..32 {
                                    ui.label(format!("r{}", i));
                                    ui.label(format!("0x{:08X}", regs.gpr[i]));
                                    if (i + 1) % 4 == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                        
                        ui.separator();
                        ui.label(format!("PC:  0x{:08X}", regs.pc));
                        ui.label(format!("LR:  0x{:08X}", regs.lr));
                        ui.label(format!("CTR: 0x{:08X}", regs.ctr));
                    } else {
                        ui.label("CPU not available in multi-threaded mode");
                    }
                });
                
                // Memory viewer
                ui.collapsing("Memory", |ui| {
                    ui.label("Memory viewer will appear here");
                });
                
                // Disassembly
                ui.collapsing("Disassembly", |ui| {
                    ui.label("Disassembly will appear here");
                });
                
                // Control buttons
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Step").clicked() {
                        let _ = emulator.step();
                    }
                    if ui.button("Continue").clicked() {
                        emulator.start();
                    }
                    if ui.button("Stop").clicked() {
                        emulator.stop();
                    }
                });
            });
    }
}

impl Default for DebuggerWindow {
    fn default() -> Self {
        Self::new()
    }
}
