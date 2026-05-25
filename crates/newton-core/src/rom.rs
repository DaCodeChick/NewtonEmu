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
        let raw_data = fs::read(path.as_ref())
            .map_err(|e| Error::Io(e))?;
        
        // Detect ROM type by checking for CHRP boot script
        let rom_type = if raw_data.len() > 20 && &raw_data[0..11] == b"<CHRP-BOOT>" {
            RomType::NewWorld
        } else {
            RomType::OldWorld
        };
        
        // For NewWorld ROMs, keep the RAW data (don't decompress)
        // The PowerPC boot code will handle decompression itself
        let (data, base_address, entry_offset) = if rom_type == RomType::NewWorld {
            // NewWorld ROMs should be loaded as-is (raw file data)
            // The PPC boot code starts at the ELF offset
            
            // Find the ELF offset from the boot script
            let elf_offset = Self::find_elf_offset(&raw_data).unwrap_or(0x4100);
            
            tracing::info!(
                "NewWorld ROM: Keeping raw ROM ({} bytes), ELF offset: 0x{:X}",
                raw_data.len(),
                elf_offset
            );
            
            // NewWorld ROMs are loaded at a lower address to accommodate the full file
            // We'll map them starting at 0xFFF00000 - rom_size, aligned
            // This ensures the ROM is accessible and the ELF code can run
            let rom_size = raw_data.len() as u32;
            let base = 0xFFFFFFFF - rom_size + 1;
            
            (raw_data, base, elf_offset)
        } else {
            // OldWorld ROMs: Keep as before
            // Typical Mac ROM is 4MB and loads at 0xFFC00000 or 0xFFF00000
            let base_address = if raw_data.len() <= 1024 * 1024 {
                0xFFF0_0000
            } else {
                0xFFC0_0000
            };
            
            (raw_data, base_address, 0)
        };

        tracing::info!(
            "Loaded ROM: {} bytes ({:?}) from {}, base: 0x{:08X}, entry offset: 0x{:X}",
            data.len(),
            rom_type,
            path.as_ref().display(),
            base_address,
            entry_offset
        );

        Ok(Self {
            data,
            base_address,
            rom_type,
            entry_offset,
        })
    }
    
    /// Decode a NewWorld ROM image
    /// NewWorld ROMs contain a CHRP boot script followed by LZSS or parcel-compressed ROM data
    fn decode_newworld_rom(data: &[u8]) -> Result<Vec<u8>> {
        // Look for lzss-offset and lzss-size in the boot script
        let data_str = String::from_utf8_lossy(data);
        
        let lzss_offset = Self::find_boot_constant(&data_str, "lzss-offset")
            .or_else(|| Self::find_boot_constant(&data_str, "parcels-offset"))
            .ok_or_else(|| Error::Other("Could not find lzss-offset in NewWorld ROM".into()))?;
        
        let lzss_size = Self::find_boot_constant(&data_str, "lzss-size")
            .or_else(|| Self::find_boot_constant(&data_str, "parcels-size"))
            .ok_or_else(|| Error::Other("Could not find lzss-size in NewWorld ROM".into()))?;
        
        tracing::info!("NewWorld ROM: lzss-offset=0x{:X}, lzss-size=0x{:X}", lzss_offset, lzss_size);
        
        if lzss_offset + lzss_size > data.len() {
            return Err(Error::Other("Invalid LZSS offset/size in ROM".into()));
        }
        
        // Check for parcels signature
        if lzss_offset + 4 <= data.len() {
            let sig = &data[lzss_offset..lzss_offset + 4];
            if sig == b"prcl" {
                tracing::info!("ROM uses parcels format - decoding parcels");
                return Self::decode_parcels(&data[lzss_offset..lzss_offset + lzss_size]);
            }
        }
        
        // Otherwise it's plain LZSS
        tracing::info!("ROM uses LZSS format - decompressing");
        let decoded = Self::decode_lzss(&data[lzss_offset..lzss_offset + lzss_size]);
        
        tracing::info!("Decoded ROM: {} bytes", decoded.len());
        Ok(decoded)
    }
    
    /// Find a constant value in the CHRP boot script
    /// Format: "h# XXXXXX constant name"
    fn find_boot_constant(script: &str, name: &str) -> Option<usize> {
        let pattern = format!("constant {}", name);
        if let Some(pos) = script.find(&pattern) {
            // Look backwards for "h# XXXXXX"
            let before = &script[..pos];
            if let Some(hex_start) = before.rfind("h# ") {
                let hex_str = &before[hex_start + 3..].trim_start();
                let hex_end = hex_str.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(hex_str.len());
                let hex_value = &hex_str[..hex_end];
                return usize::from_str_radix(hex_value, 16).ok();
            }
        }
        None
    }
    
    /// Decode parcels format ROM (Mac OS 9.x)
    fn decode_parcels(data: &[u8]) -> Result<Vec<u8>> {
        let mut result = Vec::with_capacity(4 * 1024 * 1024);
        let mut parcel_offset = 0x14; // First parcel at offset 0x14
        
        while parcel_offset != 0 && parcel_offset + 24 <= data.len() {
            let next_offset = BigEndian::read_u32(&data[parcel_offset..]) as usize;
            let parcel_type = BigEndian::read_u32(&data[parcel_offset + 4..]);
            
            tracing::debug!("Parcel at 0x{:X}: type={:08X}", parcel_offset, parcel_type);
            
            // Look for 'rom ' parcel (0x726F6D20)
            if parcel_type == 0x726F6D20 {
                let lzss_offset = BigEndian::read_u32(&data[parcel_offset + 8..]) as usize;
                let abs_offset = parcel_offset + lzss_offset;
                
                if next_offset > abs_offset && next_offset <= data.len() {
                    let lzss_size = next_offset - abs_offset;
                    tracing::info!("Found 'rom ' parcel: LZSS at 0x{:X}, size 0x{:X}", abs_offset, lzss_size);
                    result = Self::decode_lzss(&data[abs_offset..abs_offset + lzss_size]);
                    break;
                }
            }
            
            parcel_offset = next_offset;
        }
        
        if result.is_empty() {
            return Err(Error::Other("No 'rom ' parcel found in ROM".into()));
        }
        
        Ok(result)
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
        self.base_address.wrapping_add(self.entry_offset as u32)
    }
    
    /// Get the CHRP boot script (for NewWorld ROMs)
    pub fn get_boot_script(&self) -> Option<String> {
        if self.rom_type != RomType::NewWorld {
            return None;
        }
        
        // Find the end of the boot script (</CHRP-BOOT>)
        let start = "<CHRP-BOOT>".len();
        
        if let Some(end_pos) = self.data.windows(12)
            .position(|w| w == b"</CHRP-BOOT>") 
        {
            // Extract the script text
            if start < end_pos {
                if let Ok(script) = String::from_utf8(self.data[start..end_pos].to_vec()) {
                    return Some(script);
                }
            }
        }
        
        None
    }
    
    /// Get just the Forth code from the boot script (for NewWorld ROMs)
    /// This extracts the content between <BOOT-SCRIPT> and </BOOT-SCRIPT>
    pub fn get_forth_script(&self) -> Option<String> {
        let full_script = self.get_boot_script()?;
        
        // Find <BOOT-SCRIPT> and </BOOT-SCRIPT> tags
        let start_tag = "<BOOT-SCRIPT>";
        let end_tag = "</BOOT-SCRIPT>";
        
        if let Some(start_pos) = full_script.find(start_tag) {
            let script_start = start_pos + start_tag.len();
            if let Some(end_pos) = full_script[script_start..].find(end_tag) {
                let forth_script = &full_script[script_start..script_start + end_pos];
                return Some(forth_script.to_string());
            }
        }
        
        None
    }
    
    /// Find the ELF offset in raw NewWorld ROM data
    /// 
    /// Different ROM versions place the ELF at different offsets:
    /// - ROM 1.1-1.1.2 (1998): 0x3000
    /// - ROM 1.2-3.0 (1998-1999): 0x4000
    /// - ROM 3.7-10.2.1 (2000-2003): 0x5000
    fn find_elf_offset(data: &[u8]) -> Option<usize> {
        // Check common ELF offsets
        let candidates = [0x3000, 0x4000, 0x5000, 0x8000];
        const ELF_MAGIC: u32 = 0x7F454C46; // "\x7FELF"
        
        for &offset in &candidates {
            if offset + 4 <= data.len() {
                let magic = BigEndian::read_u32(&data[offset..offset + 4]);
                if magic == ELF_MAGIC {
                    tracing::info!("Found ELF at offset 0x{:X}", offset);
                    return Some(offset);
                }
            }
        }
        
        tracing::warn!("No ELF found in NewWorld ROM");
        None
    }
    
    /// Find the ELF offset in this ROM (instance method)
    /// 
    /// Different ROM versions place the ELF at different offsets:
    /// - ROM 1.1-1.1.2 (1998): 0x3000
    /// - ROM 1.2-3.0 (1998-1999): 0x4000
    /// - ROM 3.7-10.2.1 (2000-2003): 0x5000
    pub fn get_elf_offset(&self) -> Option<usize> {
        if self.rom_type != RomType::NewWorld {
            return None;
        }
        
        Self::find_elf_offset(&self.data)
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
    
    /// Read a range of bytes from ROM at the given offset
    pub fn read_range(&self, offset: usize, size: usize) -> Result<Vec<u8>> {
        if offset + size <= self.data.len() {
            Ok(self.data[offset..offset + size].to_vec())
        } else {
            Err(Error::Memory(format!(
                "ROM read_range out of bounds: offset={}, size={}, rom_size={}",
                offset, size, self.data.len()
            )))
        }
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
    
    /// Decode LZSS compressed data (used in NewWorld ROMs)
    /// Based on SheepShaver's implementation
    fn decode_lzss(src: &[u8]) -> Vec<u8> {
        let mut dest = Vec::with_capacity(4 * 1024 * 1024); // 4MB typical ROM size
        let mut dict = [0u8; 0x1000];
        let mut run_mask = 0u16;
        let mut dict_idx = 0xfee;
        let mut src_idx = 0;
        
        loop {
            if run_mask < 0x100 {
                // Start new run
                if src_idx >= src.len() {
                    break;
                }
                run_mask = (src[src_idx] as u16) | 0xff00;
                src_idx += 1;
            }
            
            let bit = (run_mask & 1) != 0;
            run_mask >>= 1;
            
            if bit {
                // Verbatim copy
                if src_idx >= src.len() {
                    break;
                }
                let c = src[src_idx];
                src_idx += 1;
                
                dict[dict_idx] = c;
                dict_idx = (dict_idx + 1) & 0xfff;
                dest.push(c);
            } else {
                // Copy from dictionary
                if src_idx + 1 >= src.len() {
                    break;
                }
                let b0 = src[src_idx] as usize;
                let b1 = src[src_idx + 1] as usize;
                src_idx += 2;
                
                let dict_offset = b0 | ((b1 & 0xf0) << 4);
                let run_length = (b1 & 0x0f) + 3;
                
                for _ in 0..run_length {
                    let c = dict[dict_offset];
                    dict[dict_idx] = c;
                    dict_idx = (dict_idx + 1) & 0xfff;
                    dest.push(c);
                }
            }
        }
        
        dest
    }
}
