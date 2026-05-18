// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Test storage device functionality

use anyhow::Result;
use newton_devices::storage::{IsoImage, RawDiskImage, StorageBus, BlockDevice};
use std::sync::{Arc, RwLock};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("==================================================");
    println!("NewtonEmu Storage Device Test");
    println!("==================================================\n");

    let mut bus = StorageBus::new();

    // Test 1: Create a raw disk image
    println!("Test 1: Creating a 100MB raw disk image...");
    let disk_path = "/tmp/opencode/test_disk.img";
    let disk = RawDiskImage::create(disk_path, 100)?;
    let disk: Arc<RwLock<dyn BlockDevice>> = Arc::new(RwLock::new(disk));
    
    // Print disk info
    {
        let d = disk.read().unwrap();
        let info = d.info();
        println!("  Created: {}", info.model);
        println!("  Size: {} bytes ({} MB)", info.size, info.size / 1024 / 1024);
        println!("  Block size: {} bytes", info.block_size);
        println!("  Blocks: {}", info.size / info.block_size as u64);
    }
    
    // Test 2: Write and read back data
    println!("\nTest 2: Writing test data...");
    {
        let mut d = disk.write().unwrap();
        let test_data = b"Hello from NewtonEmu virtual disk!";
        let mut buffer = vec![0u8; 512];
        buffer[..test_data.len()].copy_from_slice(test_data);
        
        d.write_blocks(0, 1, &buffer)?;
        println!("  Wrote {} bytes to LBA 0", test_data.len());
    }
    
    println!("  Reading back data...");
    {
        let d = disk.read().unwrap();
        let mut buffer = vec![0u8; 512];
        d.read_blocks(0, 1, &mut buffer)?;
        
        let read_data = std::str::from_utf8(&buffer[..34]).unwrap_or("(invalid UTF-8)");
        println!("  Read: {}", read_data);
    }
    
    // Test 3: Attach to SCSI bus
    println!("\nTest 3: Attaching disk to SCSI bus...");
    bus.attach_scsi(0, disk)?;
    println!("  Attached as SCSI ID 0");
    
    // Test 4: Try to open an ISO image (if available)
    println!("\nTest 4: Checking for ISO images...");
    let iso_paths = [
        "disks/macos9_install.iso",
        "disks/macos_install.iso",
        "/tmp/opencode/test.iso",
    ];
    
    let mut found_iso = false;
    for iso_path in &iso_paths {
        if let Ok(iso) = IsoImage::open(iso_path) {
            println!("  Found ISO: {}", iso_path);
            let info = iso.info();
            println!("    Label: {}", info.model);
            println!("    Size: {} bytes ({} MB)", info.size, info.size / 1024 / 1024);
            println!("    Sectors: {}", info.size / info.block_size as u64);
            
            let iso: Arc<RwLock<dyn BlockDevice>> = Arc::new(RwLock::new(iso));
            bus.attach_scsi(3, iso)?;
            println!("    Attached as SCSI ID 3");
            found_iso = true;
            break;
        }
    }
    
    if !found_iso {
        println!("  No ISO images found (this is OK)");
        println!("  To test ISO mounting, place an ISO at one of these paths:");
        for path in &iso_paths {
            println!("    - {}", path);
        }
    }
    
    // Test 5: List all devices
    println!("\nTest 5: Listing all attached devices...");
    bus.list_devices();
    
    // Test 6: SCSI command execution
    println!("\nTest 6: Testing SCSI commands...");
    if let Some(scsi_dev) = bus.scsi_device_mut(0) {
        println!("  Executing TEST UNIT READY...");
        let cdb = [0x00, 0, 0, 0, 0, 0]; // TEST UNIT READY
        let mut data = vec![0u8; 0];
        match scsi_dev.execute_command(&cdb, &mut data) {
            Ok((status, _)) => println!("    Status: {:?}", status),
            Err(e) => println!("    Error: {}", e),
        }
        
        println!("  Executing INQUIRY...");
        let cdb = [0x12, 0, 0, 0, 36, 0]; // INQUIRY
        let mut data = vec![0u8; 36];
        match scsi_dev.execute_command(&cdb, &mut data) {
            Ok((status, len)) => {
                println!("    Status: {:?}, {} bytes returned", status, len);
                
                // Parse vendor and product
                let vendor = std::str::from_utf8(&data[8..16]).unwrap_or("???");
                let product = std::str::from_utf8(&data[16..32]).unwrap_or("???");
                println!("    Vendor: {}", vendor.trim());
                println!("    Product: {}", product.trim());
            }
            Err(e) => println!("    Error: {}", e),
        }
        
        println!("  Executing READ CAPACITY...");
        let cdb = [0x25, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // READ CAPACITY
        let mut data = vec![0u8; 8];
        match scsi_dev.execute_command(&cdb, &mut data) {
            Ok((status, _)) => {
                println!("    Status: {:?}", status);
                let last_lba = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                let block_size = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
                println!("    Last LBA: {}", last_lba);
                println!("    Block size: {} bytes", block_size);
                println!("    Total capacity: {} MB", 
                        (last_lba as u64 + 1) * block_size as u64 / 1024 / 1024);
            }
            Err(e) => println!("    Error: {}", e),
        }
    }
    
    println!("\n==================================================");
    println!("All tests completed successfully!");
    println!("==================================================");
    
    Ok(())
}
