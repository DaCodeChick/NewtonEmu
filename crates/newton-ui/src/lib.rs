// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! User interface with separate display and debug windows
//! 
//! Architecture:
//! - Display window: Pure wgpu rendering for emulated Mac display (low latency)
//! - Debug window: egui-based debugging and configuration tools (separate window)

// pub mod app;  // Using main.rs directly for now
pub mod display;
pub mod debug;
pub mod debugger;
pub mod renderer;
pub mod launcher;

// pub use app::EmulatorApp;
pub use display::DisplayWindow;
pub use debug::DebugWindow;
pub use renderer::WgpuRenderer;
pub use launcher::{LauncherWindow, LauncherConfig};
