// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Test various disk image formats
//!
//! This example demonstrates opening different disk image formats:
//! - ISO9660 (.iso)
//! - Raw disk images (.img, .bin)
//! - Apple Disk Images (.dmg)
//! - Roxio Toast images (.toast)
//! - ZIP-compressed images (.zip)

use newton_devices::storage::{BlockDevice, IsoImage, RawDiskImage, DmgImage, ToastImage, ZipImage};
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
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <image-file>", args[0]);
        eprintln!();
        eprintln!("Supported formats:");
        eprintln!("  - ISO9660 (.iso)");
        eprintln!("  - Raw disk (.img, .bin, .cdr)");
        eprintln!("  - Apple Disk Image (.dmg)");
        eprintln!("  - Roxio Toast (.toast)");
        eprintln!("  - ZIP archives containing disk images (.zip)");
        std::process::exit(1);
    }
    
    let path = &args[1];
    let path_lower = path.to_lowercase();
    
    println!("Opening disk image: {}", path);
    println!();
    
    // Determine format and open
    if path_lower.ends_with(".iso") {
        test_iso(path)?;
    } else if path_lower.ends_with(".dmg") {
        test_dmg(path)?;
    } else if path_lower.ends_with(".toast") {
        test_toast(path)?;
    } else if path_lower.ends_with(".zip") {
        test_zip(path)?;
    } else {
        // Try as raw
        test_raw(path)?;
    }
    
    Ok(())
}

fn test_iso(path: &str) -> Result<()> {
    println!("Opening as ISO9660 image...");
    let iso = IsoImage::open(path)?;
    print_device_info(&iso);
    test_read(&iso)?;
    Ok(())
}

fn test_raw(path: &str) -> Result<()> {
    println!("Opening as raw disk image...");
    let raw = RawDiskImage::open(path, false)?;
    print_device_info(&raw);
    test_read(&raw)?;
    Ok(())
}

fn test_dmg(path: &str) -> Result<()> {
    println!("Opening as Apple Disk Image (.dmg)...");
    let dmg = DmgImage::open(path)?;
    print_device_info(&dmg);
    test_read(&dmg)?;
    Ok(())
}

fn test_toast(path: &str) -> Result<()> {
    println!("Opening as Roxio Toast image (.toast)...");
    let toast = ToastImage::open(path)?;
    print_device_info(&toast);
    test_read(&toast)?;
    Ok(())
}

fn test_zip(path: &str) -> Result<()> {
    println!("Opening as ZIP-compressed disk image...");
    let zip = ZipImage::open(path)?;
    print_device_info(&zip);
    test_read(&zip)?;
    Ok(())
}

fn print_device_info<D: BlockDevice>(device: &D) {
    let info = device.info();
    println!("Device Information:");
    println!("  Type:         {:?}", info.device_type);
    println!("  Model:        {}", info.model);
    println!("  Serial:       {}", info.serial);
    println!("  Block Size:   {} bytes", info.block_size);
    println!("  Total Size:   {} MB ({} bytes)", info.size / (1024 * 1024), info.size);
    println!("  Read-Only:    {}", info.read_only);
    println!("  Removable:    {}", info.removable);
    println!();
}

fn test_read<D: BlockDevice>(device: &D) -> Result<()> {
    println!("Testing read operations...");
    
    // Read first block
    let mut buffer = vec![0u8; device.info().block_size as usize];
    let bytes_read = device.read_blocks(0, 1, &mut buffer)?;
    
    println!("  Read {} bytes from LBA 0", bytes_read);
    println!("  First 64 bytes (hex):");
    
    let to_print = std::cmp::min(64, bytes_read);
    for (i, chunk) in buffer[..to_print].chunks(16).enumerate() {
        print!("    {:04X}:", i * 16);
        for byte in chunk {
            print!(" {:02X}", byte);
        }
        println!();
    }
    
    println!();
    println!("Success!");
    
    Ok(())
}
