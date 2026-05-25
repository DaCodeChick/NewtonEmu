// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! ROM patches for NewWorld ROMs
//! 
//! Based on SheepShaver's rom_patches.cpp

use byteorder::{BigEndian, ByteOrder};
use newton_utils::Result;

/// Apply necessary patches to NewWorld ROM
pub fn patch_newworld_rom(rom_data: &mut [u8], ram_base: u32, ram_size: u32) -> Result<()> {
    tracing::info!("Applying NewWorld ROM patches");
    
    // Check if this is a NewWorld ROM (has boot structure at 0x30d000)
    if rom_data.len() < 0x30d400 {
        tracing::warn!("ROM too small for NewWorld patches");
        return Ok(());
    }
    
    // ROM boot structure patches (at 0x30d000)
    let boot_struct_offset = 0x30d000;
    
    // Physical RAM base (offset 0x360 from boot structure)
    // NewWorld ROMs expect this to tell them where RAM is
    let ram_base_offset = boot_struct_offset + 0x360;
    BigEndian::write_u32(&mut rom_data[ram_base_offset..], ram_base);
    tracing::info!("  ✓ Set physical RAM base to 0x{:08X} at ROM offset 0x{:08X}", 
                   ram_base, ram_base_offset);
    
    // Note: SheepShaver also patches:
    // - LA_InfoRecord, LA_KernelData, LA_EmulatorData (0x9c, 0xa0, 0xa4)
    // - LA_DispatchTable, LA_EmulatorCode (0xa8, 0xac)
    // - 68k reset vector (0xfd8)
    // But these are for the 68k emulator which we don't need yet
    
    // Skip SR/BAT/SDR init at 0x310000
    // These initialize MMU which we're handling differently
    let sr_init_offset = 0x310000;
    if rom_data.len() > sr_init_offset + 8 {
        // Write NOPs
        const POWERPC_NOP: u32 = 0x60000000;
        BigEndian::write_u32(&mut rom_data[sr_init_offset..], POWERPC_NOP);
        BigEndian::write_u32(&mut rom_data[sr_init_offset + 4..], 0x38000000);
        tracing::info!("  ✓ Patched SR/BAT/SDR init at 0x{:08X}", sr_init_offset);
    }
    
    tracing::info!("NewWorld ROM patches applied successfully");
    Ok(())
}

/// Search for a byte pattern in ROM
#[allow(dead_code)]
pub fn find_rom_pattern(rom_data: &[u8], start: usize, end: usize, pattern: &[u8]) -> Option<usize> {
    let end = end.min(rom_data.len());
    if start >= end || pattern.is_empty() {
        return None;
    }
    
    for i in start..=(end - pattern.len()) {
        if &rom_data[i..i + pattern.len()] == pattern {
            return Some(i);
        }
    }
    None
}
