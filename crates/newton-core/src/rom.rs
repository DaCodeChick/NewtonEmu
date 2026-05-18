// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! ROM loading and management

use newton_utils::{Error, Result};
use byteorder::{BigEndian, ByteOrder};
use std::fs;
use std::path::Path;

/// Mac ROM image
pub struct Rom {
    data: Vec<u8>,
    base_address: u32,
}

impl Rom {
    /// Load ROM from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = fs::read(path.as_ref())
            .map_err(|e| Error::Io(e))?;
        
        // Typical Mac ROM is 4MB and loads at 0xFFC00000 or 0xFFF00000
        // We'll use 0xFFF00000 for now (1MB ROM space)
        let base_address = if data.len() <= 1024 * 1024 {
            0xFFF0_0000
        } else {
            0xFFC0_0000
        };

        tracing::info!(
            "Loaded ROM: {} bytes from {}",
            data.len(),
            path.as_ref().display()
        );

        Ok(Self {
            data,
            base_address,
        })
    }

    /// Create ROM from raw data
    pub fn from_data(data: Vec<u8>, base_address: u32) -> Self {
        Self { data, base_address }
    }

    /// Get ROM base address
    pub fn base_address(&self) -> u32 {
        self.base_address
    }

    /// Get ROM size
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Read a byte from ROM
    pub fn read_u8(&self, offset: usize) -> u8 {
        self.data.get(offset).copied().unwrap_or(0)
    }

    /// Read a 32-bit word from ROM (big-endian)
    pub fn read_u32(&self, offset: usize) -> u32 {
        if offset + 4 <= self.data.len() {
            BigEndian::read_u32(&self.data[offset..])
        } else {
            0
        }
    }

    /// Get raw ROM data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Check if an address is within ROM space and return the offset
    /// Handles ROM mirroring in the top 4MB of address space
    pub fn address_to_offset(&self, addr: u32) -> Option<usize> {
        // ROM is typically mapped in the top of memory (0xFFC00000-0xFFFFFFFF)
        // For larger ROMs (>1MB), they're at 0xFFC00000
        // For smaller ROMs (<=1MB), they're at 0xFFF00000
        // But they are often mirrored/aliased in the 0xFFC00000-0xFFFFFFFF range
        
        if addr >= 0xFFC0_0000 {
            // Calculate offset within the top 4MB
            let offset_in_top_4mb = (addr - 0xFFC0_0000) as usize;
            
            // Mirror/wrap based on ROM size
            let rom_offset = offset_in_top_4mb % self.data.len();
            
            Some(rom_offset)
        } else {
            None
        }
    }
}
