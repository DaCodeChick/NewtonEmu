// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Test reading Mac OS 9.2 Toast image
//!
//! This example tests the HFS parser on a bootable Mac OS 9.2 Toast image.

use newton_devices::storage::hfs::HfsVolume;
use std::fs::File;

fn main() -> anyhow::Result<()> {
    println!("==================================================");
    println!("Testing Toast Image HFS Parsing");
    println!("==================================================\n");
    
    let file = File::open("disks/Power Mac G4 Install 9.2.toast")?;
    
    // HFS partition "Macintosh HD" starts at block 968 (512-byte blocks)
    let hfs_offset = 968 * 512;
    
    println!("Opening HFS volume at offset: 0x{:X}", hfs_offset);
    let mut volume = HfsVolume::open(file, hfs_offset)?;
    
    println!("Volume opened successfully");
    println!();
    
    // Try to read a file from the System Folder
    println!("Attempting to read: System Folder:Mac OS ROM");
    match volume.find_file("System Folder:Mac OS ROM") {
        Ok(data) => {
            println!("✓ Successfully read Mac OS ROM file!");
            println!("  Size: {} bytes ({:.1} KB)", data.len(), data.len() as f64 / 1024.0);
            println!("  First 32 bytes: {:02X?}", &data[..32.min(data.len())]);
        }
        Err(e) => {
            println!("✗ Failed to read Mac OS ROM: {}", e);
            
            // Try some other common files
            println!("\nTrying other files:");
            for path in &["System Folder", "System", "Applications (Mac OS 9)"] {
                match volume.find_file(path) {
                    Ok(data) => println!("  ✓ Found: {} ({} bytes)", path, data.len()),
                    Err(_) => println!("  ✗ Not found: {}", path),
                }
            }
        }
    }
    
    Ok(())
}
