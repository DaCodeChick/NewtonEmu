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

/// ROM type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RomType {
    /// OldWorld ROM (pre-iMac, typically 4MB)
    OldWorld,
    /// NewWorld ROM (iMac and later, with CHRP boot script)
    NewWorld,
}

/// Mac ROM image
pub struct Rom {
    data: Vec<u8>,
    base_address: u32,
    rom_type: RomType,
    entry_offset: usize,
}

impl Rom {
    /// Load ROM from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = fs::read(path.as_ref())
            .map_err(|e| Error::Io(e))?;
        
        // Detect ROM type by checking for CHRP boot script
        let rom_type = if data.len() > 20 && &data[0..11] == b"<CHRP-BOOT>" {
            RomType::NewWorld
        } else {
            RomType::OldWorld
        };
        
        // Typical Mac ROM is 4MB and loads at 0xFFC00000 or 0xFFF00000
        // Larger ROMs (>1MB) use 0xFFC00000, smaller use 0xFFF00000
        let base_address = if data.len() <= 1024 * 1024 {
            0xFFF0_0000
        } else {
            0xFFC0_0000
        };
        
        // Find entry point for NewWorld ROMs
        let entry_offset = if rom_type == RomType::NewWorld {
            Self::find_newworld_entry(&data)
        } else {
            0  // OldWorld ROMs start at beginning
        };

        tracing::info!(
            "Loaded ROM: {} bytes ({:?}) from {}, entry offset: 0x{:X}",
            data.len(),
            rom_type,
            path.as_ref().display(),
            entry_offset
        );

        Ok(Self {
            data,
            base_address,
            rom_type,
            entry_offset,
        })
    }
    
    /// Find the entry point in a NewWorld ROM by locating the end of the CHRP boot script
    fn find_newworld_entry(data: &[u8]) -> usize {
        // NewWorld ROMs typically have:
        // - CHRP boot script at start
        // - ELF or other headers around 0x4000
        // - Actual PowerPC code around 0x4100
        //
        // For now, use a simple heuristic: look for PowerPC code signature
        // The first instruction is often mflr r0 (0x7C0802A6) or similar
        
        // Common entry points to try
        let candidates = [0x4100, 0x4000, 0x3800, 0x3000];
        
        for &offset in &candidates {
            if offset + 16 <= data.len() {
                // Check if this looks like PowerPC code
                let word = BigEndian::read_u32(&data[offset..]);
                
                // Look for common PowerPC instruction patterns:
                // - mflr (0x7C08xxxx range)
                // - li/lis (0x3xxxxxxx range)
                // - stwu (0x94xxxxxx range)
                // - or anything non-zero that's not ELF magic
                if word != 0 && word != 0x7F454C46 && ((word & 0xFC000000) == 0x7C000000 || 
                   (word & 0xFC000000) == 0x94000000 || (word & 0xF0000000) == 0x30000000) {
                    tracing::debug!("Found PowerPC code at offset 0x{:X}, first instruction: 0x{:08X}", offset, word);
                    return offset;
                }
            }
        }
        
        // Fallback
        tracing::warn!("Could not find PowerPC code start, using default entry offset 0x4100");
        0x4100
    }

    /// Create ROM from raw data
    pub fn from_data(data: Vec<u8>, base_address: u32) -> Self {
        let rom_type = if data.len() > 20 && &data[0..11] == b"<CHRP-BOOT>" {
            RomType::NewWorld
        } else {
            RomType::OldWorld
        };
        
        let entry_offset = if rom_type == RomType::NewWorld {
            Self::find_newworld_entry(&data)
        } else {
            0
        };
        
        Self { data, base_address, rom_type, entry_offset }
    }

    /// Get ROM base address
    pub const fn base_address(&self) -> u32 {
        self.base_address
    }
    
    /// Get ROM entry point address (base + entry_offset)
    pub fn entry_address(&self) -> u32 {
        self.base_address + self.entry_offset as u32
    }

    /// Get ROM size
    pub const fn size(&self) -> usize {
        self.data.len()
    }
    
    /// Get ROM type
    pub const fn rom_type(&self) -> RomType {
        self.rom_type
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
        // For larger ROMs (>1MB), they start at 0xFFC00000
        // For smaller ROMs (<=1MB), they start at 0xFFF00000
        //
        // NewWorld ROMs are NOT mirrored - they only exist at their base address.
        // Accesses outside the ROM range should return None.
        
        if addr >= self.base_address {
            let offset = (addr - self.base_address) as usize;
            if offset < self.data.len() {
                Some(offset)
            } else {
                None
            }
        } else {
            None
        }
    }
}
