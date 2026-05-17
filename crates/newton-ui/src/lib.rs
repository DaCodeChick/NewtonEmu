// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! User interface using egui and wgpu

pub mod app;
pub mod debugger;
pub mod renderer;

pub use app::EmulatorApp;
pub use renderer::WgpuRenderer;
