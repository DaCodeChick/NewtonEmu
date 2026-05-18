// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! ROM execution test
//! 
//! This example loads a Mac ROM file and executes the first few instructions.
//! Usage: cargo run --example rom_test -- <rom_file> [num_instructions]

use anyhow::Result;
use newton_core::{Emulator, EmulatorConfig, EmulatorMode};
use newton_core::memory::MemoryInterface;
use newton_utils::logging;
use std::env;

fn main() -> Result<()> {
    logging::init();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <rom_file> [num_instructions]", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} roms/my_rom.rom", args[0]);
        eprintln!("  {} roms/my_rom.rom 100", args[0]);
        std::process::exit(1);
    }
    
    let rom_path = &args[1];
    let num_instructions: usize = if args.len() > 2 {
        args[2].parse().unwrap_or(10)
    } else {
        10
    };
    
    tracing::info!("NewtonEmu ROM Test");
    tracing::info!("===================");
    tracing::info!("ROM file: {}", rom_path);
    tracing::info!("Instructions to execute: {}", num_instructions);
    tracing::info!("");
    
    // Create config with ROM path
    let mut config = EmulatorConfig::default();
    config.memory.rom_path = Some(rom_path.into());
    
    // Create emulator in single-threaded mode for easier debugging
    let mut emulator = Emulator::with_mode(config, EmulatorMode::SingleThreaded)?;
    
    tracing::info!("Emulator created successfully!");
    tracing::info!("ROM loaded at reset vector");
    tracing::info!("");
    
    // Reset CPU to start at reset vector
    emulator.reset();
    
    // For larger ROMs at 0xFFC00000, adjust PC if needed
    {
        let memory = emulator.memory();
        let needs_adjustment = memory.read_u32(0xFFF0_0100).is_err();
        let rom_base_ok = memory.read_u32(0xFFC0_0100).is_ok();
        drop(memory);
        
        if needs_adjustment && rom_base_ok {
            tracing::warn!("Cannot read from 0xFFF00100, trying ROM base + 0x100");
            if let Some(cpu) = emulator.cpu_mut() {
                tracing::info!("Setting PC to 0xFFC00100 (ROM base + 0x100)");
                cpu.registers.pc = 0xFFC0_0100;
            }
        }
    }
    
    if let Some(cpu) = emulator.cpu() {
        tracing::info!("Initial CPU state:");
        tracing::info!("  PC: 0x{:08X}", cpu.registers.pc);
        tracing::info!("  MSR: 0x{:08X}", cpu.registers.msr.bits());
        tracing::info!("  PVR: 0x{:08X}", cpu.registers.spr[newton_cpu::registers::spr::PVR]);
        tracing::info!("");
    }
    
    // Peek at first instruction
    if let Some(cpu) = emulator.cpu() {
        let pc = cpu.registers.pc;
        let memory = emulator.memory();
        if let Ok(first_instr) = memory.read_u32(pc) {
            tracing::info!("First instruction at 0x{:08X}: 0x{:08X}", pc, first_instr);
        }
    }
    
    tracing::info!("");
    tracing::info!("Executing {} instructions...", num_instructions);
    tracing::info!("---");
    
    // Execute instructions
    for i in 0..num_instructions {
        if let Some(cpu) = emulator.cpu() {
            let pc = cpu.registers.pc;
            
            // Read the instruction before executing
            let memory = emulator.memory();
            if let Ok(instr_word) = memory.read_u32(pc) {
                // Drop memory reference before stepping
                drop(memory);
                
                // Try to execute
                match emulator.step() {
                    Ok(()) => {
                        if let Some(cpu_after) = emulator.cpu() {
                            tracing::info!(
                                "{:3}: PC=0x{:08X} Instr=0x{:08X} -> PC=0x{:08X}",
                                i,
                                pc,
                                instr_word,
                                cpu_after.registers.pc
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "{:3}: PC=0x{:08X} Instr=0x{:08X} -> ERROR: {}",
                            i,
                            pc,
                            instr_word,
                            e
                        );
                        break;
                    }
                }
            } else {
                tracing::error!("Failed to read instruction at PC=0x{:08X}", pc);
                break;
            }
        }
    }
    
    tracing::info!("---");
    tracing::info!("");
    
    // Show final state
    if let Some(cpu) = emulator.cpu() {
        tracing::info!("Final CPU state:");
        tracing::info!("  PC: 0x{:08X}", cpu.registers.pc);
        tracing::info!("");
        tracing::info!("  GPRs:");
        for i in 0..32 {
            if cpu.registers.gpr[i] != 0 {
                tracing::info!("    r{}: 0x{:08X}", i, cpu.registers.gpr[i]);
            }
        }
        
        if cpu.registers.lr != 0 {
            tracing::info!("  LR: 0x{:08X}", cpu.registers.lr);
        }
        if cpu.registers.ctr != 0 {
            tracing::info!("  CTR: 0x{:08X}", cpu.registers.ctr);
        }
        if cpu.registers.cr.bits() != 0 {
            tracing::info!("  CR: 0x{:08X}", cpu.registers.cr.bits());
        }
    }
    
    Ok(())
}
