// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Simple ROM boot test
//!
//! This example loads a Mac OS ROM and executes the first few instructions,
//! demonstrating that the CPU and memory integration is working.

use anyhow::Result;
use newton_core::{Emulator, EmulatorConfig, EmulatorMode};
use newton_core::memory::MemoryInterface;
use newton_utils::logging;

fn main() -> Result<()> {
    // Initialize logging
    logging::init();
    
    println!("=== NewtonEmu ROM Boot Test ===\n");
    
    // Use a NewWorld ROM
    let rom_path = "roms/1999-09-27 - Mac OS ROM 3.0.rom";
    
    println!("Loading ROM: {}", rom_path);
    
    // Create emulator config with ROM
    let mut config = EmulatorConfig::default();
    config.memory.rom_path = Some(rom_path.to_string());
    
    // Use single-threaded mode for easier debugging
    let mut emulator = Emulator::with_mode(config, EmulatorMode::SingleThreaded)?;
    
    println!("ROM loaded successfully!");
    println!("ROM base address: 0x{:08X}", emulator.memory().rom().map(|r| r.base_address()).unwrap_or(0));
    println!("ROM size: {} bytes\n", emulator.memory().rom().map(|r| r.size()).unwrap_or(0));
    
    // Reset CPU to ROM entry point
    emulator.reset();
    
    if let Some(cpu) = emulator.cpu() {
        println!("Initial CPU state:");
        println!("  PC:  0x{:08X}", cpu.registers.pc);
        println!("  MSR: 0x{:08X}", cpu.registers.msr.bits());
        println!("  r1 (SP): 0x{:08X}", cpu.registers.gpr[1]);
        println!("  r5 (OF entry): 0x{:08X}\n", cpu.registers.gpr[5]);
    }
    
    // Execute first 10 instructions
    println!("Executing first 10 instructions...\n");
    
    for i in 0..10 {
        if let Some(cpu) = emulator.cpu() {
            // Read instruction at PC
            let pc = cpu.registers.pc;
            if let Ok(instr_word) = emulator.memory().read_u32(pc) {
                println!("Step {}: PC=0x{:08X}, Instruction=0x{:08X}", i + 1, pc, instr_word);
            }
        }
        
        match emulator.step() {
            Ok(_) => {
                // Success - continue
            }
            Err(e) => {
                eprintln!("  Error: {}", e);
                if let Some(cpu) = emulator.cpu() {
                    eprintln!("  Final PC: 0x{:08X}", cpu.registers.pc);
                }
                break;
            }
        }
    }
    
    println!("\nFinal CPU state:");
    if let Some(cpu) = emulator.cpu() {
        println!("  PC:  0x{:08X}", cpu.registers.pc);
        println!("  LR:  0x{:08X}", cpu.registers.lr);
        println!("  CTR: 0x{:08X}", cpu.registers.ctr);
        println!("  r1 (SP): 0x{:08X}", cpu.registers.gpr[1]);
        println!("  r3: 0x{:08X}", cpu.registers.gpr[3]);
        println!("  CR: 0x{:08X}", cpu.registers.cr.bits());
    }
    
    println!("\n=== ROM Boot Test Complete ===");
    
    Ok(())
}
