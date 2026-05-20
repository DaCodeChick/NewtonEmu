// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Scan ROM files to detect NewWorld ROMs with CHRP boot scripts

use newton_core::rom::Rom;
use std::fs;
use std::path::PathBuf;

fn main() -> newton_utils::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let roms_dir = PathBuf::from("roms");
    
    if !roms_dir.exists() {
        eprintln!("roms/ directory not found");
        return Ok(());
    }
    
    println!("Scanning ROMs in roms/...\n");
    
    let mut entries: Vec<_> = fs::read_dir(&roms_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("rom"))
                .unwrap_or(false)
        })
        .collect();
    
    entries.sort_by_key(|e| e.file_name());
    
    for entry in entries {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy();
        
        match Rom::load_from_file(&path) {
            Ok(rom) => {
                println!("✓ {}", filename);
                println!("  Type: {:?}", rom.rom_type());
                println!("  Size: {} bytes ({:.2} MB)", rom.size(), rom.size() as f64 / (1024.0 * 1024.0));
                println!("  Base: 0x{:08X}", rom.base_address());
                println!("  Entry: 0x{:08X}", rom.entry_address());
                
                // Check for CHRP boot script
                if let Some(script) = rom.get_boot_script() {
                    println!("  Boot script: {} bytes", script.len());
                    
                    // Show first few lines
                    let lines: Vec<&str> = script.lines().take(3).collect();
                    for line in lines {
                        println!("    {}", line);
                    }
                    if script.len() > 200 {
                        println!("    ...");
                    }
                }
                println!();
            }
            Err(e) => {
                println!("✗ {}: {}", filename, e);
                println!();
            }
        }
    }
    
    Ok(())
}
