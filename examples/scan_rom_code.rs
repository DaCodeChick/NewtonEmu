// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Scan ROM for executable code sections
//!
//! This example analyzes a Mac OS ROM to find regions that contain
//! actual PowerPC code vs. data/resources.

use anyhow::Result;
use std::fs;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let rom_path = args.get(1)
        .map(|s| s.as_str())
        .unwrap_or("roms/1999-09-27 - Mac OS ROM 3.0.rom");
    
    println!("=== ROM Code Section Scanner ===\n");
    println!("Analyzing: {}\n", rom_path);
    
    let rom = fs::read(rom_path)?;
    let rom_base = if rom.len() <= 1024 * 1024 {
        0xFFF00000
    } else {
        0xFFC00000
    };
    
    println!("ROM size: {} bytes ({:.1} MB)", rom.len(), rom.len() as f64 / 1024.0 / 1024.0);
    println!("ROM base: 0x{:08X}", rom_base);
    println!("ROM end:  0x{:08X}\n", rom_base + rom.len() as u32);
    
    println!("Scanning for code sections (analyzing every 1KB)...\n");
    println!("{:12} {:12} {:8} {}", "ROM Offset", "Address", "Code %", "Classification");
    println!("{}", "-".repeat(60));
    
    let mut best_code_sections = Vec::new();
    
    for offset in (0..rom.len()).step_by(1024) {
        let chunk_size = std::cmp::min(256, rom.len() - offset);  // Sample 256 bytes
        let mut valid_count = 0;
        let mut total_count = 0;
        
        for i in (0..chunk_size).step_by(4) {
            if offset + i + 4 > rom.len() {
                break;
            }
            
            let bytes = &rom[offset + i..offset + i + 4];
            let instr = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            let opcode = (instr >> 26) & 0x3F;
            
            // Common PowerPC opcodes
            let is_valid = matches!(opcode,
                // Branches
                16 | 18 |
                // Extended (includes many instructions)
                31 |
                // Loads
                32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 40 | 41 | 42 | 43 |
                // Stores  
                44 | 45 | 46 | 47 |
                // Immediate arithmetic/logical
                7 | 8 | 10 | 11 | 12 | 13 | 14 | 15 |
                20 | 21 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 30
            );
            
            if is_valid {
                valid_count += 1;
            }
            total_count += 1;
        }
        
        let code_percent = if total_count > 0 {
            (valid_count * 100) / total_count
        } else {
            0
        };
        
        let classification = if code_percent >= 75 {
            "Likely code"
        } else if code_percent >= 50 {
            "Mixed code/data"
        } else if code_percent >= 25 {
            "Mostly data"
        } else {
            "Data/resources"
        };
        
        if code_percent >= 50 {
            let addr = rom_base + offset as u32;
            println!("0x{:08X}   0x{:08X}   {:3}%     {}", 
                     offset, addr, code_percent, classification);
            
            if code_percent >= 75 {
                best_code_sections.push((offset, addr, code_percent));
            }
        }
    }
    
    println!("\n{}", "=".repeat(60));
    println!("Best code sections (>=75% valid opcodes):\n");
    
    if best_code_sections.is_empty() {
        println!("No high-confidence code sections found.");
        println!("This ROM may be compressed or encrypted.");
    } else {
        for (offset, addr, percent) in best_code_sections.iter().take(10) {
            println!("  ROM offset 0x{:08X} -> PC 0x{:08X} ({}% code)", offset, addr, percent);
        }
        
        println!("\nRecommended entry points to try:");
        let (offset, addr, _) = best_code_sections[0];
        println!("  Primary:   0x{:08X} (ROM offset 0x{:08X})", addr, offset);
        
        if best_code_sections.len() > 1 {
            let (offset, addr, _) = best_code_sections[1];
            println!("  Secondary: 0x{:08X} (ROM offset 0x{:08X})", addr, offset);
        }
    }
    
    println!("\n=== Scan Complete ===");
    
    Ok(())
}
