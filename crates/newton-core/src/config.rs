// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Emulator configuration

use serde::{Deserialize, Serialize};
use newton_cpu::PpcModel;
use std::path::PathBuf;

/// Main emulator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorConfig {
    pub cpu: CpuConfig,
    pub memory: MemoryConfig,
    pub display: DisplayConfig,
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            cpu: CpuConfig::default(),
            memory: MemoryConfig::default(),
            display: DisplayConfig::default(),
        }
    }
}

/// CPU configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuConfig {
    /// CPU model (G3/G4/G5)
    pub model: CpuModel,
    
    /// Clock speed in MHz
    pub clock_speed: u32,
    
    /// Enable JIT compilation
    pub enable_jit: bool,
}

impl Default for CpuConfig {
    fn default() -> Self {
        Self {
            model: CpuModel::G4,
            clock_speed: 500,
            enable_jit: false,  // Start with interpreter
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CpuModel {
    G3,
    G4,
    G5,
}

impl From<CpuModel> for PpcModel {
    fn from(model: CpuModel) -> Self {
        match model {
            CpuModel::G3 => PpcModel::G3,
            CpuModel::G4 => PpcModel::G4,
            CpuModel::G5 => PpcModel::G5,
        }
    }
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// RAM size in MB
    pub ram_size_mb: usize,
    
    /// ROM file path
    pub rom_path: Option<PathBuf>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            ram_size_mb: 256,
            rom_path: None,
        }
    }
}

/// Display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Display width
    pub width: u32,
    
    /// Display height
    pub height: u32,
    
    /// Color depth (8, 16, or 32 bits)
    pub color_depth: u8,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            color_depth: 32,
        }
    }
}
