// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Test storage device attachment
//!
//! This example demonstrates how to attach disk images to the emulator.

use newton_core::{Emulator, EmulatorConfig};
use newton_core::config::{CpuConfig, CpuModel, MemoryConfig, DisplayConfig};
use newton_utils::Result;
use std::env;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();
    
    println!("==================================================");
    println!("NewtonEmu - Storage Attachment Test");
    println!("==================================================");
    println!();
    
    // Create basic configuration
    let config = EmulatorConfig {
        cpu: CpuConfig {
            model: CpuModel::G4,
            clock_speed: 450,
            enable_jit: false,
        },
        memory: MemoryConfig {
            ram_size_mb: 256,
            rom_path: None,
        },
        display: DisplayConfig {
            width: 800,
            height: 600,
            color_depth: 32,
        },
    };
    
    // Create emulator
    println!("Creating emulator...");
    let mut emulator = Emulator::new(config)?;
    println!("✅ Emulator created");
    println!();
    
    // Check command-line arguments for disk images to attach
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <disk-image> [disk-image2] ...", args[0]);
        println!();
        println!("Supported formats:");
        println!("  - ISO9660 (.iso)");
        println!("  - Apple Disk Image (.dmg)");
        println!("  - Roxio Toast (.toast)");
        println!("  - ZIP archives (.zip)");
        println!("  - Raw disk images (.img, .bin, etc.)");
        println!();
        println!("Example:");
        println!("  {} macos9.iso disk.dmg", args[0]);
        println!();
        return Ok(());
    }
    
    // Attach disk images
    println!("Attaching disk images...");
    println!();
    
    for (i, path) in args.iter().skip(1).enumerate() {
        let scsi_id = i as u8;
        
        println!("Image {}: {}", i + 1, path);
        match emulator.attach_disk_image(path, scsi_id, true) {
            Ok(()) => println!("  ✅ Attached to SCSI ID {}", scsi_id),
            Err(e) => println!("  ❌ Failed: {}", e),
        }
        println!();
    }
    
    // List all devices
    println!("==================================================");
    println!("Attached Storage Devices:");
    println!("==================================================");
    emulator.storage_bus().list_devices();
    println!();
    
    println!("✅ Test completed successfully!");
    
    Ok(())
}
