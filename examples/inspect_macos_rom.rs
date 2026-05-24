// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Inspect Mac OS ROM files

use std::fs::File;
use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rom_path = if args.len() > 1 {
        &args[1]
    } else {
        "roms/2003-04-03 - Mac OS ROM 10.2.1.rom"
    };
    
    println!("Reading Mac OS ROM: {}", rom_path);
    
    let mut file = File::open(rom_path).expect("Failed to open ROM file");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("Failed to read ROM file");
    
    println!("ROM Size: {} bytes ({} KB, {} MB)", 
             data.len(), data.len() / 1024, data.len() / (1024 * 1024));
    
    // Check for common ROM signatures
    println!("\n=== ROM Header Analysis ===");
    
    // First 16 bytes
    println!("First 16 bytes: {:02X?}", &data[..16.min(data.len())]);
    
    // Look for copyright strings
    println!("\n=== Searching for Copyright Strings ===");
    let data_str = String::from_utf8_lossy(&data);
    for (i, line) in data_str.lines().enumerate() {
        if line.contains("Copyright") || line.contains("Apple") {
            println!("  Line {}: {}", i, line.chars().take(80).collect::<String>());
        }
    }
    
    // Check for "Mac OS ROM" string
    println!("\n=== Searching for 'Mac OS ROM' String ===");
    if let Some(pos) = data.windows(11).position(|w| w == b"Mac OS ROM\0") {
        println!("  Found at offset: 0x{:X}", pos);
        let context_start = pos.saturating_sub(32);
        let context_end = (pos + 48).min(data.len());
        println!("  Context: {:?}", String::from_utf8_lossy(&data[context_start..context_end]));
    } else {
        println!("  'Mac OS ROM' string not found");
    }
}
