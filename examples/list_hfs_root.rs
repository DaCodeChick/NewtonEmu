// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! List contents of HFS root directory

use std::fs::File;
use newton_devices::storage::hfs::HfsVolume;

fn main() {
    let file = File::open("disks/macos-922-uni.iso").expect("Failed to open ISO");
    
    // HFS partition starts at block 329, with 512-byte blocks
    let hfs_offset = 329 * 512;
    
    println!("Opening HFS volume at offset: 0x{:X}", hfs_offset);
    let mut volume = HfsVolume::open(file, hfs_offset).expect("Failed to open HFS volume");
    println!("Volume name: {}\n", volume.mdb.volume_name);
    
    // List root directory (ID 2)
    println!("=== Root Directory Contents ===");
    match volume.list_directory(2) {
        Ok(entries) => {
            for (name, is_file) in &entries {
                let type_str = if *is_file { "[FILE]" } else { "[DIR] " };
                println!("  {} {}", type_str, name);
            }
            println!("\nTotal: {} entries", entries.len());
        }
        Err(e) => {
            println!("Error listing directory: {}", e);
        }
    }
}
