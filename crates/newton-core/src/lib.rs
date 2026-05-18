// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Core emulator functionality
//!
//! This crate ties together the CPU, memory, and devices into a complete emulator.

pub mod config;
pub mod cpu_thread;
pub mod emulator;
pub mod memory;
pub mod rom;

pub use config::EmulatorConfig;
pub use cpu_thread::{CpuThread, CpuCommand, CpuEvent, CpuState, CpuStateSnapshot};
pub use emulator::Emulator;
pub use memory::Memory;
