// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Test SCSI commands through MESH MMIO
//!
//! This example demonstrates exercising SCSI commands (INQUIRY, READ CAPACITY, READ)
//! through the MESH controller via memory-mapped I/O.

use newton_core::{Emulator, EmulatorConfig};
use newton_core::config::{CpuConfig, CpuModel, MemoryConfig, DisplayConfig};
use newton_cpu::MemoryInterface;
use newton_utils::Result;
use std::env;

// MESH register offsets
const MESH_REG_FIFO: u32 = 0x01;
const MESH_REG_SEQUENCE: u32 = 0x02;
const MESH_REG_FIFO_COUNT: u32 = 0x05;
const MESH_REG_DEST_ID: u32 = 0x0B;

// MESH sequence commands
const MESH_CMD_ARBITRATE: u8 = 0x01;
const MESH_CMD_SELECT: u8 = 0x02;
const MESH_CMD_COMMAND: u8 = 0x03;
const MESH_CMD_DATA_IN: u8 = 0x05;

// SCSI commands
const SCSI_CMD_INQUIRY: u8 = 0x12;
const SCSI_CMD_READ_CAPACITY: u8 = 0x25;
const SCSI_CMD_READ10: u8 = 0x28;

// MESH controller base address (must match emulator.rs)
const MESH_BASE_ADDR: u32 = 0xF3000000;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();
    
    println!("==================================================");
    println!("SCSI Command Test via MESH MMIO");
    println!("==================================================");
    println!();
    
    // Check for disk image argument
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <disk-image>", args[0]);
        println!();
        println!("This test will:");
        println!("  1. Attach the disk image to SCSI ID 3");
        println!("  2. Execute INQUIRY command via MESH MMIO");
        println!("  3. Execute READ CAPACITY command via MESH MMIO");
        println!("  4. Execute READ(10) command via MESH MMIO");
        println!();
        println!("Example: {} test.iso", args[0]);
        return Ok(());
    }
    
    let disk_path = &args[1];
    
    // Create emulator
    println!("Creating emulator...");
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
    
    let mut emulator = Emulator::new(config)?;
    println!("✅ Emulator created");
    println!();
    
    // Attach disk image to SCSI ID 3
    println!("Attaching disk image: {}", disk_path);
    emulator.attach_disk_image(disk_path, 3, true)?;
    println!("✅ Disk attached to SCSI ID 3");
    println!();
    
    // Get memory interface for MMIO access
    let memory = emulator.memory();
    
    println!("==================================================");
    println!("Test 1: SCSI INQUIRY Command");
    println!("==================================================");
    
    // Set target ID to 3
    println!("Setting target ID to 3...");
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_DEST_ID, 3)?;
    
    // Arbitrate
    println!("Arbitrating bus...");
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_SEQUENCE, MESH_CMD_ARBITRATE)?;
    
    // Select target
    println!("Selecting target...");
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_SEQUENCE, MESH_CMD_SELECT)?;
    
    // Build INQUIRY command (6 bytes)
    println!("Sending INQUIRY command...");
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, SCSI_CMD_INQUIRY)?;  // Opcode
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, 0x00)?;  // LUN / reserved
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, 0x00)?;  // Page code
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, 0x00)?;  // Reserved
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, 0x24)?;  // Allocation length (36 bytes)
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, 0x00)?;  // Control
    
    // Execute command
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_SEQUENCE, MESH_CMD_COMMAND)?;
    
    // Read data
    println!("Reading INQUIRY response...");
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_SEQUENCE, MESH_CMD_DATA_IN)?;
    
    // Check FIFO count
    let fifo_count = memory.read_u8(MESH_BASE_ADDR + MESH_REG_FIFO_COUNT)?;
    println!("FIFO contains {} bytes", fifo_count);
    
    // Read response data from FIFO
    if fifo_count > 0 {
        let mut response = Vec::new();
        for _ in 0..fifo_count.min(36) {
            let byte = memory.read_u8(MESH_BASE_ADDR + MESH_REG_FIFO)?;
            response.push(byte);
        }
        
        println!();
        println!("INQUIRY Response ({} bytes):", response.len());
        println!("  Device type: 0x{:02X}", response[0] & 0x1F);
        println!("  Removable: {}", (response[1] & 0x80) != 0);
        
        if response.len() >= 36 {
            let vendor = String::from_utf8_lossy(&response[8..16]).trim().to_string();
            let product = String::from_utf8_lossy(&response[16..32]).trim().to_string();
            let revision = String::from_utf8_lossy(&response[32..36]).trim().to_string();
            
            println!("  Vendor: {}", vendor);
            println!("  Product: {}", product);
            println!("  Revision: {}", revision);
        }
        
        println!("✅ INQUIRY command successful");
    } else {
        println!("❌ No data returned");
    }
    
    println!();
    println!("==================================================");
    println!("Test 2: SCSI READ CAPACITY Command");
    println!("==================================================");
    
    // Arbitrate
    println!("Arbitrating bus...");
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_SEQUENCE, MESH_CMD_ARBITRATE)?;
    
    // Select target
    println!("Selecting target...");
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_SEQUENCE, MESH_CMD_SELECT)?;
    
    // Build READ CAPACITY command (10 bytes)
    println!("Sending READ CAPACITY command...");
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, SCSI_CMD_READ_CAPACITY)?;  // Opcode
    for _ in 0..9 {
        memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, 0x00)?;  // Reserved/unused
    }
    
    // Execute command
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_SEQUENCE, MESH_CMD_COMMAND)?;
    
    // Read data
    println!("Reading READ CAPACITY response...");
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_SEQUENCE, MESH_CMD_DATA_IN)?;
    
    // Check FIFO count
    let fifo_count = memory.read_u8(MESH_BASE_ADDR + MESH_REG_FIFO_COUNT)?;
    println!("FIFO contains {} bytes", fifo_count);
    
    if fifo_count >= 8 {
        let mut response = Vec::new();
        for _ in 0..8 {
            let byte = memory.read_u8(MESH_BASE_ADDR + MESH_REG_FIFO)?;
            response.push(byte);
        }
        
        let last_lba = u32::from_be_bytes([response[0], response[1], response[2], response[3]]);
        let block_size = u32::from_be_bytes([response[4], response[5], response[6], response[7]]);
        let total_blocks = last_lba + 1;
        let total_size_mb = (total_blocks as u64 * block_size as u64) / (1024 * 1024);
        
        println!();
        println!("READ CAPACITY Response:");
        println!("  Last LBA: {} (0x{:08X})", last_lba, last_lba);
        println!("  Block size: {} bytes", block_size);
        println!("  Total blocks: {}", total_blocks);
        println!("  Total size: {} MB", total_size_mb);
        println!("✅ READ CAPACITY command successful");
    } else {
        println!("❌ Insufficient data returned");
    }
    
    println!();
    println!("==================================================");
    println!("Test 3: SCSI READ(10) Command");
    println!("==================================================");
    
    // Arbitrate
    println!("Arbitrating bus...");
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_SEQUENCE, MESH_CMD_ARBITRATE)?;
    
    // Select target
    println!("Selecting target...");
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_SEQUENCE, MESH_CMD_SELECT)?;
    
    // Build READ(10) command to read LBA 0
    println!("Sending READ(10) command (LBA=0, length=1)...");
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, SCSI_CMD_READ10)?;  // Opcode
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, 0x00)?;  // LUN / flags
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, 0x00)?;  // LBA byte 0
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, 0x00)?;  // LBA byte 1
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, 0x00)?;  // LBA byte 2
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, 0x00)?;  // LBA byte 3 (LBA = 0)
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, 0x00)?;  // Reserved
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, 0x00)?;  // Transfer length MSB
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, 0x01)?;  // Transfer length LSB (1 block)
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_FIFO, 0x00)?;  // Control
    
    // Execute command
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_SEQUENCE, MESH_CMD_COMMAND)?;
    
    // Read data
    println!("Reading block data...");
    memory.write_u8(MESH_BASE_ADDR + MESH_REG_SEQUENCE, MESH_CMD_DATA_IN)?;
    
    // Check FIFO count
    let fifo_count = memory.read_u8(MESH_BASE_ADDR + MESH_REG_FIFO_COUNT)?;
    println!("FIFO contains {} bytes", fifo_count);
    
    if fifo_count > 0 {
        let bytes_to_read = fifo_count.min(64); // Read first 64 bytes
        let mut data = Vec::new();
        for _ in 0..bytes_to_read {
            let byte = memory.read_u8(MESH_BASE_ADDR + MESH_REG_FIFO)?;
            data.push(byte);
        }
        
        println!();
        println!("First {} bytes of block 0:", data.len());
        for (i, chunk) in data.chunks(16).enumerate() {
            print!("  {:04X}:", i * 16);
            for byte in chunk {
                print!(" {:02X}", byte);
            }
            println!();
        }
        println!("✅ READ(10) command successful");
    } else {
        println!("❌ No data returned");
    }
    
    println!();
    println!("==================================================");
    println!("All SCSI tests completed!");
    println!("==================================================");
    
    Ok(())
}
