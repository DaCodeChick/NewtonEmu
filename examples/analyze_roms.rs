// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! ROM analysis tool - examine multiple ROM files and compare their structure

use anyhow::Result;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    println!("==================================================");
    println!("Mac OS ROM Analysis");
    println!("==================================================\n");

    // Get all ROM files
    let rom_dir = Path::new("roms");
    let mut rom_files: Vec<_> = fs::read_dir(rom_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "rom")
                .unwrap_or(false)
        })
        .collect();
    
    rom_files.sort_by_key(|entry| entry.file_name());

    println!("Found {} ROM files:\n", rom_files.len());

    for entry in &rom_files {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy();
        
        if let Ok(rom_data) = fs::read(&path) {
            analyze_rom(&name, &rom_data);
        }
    }

    Ok(())
}

fn analyze_rom(name: &str, data: &[u8]) {
    println!("ROM: {}", name);
    println!("  Size: {} bytes ({:.2} MB)", data.len(), data.len() as f64 / 1_048_576.0);
    
    // Check for CHRP boot script
    let has_chrp = data.len() >= 11 && &data[0..11] == b"<CHRP-BOOT>";
    println!("  CHRP boot script: {}", if has_chrp { "YES" } else { "NO" });
    
    if has_chrp {
        // Find end of CHRP script
        if let Some(end_pos) = find_pattern(data, b"</CHRP-BOOT>") {
            println!("  CHRP script length: {} bytes (ends at 0x{:X})", end_pos + 13, end_pos + 13);
        }
    }
    
    // Check for ELF at common offsets
    let elf_offsets = [0x0000, 0x3000, 0x4000, 0x5000, 0x8000, 0x10000];
    for &offset in &elf_offsets {
        if offset + 4 <= data.len() {
            let magic = u32::from_be_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
            if magic == 0x7f454c46 { // ELF magic
                println!("  ELF found at: 0x{:X}", offset);
                analyze_elf(data, offset);
            }
        }
    }
    
    // Check for common code patterns at start
    if data.len() >= 4 {
        let first_instr = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        if !has_chrp {
            println!("  First instruction: 0x{:08X} ({})", first_instr, decode_simple(first_instr));
        }
    }
    
    // Check for LZSS/prcl data
    if let Some(prcl_offset) = find_pattern(data, b"prcl") {
        println!("  LZSS/prcl found at: 0x{:X}", prcl_offset);
    }
    
    // Check for "Mac OS ROM" string
    if let Some(str_offset) = find_pattern(data, b"Mac OS ROM") {
        println!("  'Mac OS ROM' string at: 0x{:X}", str_offset);
    }
    
    // Check for OpenFirmware signatures
    if find_pattern(data, b"AAPL,MacOS,PowerPC").is_some() {
        println!("  Contains: OpenFirmware PowerPC marker");
    }
    
    println!();
}

fn analyze_elf(data: &[u8], offset: usize) {
    if offset + 52 > data.len() {
        return;
    }
    
    let e_type = u16::from_be_bytes([data[offset+16], data[offset+17]]);
    let e_machine = u16::from_be_bytes([data[offset+18], data[offset+19]]);
    let e_entry = u32::from_be_bytes([
        data[offset+24], data[offset+25], data[offset+26], data[offset+27]
    ]);
    let e_phoff = u32::from_be_bytes([
        data[offset+28], data[offset+29], data[offset+30], data[offset+31]
    ]);
    let e_phnum = u16::from_be_bytes([data[offset+44], data[offset+45]]);
    
    let type_str = match e_type {
        1 => "REL",
        2 => "EXEC",
        3 => "DYN",
        4 => "CORE",
        _ => "UNKNOWN",
    };
    
    println!("    Type: {} ({})", e_type, type_str);
    println!("    Machine: {} ({})", e_machine, if e_machine == 20 { "PowerPC" } else { "?" });
    println!("    Entry point: 0x{:08X}", e_entry);
    println!("    Program headers: {} at offset 0x{:X}", e_phnum, e_phoff);
}

fn decode_simple(instr: u32) -> &'static str {
    let opcode = instr >> 26;
    match opcode {
        16 => "bc (branch conditional)",
        18 => "b (branch)",
        19 => {
            let xo = (instr >> 1) & 0x3FF;
            match xo {
                16 => "bclr (branch to LR)",
                528 => "bcctr (branch to CTR)",
                _ => "SPR/CR ops",
            }
        }
        31 => {
            let xo = (instr >> 1) & 0x3FF;
            match xo {
                467 => "mtspr",
                339 => "mfspr",
                _ => "extended ops",
            }
        }
        32 => "lwz",
        36 => "stw",
        _ => "other",
    }
}

fn find_pattern(data: &[u8], pattern: &[u8]) -> Option<usize> {
    data.windows(pattern.len())
        .position(|window| window == pattern)
}
