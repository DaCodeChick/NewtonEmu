// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Diagnostic device for ROM debugging
//!
//! Provides dummy hardware registers that the ROM writes to during initialization.
//! Common addresses:
//! - 0xFFFF0000-0xFFFFFFFF: High memory hardware registers

use crate::MmioDevice;
use newton_utils::Result;
use parking_lot::RwLock;

/// Diagnostic device that logs all reads/writes
pub struct DiagnosticDevice {
    name: String,
    /// Storage for register values (thread-safe with interior mutability)
    registers: RwLock<Vec<u8>>,
}

impl DiagnosticDevice {
    /// Create a new diagnostic device
    pub fn new(name: impl Into<String>, size: usize) -> Self {
        Self {
            name: name.into(),
            registers: RwLock::new(vec![0; size]),
        }
    }
}

impl MmioDevice for DiagnosticDevice {
    fn name(&self) -> &str {
        &self.name
    }

    fn read(&self, offset: u32, size: u8) -> Result<u32> {
        let registers = self.registers.read();
        let offset = offset as usize;
        let value = match (size, offset) {
            (1, _) => registers.get(offset).copied().unwrap_or(0) as u32,
            (2, o) if o + 1 < registers.len() => {
                u16::from_be_bytes([
                    registers[o],
                    registers[o + 1],
                ]) as u32
            }
            (4, o) if o + 3 < registers.len() => {
                u32::from_be_bytes([
                    registers[o],
                    registers[o + 1],
                    registers[o + 2],
                    registers[o + 3],
                ])
            }
            _ => 0,
        };

        tracing::trace!("{}: Read  offset=0x{:04X} size={} value=0x{:08X}", 
            self.name, offset, size, value);
        Ok(value)
    }

    fn write(&self, offset: u32, size: u8, value: u32) -> Result<()> {
        tracing::debug!("{}: Write offset=0x{:04X} size={} value=0x{:08X}", 
            self.name, offset, size, value);
        
        let mut registers = self.registers.write();
        let offset = offset as usize;
        match size {
            1 => {
                if offset < registers.len() {
                    registers[offset] = value as u8;
                }
            }
            2 => {
                if offset + 1 < registers.len() {
                    let bytes = (value as u16).to_be_bytes();
                    registers[offset] = bytes[0];
                    registers[offset + 1] = bytes[1];
                }
            }
            4 => {
                if offset + 3 < registers.len() {
                    let bytes = value.to_be_bytes();
                    registers[offset] = bytes[0];
                    registers[offset + 1] = bytes[1];
                    registers[offset + 2] = bytes[2];
                    registers[offset + 3] = bytes[3];
                }
            }
            _ => {}
        }

        Ok(())
    }
}
