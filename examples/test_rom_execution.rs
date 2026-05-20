// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Test ROM entry point execution
//!
//! This test verifies that:
//! - ROM loads correctly
//! - CPU starts at ROM entry point
//! - Instructions can be fetched and decoded
//! - Memory access works

use newton_core::emulator::Emulator;
use newton_core::config::{EmulatorConfig, CpuConfig, CpuModel, MemoryConfig, DisplayConfig};
use std::path::PathBuf;
use tracing_subscriber;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();
    
    println!("=== ROM Entry Point Execution Test ===\n");
    
    // Find a ROM file
    let rom_path = find_rom().expect("No ROM file found in roms/ directory");
    println!("Using ROM: {}\n", rom_path.display());
    
    // Create emulator configuration
    let config = EmulatorConfig {
        cpu: CpuConfig {
            model: CpuModel::G4,
            clock_speed: 500,
            enable_jit: false,
        },
        memory: MemoryConfig {
            ram_size_mb: 256,
            rom_path: Some(rom_path),
        },
        display: DisplayConfig {
            width: 800,
            height: 600,
            color_depth: 32,
        },
    };
    
    // Create emulator
    let mut emulator = Emulator::new(config).expect("Failed to create emulator");
    
    // Reset to initialize
    println!("Resetting emulator...");
    emulator.reset();
    
    // Get initial CPU state
    if let Some(cpu) = emulator.cpu() {
        println!("\nInitial CPU state:");
        println!("  PC  = 0x{:08X}", cpu.registers.pc);
        println!("  LR  = 0x{:08X}", cpu.registers.lr);
        println!("  r1  = 0x{:08X} (stack pointer)", cpu.registers.gpr[1]);
        println!("  r2  = 0x{:08X} (TOC/globals)", cpu.registers.gpr[2]);
        println!("  r5  = 0x{:08X} (OpenFirmware entry)", cpu.registers.gpr[5]);
    }
    
    // Execute a few instructions
    println!("\nExecuting 10 instructions...\n");
    
    for i in 0..10 {
        if let Some(cpu) = emulator.cpu() {
            let pc = cpu.registers.pc;
            
            // Try to read the instruction from memory
            let memory = emulator.memory();
            match newton_cpu::MemoryInterface::read_u32(&*memory, pc) {
                Ok(instr_word) => {
                    // Decode instruction
                    match newton_cpu::decode_instruction(instr_word) {
                        Ok(instr) => {
                            println!("[{}] 0x{:08X}: {:08X}  {:?}", i, pc, instr_word, instr);
                        }
                        Err(e) => {
                            println!("[{}] 0x{:08X}: {:08X}  (decode error: {})", i, pc, instr_word, e);
                        }
                    }
                }
                Err(e) => {
                    println!("[{}] 0x{:08X}: (read error: {})", i, pc, e);
                    break;
                }
            }
        }
        
        // Execute one instruction
        match emulator.step() {
            Ok(_) => {}
            Err(e) => {
                println!("\nExecution error: {}", e);
                break;
            }
        }
    }
    
    // Final CPU state
    if let Some(cpu) = emulator.cpu() {
        println!("\nFinal CPU state:");
        println!("  PC  = 0x{:08X}", cpu.registers.pc);
        println!("  LR  = 0x{:08X}", cpu.registers.lr);
        println!("  r3  = 0x{:08X}", cpu.registers.gpr[3]);
        println!("  r4  = 0x{:08X}", cpu.registers.gpr[4]);
    }
    
    println!("\n=== Test Complete ===");
}

fn find_rom() -> Option<PathBuf> {
    let roms_dir = PathBuf::from("roms");
    
    if !roms_dir.exists() {
        return None;
    }
    
    // Find first .rom file
    for entry in std::fs::read_dir(roms_dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("rom") {
            return Some(path);
        }
    }
    
    None
}
