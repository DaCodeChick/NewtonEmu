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
pub mod debugger;
pub mod debugger_protocol;
pub mod emulator;
pub mod memory;
pub mod rom;
pub mod openfirmware;
pub mod elf;
pub mod lzss;

pub use config::EmulatorConfig;
pub use cpu_thread::{CpuThread, CpuCommand, CpuEvent, CpuState, CpuStateSnapshot};
pub use debugger::{Debugger, Breakpoint, BreakpointType, ExecutionState, StackFrame};
pub use debugger_protocol::{DebuggerCommand, DebuggerResponse, CpuStateDto, BreakpointDto, MemoryDto, DisassemblyDto};
pub use emulator::{Emulator, EmulatorMode};
pub use memory::Memory;
pub use openfirmware::OpenFirmware;
pub use rom::{Rom, RomType};
pub use elf::ElfFile;
