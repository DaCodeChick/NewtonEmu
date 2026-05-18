// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Debug window - egui-based debugging and configuration interface

use newton_core::Emulator;

/// Debug/config window with egui
pub struct DebugWindow {
    show_registers: bool,
    show_memory: bool,
    show_disassembly: bool,
    show_config: bool,
    memory_address: u32,
    disasm_address: u32,
}

impl DebugWindow {
    /// Create a new debug window
    pub fn new() -> Self {
        Self {
            show_registers: true,
            show_memory: false,
            show_disassembly: false,
            show_config: false,
            memory_address: 0,
            disasm_address: 0,
        }
    }

    /// Update the debug window UI
    pub fn update(&mut self, ctx: &egui::Context, emulator: &mut Emulator) {
        // Main debug control panel
        egui::Window::new("Debug Controls")
            .default_pos([10.0, 10.0])
            .default_size([300.0, 400.0])
            .show(ctx, |ui| {
                ui.heading("NewtonEmu Debugger");
                ui.separator();
                
                // Emulation control
                ui.group(|ui| {
                    ui.label("Emulation Control");
                    ui.horizontal(|ui| {
                        if ui.button("▶ Start").clicked() {
                            emulator.start();
                        }
                        if ui.button("⏸ Pause").clicked() {
                            emulator.stop();
                        }
                        if ui.button("⟳ Reset").clicked() {
                            emulator.reset();
                        }
                    });
                    
                    if ui.button("⏭ Step").clicked() {
                        let _ = emulator.step();
                    }
                    
                    ui.horizontal(|ui| {
                        ui.label("Status:");
                        if emulator.is_running() {
                            ui.colored_label(egui::Color32::GREEN, "Running");
                        } else {
                            ui.colored_label(egui::Color32::RED, "Stopped");
                        }
                    });
                });
                
                ui.separator();
                
                // Window toggles
                ui.group(|ui| {
                    ui.label("Debug Windows");
                    ui.checkbox(&mut self.show_registers, "Registers");
                    ui.checkbox(&mut self.show_memory, "Memory Viewer");
                    ui.checkbox(&mut self.show_disassembly, "Disassembly");
                    ui.checkbox(&mut self.show_config, "Configuration");
                });
                
                ui.separator();
                
                // Quick status
                ui.group(|ui| {
                    ui.label("Quick Status");
                    if let Some(regs) = emulator.registers() {
                        ui.monospace(format!("PC:  0x{:08X}", regs.pc));
                        ui.monospace(format!("LR:  0x{:08X}", regs.lr));
                        ui.monospace(format!("CTR: 0x{:08X}", regs.ctr));
                        ui.monospace(format!("CR:  0x{:08X}", regs.cr.bits()));
                    } else {
                        ui.label("CPU not available (multi-threaded mode)");
                    }
                });
            });

        // Registers window
        if self.show_registers {
            self.show_registers_window(ctx, emulator);
        }

        // Memory viewer window
        if self.show_memory {
            self.show_memory_window(ctx, emulator);
        }

        // Disassembly window
        if self.show_disassembly {
            self.show_disassembly_window(ctx, emulator);
        }

        // Configuration window
        if self.show_config {
            self.show_config_window(ctx, emulator);
        }
    }

    fn show_registers_window(&mut self, ctx: &egui::Context, emulator: &Emulator) {
        egui::Window::new("Registers")
            .open(&mut self.show_registers)
            .default_size([400.0, 600.0])
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(regs) = emulator.registers() {
                        ui.heading("General Purpose Registers");
                        egui::Grid::new("gpr_grid").num_columns(2).striped(true).show(ui, |ui| {
                            for i in 0..32 {
                                ui.monospace(format!("r{:2}", i));
                                ui.monospace(format!("0x{:08X} ({})", 
                                    regs.gpr[i], regs.gpr[i] as i32));
                                ui.end_row();
                            }
                        });
                        
                        ui.separator();
                        ui.heading("Special Registers");
                        egui::Grid::new("spr_grid").num_columns(2).striped(true).show(ui, |ui| {
                            ui.label("PC");
                            ui.monospace(format!("0x{:08X}", regs.pc));
                            ui.end_row();
                            
                            ui.label("LR");
                            ui.monospace(format!("0x{:08X}", regs.lr));
                        ui.end_row();
                        
                        ui.label("CTR");
                        ui.monospace(format!("0x{:08X}", regs.ctr));
                        ui.end_row();
                        
                        ui.label("CR");
                        ui.monospace(format!("0x{:08X}", regs.cr.bits()));
                        ui.end_row();
                        
                        ui.label("XER");
                        ui.monospace(format!("0x{:08X}", regs.xer.bits()));
                        ui.end_row();
                        
                        ui.label("MSR");
                        ui.monospace(format!("0x{:08X}", regs.msr.bits()));
                        ui.end_row();
                    });
                } else {
                    ui.label("CPU not available in multi-threaded mode");
                    ui.label("Use send_cpu_command(CpuCommand::GetState) to get state");
                }
            });
        });
    }

    fn show_memory_window(&mut self, ctx: &egui::Context, _emulator: &Emulator) {
        egui::Window::new("Memory Viewer")
            .open(&mut self.show_memory)
            .default_size([600.0, 400.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Address:");
                    ui.add(egui::DragValue::new(&mut self.memory_address)
                        .hexadecimal(8, false, true)
                        .speed(16));
                    if ui.button("Go").clicked() {
                        // Snap to 16-byte boundary
                        self.memory_address &= !0xF;
                    }
                });
                
                ui.separator();
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.monospace("Memory view not yet implemented");
                    // TODO: Implement hex dump view
                    // Format: ADDR: 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F  ................
                });
            });
    }

    fn show_disassembly_window(&mut self, ctx: &egui::Context, emulator: &Emulator) {
        egui::Window::new("Disassembly")
            .open(&mut self.show_disassembly)
            .default_size([600.0, 400.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Address:");
                    ui.add(egui::DragValue::new(&mut self.disasm_address)
                        .hexadecimal(8, false, true)
                        .speed(4));
                    if ui.button("Go to PC").clicked() {
                        if let Some(regs) = emulator.registers() {
                            self.disasm_address = regs.pc;
                        }
                    }
                });
                
                ui.separator();
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.monospace("Disassembly not yet implemented");
                    // TODO: Disassemble instructions from memory
                    // Show ~20 instructions around the address
                    // Highlight PC location
                });
            });
    }

    fn show_config_window(&mut self, ctx: &egui::Context, _emulator: &Emulator) {
        egui::Window::new("Configuration")
            .open(&mut self.show_config)
            .default_size([400.0, 300.0])
            .show(ctx, |ui| {
                ui.heading("Emulator Configuration");
                ui.separator();
                
                ui.group(|ui| {
                    ui.label("CPU Settings");
                    ui.label("• Clock speed: 400 MHz (G4)");
                    ui.label("• Execution mode: Interpreter");
                    // TODO: Add configuration options
                });
                
                ui.separator();
                
                ui.group(|ui| {
                    ui.label("Memory Settings");
                    ui.label("• RAM: 128 MB");
                    ui.label("• ROM: Loaded from file");
                });
                
                ui.separator();
                
                ui.group(|ui| {
                    ui.label("Display Settings");
                    ui.label("• Resolution: 800x600");
                    ui.label("• Color depth: 32-bit");
                });
            });
    }
}

impl Default for DebugWindow {
    fn default() -> Self {
        Self::new()
    }
}
