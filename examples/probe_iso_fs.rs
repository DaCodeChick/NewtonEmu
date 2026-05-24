// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Probe ISO filesystem to detect HFS/HFS+ format

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "disks/macos-922-uni.iso".to_string());

    println!("Probing: {}", path);
    let mut file = File::open(&path)?;

    // Read Apple Partition Map
    // Block 0: Driver Descriptor Map
    file.seek(SeekFrom::Start(0))?;
    let mut block = vec![0u8; 512];
    file.read_exact(&mut block)?;
    
    let signature = u16::from_be_bytes([block[0], block[1]]);
    println!("\nBlock 0 signature: 0x{:04X}", signature);
    if signature == 0x4552 {
        println!("  -> Apple Driver Descriptor Map (ER)");
        let block_size = u16::from_be_bytes([block[2], block[3]]);
        let block_count = u32::from_be_bytes([block[4], block[5], block[6], block[7]]);
        println!("  Block size: {}", block_size);
        println!("  Block count: {}", block_count);
    }

    // Read partition map starting at block 1
    file.seek(SeekFrom::Start(512))?;
    file.read_exact(&mut block)?;
    
    let pm_sig = u16::from_be_bytes([block[0], block[1]]);
    println!("\nBlock 1 signature: 0x{:04X}", pm_sig);
    if pm_sig == 0x504D {
        println!("  -> Apple Partition Map (PM)");
        let map_block_count = u32::from_be_bytes([block[4], block[5], block[6], block[7]]);
        println!("  Map block count: {}", map_block_count);
        
        // Find the HFS partition
        for i in 1..=map_block_count.min(20) {
            file.seek(SeekFrom::Start(512 * i as u64))?;
            file.read_exact(&mut block)?;
            
            // Partition type starts at offset 48
            let ptype = String::from_utf8_lossy(&block[48..80]).trim_end_matches('\0').to_string();
            let pname = String::from_utf8_lossy(&block[16..48]).trim_end_matches('\0').to_string();
            let pstart = u32::from_be_bytes([block[8], block[9], block[10], block[11]]);
            let pcount = u32::from_be_bytes([block[12], block[13], block[14], block[15]]);
            
            println!("\nPartition {}: {}", i, pname);
            println!("  Type: {}", ptype);
            println!("  Start block: {}", pstart);
            println!("  Block count: {}", pcount);
            
            if ptype.contains("HFS") || ptype.contains("Apple_HFS") {
                // Check if it's HFS+ or classic HFS
                let hfs_start = 512 * pstart as u64;
                println!("  HFS partition at offset: 0x{:X}", hfs_start);
                
                // Check at offset 1024 for HFS+ header
                file.seek(SeekFrom::Start(hfs_start + 1024))?;
                let mut hfs_header = vec![0u8; 512];
                file.read_exact(&mut hfs_header)?;
                
                let hfs_sig = u16::from_be_bytes([hfs_header[0], hfs_header[1]]);
                println!("  Filesystem signature at +1024: 0x{:04X}", hfs_sig);
                match hfs_sig {
                    0x482B => println!("    -> HFS+ (H+)"),
                    0x4858 => println!("    -> HFSX (HX)"),
                    0x4244 => println!("    -> HFS (BD - classic)"),
                    _ => {
                        // Check at offset 0 for classic HFS
                        file.seek(SeekFrom::Start(hfs_start))?;
                        file.read_exact(&mut hfs_header)?;
                        let hfs_sig2 = u16::from_be_bytes([hfs_header[0], hfs_header[1]]);
                        println!("  Filesystem signature at +0: 0x{:04X}", hfs_sig2);
                        if hfs_sig2 == 0x4244 {
                            println!("    -> HFS (BD - classic)");
                        } else {
                            println!("    -> Unknown filesystem");
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
