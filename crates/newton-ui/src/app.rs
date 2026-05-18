// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Main application - Coordinates emulation, display, and debug windows

use newton_core::{Emulator, EmulatorConfig};
use newton_utils::Result;
use crate::display::DisplayWindow;
use crate::debug::DebugWindow;

/// Main emulator application
/// 
/// Manages two separate windows:
/// - Display window: Pure wgpu rendering of emulated Mac display
/// - Debug window: egui-based debugging and configuration tools
pub struct EmulatorApp {
    emulator: Emulator,
    display_window: DisplayWindow,
    debug_window: DebugWindow,
    show_debug_window: bool,
}

impl EmulatorApp {
    /// Create a new emulator application
    pub fn new(config: EmulatorConfig) -> Result<Self> {
        let emulator = Emulator::new(config)?;
        let display_window = DisplayWindow::new(800, 600);
        let debug_window = DebugWindow::new();
        
        Ok(Self {
            emulator,
            display_window,
            debug_window,
            show_debug_window: true, // Show debug window by default
        })
    }

    /// Update the emulator and render display
    pub fn update(&mut self) -> Result<()> {
        // Run emulator if it's running
        if self.emulator.is_running() {
            // TODO: Run for a time slice
            self.emulator.step()?;
        }
        
        // Render display window
        self.display_window.render(&self.emulator)?;
        
        Ok(())
    }

    /// Update debug window UI
    pub fn update_debug_ui(&mut self, ctx: &egui::Context) {
        if self.show_debug_window {
            self.debug_window.update(ctx, &mut self.emulator);
        }
    }

    /// Toggle debug window visibility
    pub fn toggle_debug_window(&mut self) {
        self.show_debug_window = !self.show_debug_window;
    }

    /// Check if debug window is visible
    pub fn is_debug_window_visible(&self) -> bool {
        self.show_debug_window
    }

    /// Get emulator reference
    pub fn emulator(&self) -> &Emulator {
        &self.emulator
    }

    /// Get mutable emulator reference
    pub fn emulator_mut(&mut self) -> &mut Emulator {
        &mut self.emulator
    }

    /// Get display window reference
    pub fn display_window(&self) -> &DisplayWindow {
        &self.display_window
    }

    /// Get mutable display window reference
    pub fn display_window_mut(&mut self) -> &mut DisplayWindow {
        &mut self.display_window
    }
}
