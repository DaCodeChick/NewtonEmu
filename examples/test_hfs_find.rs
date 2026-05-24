// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Test HFS file finding on Mac OS 9.2.2 CD

use std::fs::File;
use newton_devices::storage::hfs::HfsVolume;

fn main() {
    tracing_subscriber::fmt::init();
    
    let file = File::open("disks/macos-922-uni.iso").expect("Failed to open ISO");
    
    // HFS partition starts at block 329, with 512-byte blocks (Apple Partition Map uses 512-byte blocks)
    let hfs_offset = 329 * 512;
    
    println!("Opening HFS volume at offset: 0x{:X}", hfs_offset);
    let mut volume = HfsVolume::open(file, hfs_offset).expect("Failed to open HFS volume");
    println!("Volume name: {}", volume.mdb.volume_name);
    
    // Try to find the Mac OS ROM file
    println!("\n=== Searching for System Folder:Mac OS ROM ===");
    match volume.find_file("System Folder:Mac OS ROM") {
        Ok(data) => {
            println!("✓ Found Mac OS ROM!");
            println!("  Size: {} bytes ({} KB)", data.len(), data.len() / 1024);
            println!("  First 16 bytes: {:02X?}", &data[..16.min(data.len())]);
        }
        Err(e) => {
            println!("✗ Failed to find Mac OS ROM: {}", e);
        }
    }
    
    // Also try just finding the System Folder first
    println!("\n=== Searching for System Folder ===");
    match volume.find_file("System Folder") {
        Ok(_) => println!("✓ Found System Folder (but it's a file?)"),
        Err(e) => println!("✗ System Folder not found: {}", e),
    }
}
