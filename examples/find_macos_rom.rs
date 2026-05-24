// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Navigate HFS directory structure to find Mac OS ROM

use std::fs::File;
use newton_devices::storage::hfs::HfsVolume;

fn list_dir(volume: &mut HfsVolume<File>, parent_id: u32, indent: usize) -> Vec<(String, u32, bool)> {
    let mut result = Vec::new();
    
    match volume.list_directory(parent_id) {
        Ok(entries) => {
            for (name, is_file) in entries {
                let type_str = if is_file { "[FILE]" } else { "[DIR] " };
                println!("{}{} {}", "  ".repeat(indent), type_str, name);
                
                // For now, just collect directory names and IDs
                if !is_file {
                    // We need to get the directory ID, but list_directory doesn't return it
                    // Let's search for it individually
                    result.push((name.clone(), 0, is_file));
                }
            }
        }
        Err(e) => {
            println!("{}Error: {}", "  ".repeat(indent), e);
        }
    }
    
    result
}

fn main() {
    let file = File::open("disks/macos-922-uni.iso").expect("Failed to open ISO");
    
    // HFS partition starts at block 329, with 512-byte blocks
    let hfs_offset = 329 * 512;
    
    println!("Opening HFS volume at offset: 0x{:X}", hfs_offset);
    let mut volume = HfsVolume::open(file, hfs_offset).expect("Failed to open HFS volume");
    println!("Volume name: {}\n", volume.mdb.volume_name);
    
    // List root directory (ID 2)
    println!("=== Root Directory ===");
    let root_entries = list_dir(&mut volume, 2, 0);
    
    // Try to find "System Folder" or something that might contain the Mac OS ROM
    println!("\n=== Searching for Mac OS ROM ===");
    
    // Common locations to check
    let paths_to_try = vec![
        "System Folder:Mac OS ROM",
        "Applications (Mac OS 9):Installers:Mac OS ROM",
        "Mac OS ROM",
        "System:Mac OS ROM",
    ];
    
    for path in paths_to_try {
        println!("Trying: {}", path);
        match volume.find_file(path) {
            Ok(data) => {
                println!("  ✓ Found! Size: {} bytes ({} KB)", data.len(), data.len() / 1024);
                return;
            }
            Err(e) => {
                println!("  ✗ {}", e);
            }
        }
    }
    
    println!("\nMac OS ROM not found in common locations.");
}
