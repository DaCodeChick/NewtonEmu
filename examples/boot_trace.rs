// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Boot trace example - load a ROM and trace initial execution
//!
//! Usage: cargo run --example boot_trace -- <rom_file>

use anyhow::Result;
use newton_core::{Emulator, EmulatorConfig, EmulatorMode};
use newton_cpu::MemoryInterface;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .with_line_number(true)
        .init();

    // Get ROM path from command line
    let args: Vec<String> = env::args().collect();
    let rom_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        // Default to a common ROM
        PathBuf::from("roms/1999-09-17 - Mac OS ROM 2.5.1.rom")
    };

    eprintln!("==================================================");
    eprintln!("NewtonEmu Boot Trace");
    eprintln!("==================================================");
    eprintln!("ROM file: {}", rom_path.display());
    eprintln!();

    // Create emulator config
    let mut config = EmulatorConfig::default();
    config.memory.rom_path = Some(rom_path.to_string_lossy().to_string());
    config.memory.ram_size_mb = 128;

    // Create emulator in single-threaded mode for easier debugging
    let mut emulator = Emulator::with_mode(config, EmulatorMode::SingleThreaded)?;

    // Reset to initialize boot state
    tracing::info!("Resetting emulator...");
    emulator.reset();

    // Get initial CPU state
    if let Some(cpu) = emulator.cpu() {
        dbg!(&cpu.registers.pc);
        dbg!(&cpu.registers.msr.bits());
        dbg!(&cpu.registers.gpr[1]); // SP
        dbg!(&cpu.registers.gpr[5]); // OF
        dbg!(&cpu.registers.lr);      // Link Register
    }

    // Execute first 10000 instructions and trace them
    eprintln!("\nExecuting first 10000 instructions:");
    eprintln!("--------------------------------------------------");

    for i in 0..10000 {
        if let Some(cpu) = emulator.cpu() {
            let pc = cpu.registers.pc;
            
            // Read the instruction at PC
            let instr_word = match emulator.memory().read_u32(pc) {
                Ok(word) => word,
                Err(e) => {
                    tracing::error!("Failed to read instruction at PC=0x{:08X}: {}", pc, e);
                    break;
                }
            };

            // Decode instruction for display
            let decoded = match newton_cpu::decode_instruction(instr_word) {
                Ok(instr) => format!("{:?}", instr),
                Err(_) => format!("INVALID(0x{:08X})", instr_word),
            };

            // Track key instructions around the problematic area
            let opcode = instr_word >> 26;
            let xo = (instr_word >> 1) & 0x3FF;
            let _rt = (instr_word >> 21) & 0x1F;
            let ra = (instr_word >> 16) & 0x1F;
            
            // Show simplified trace
            if i % 100 == 0 || i < 100 {
                eprintln!("{:4}: PC=0x{:08X}  0x{:08X}  {}", i, pc, instr_word, decoded);
            }
            
            // Show detailed info when calling or returning from stub function
            if pc == 0x1000 || pc == 0x3000 {
                // Entering OpenFirmware or special stub - show registers
                eprintln!("{:4}: PC=0x{:08X}  0x{:08X}  {} [r3=0x{:08X} r4=0x{:08X} r5=0x{:08X} LR=0x{:08X}]",
                    i, pc, instr_word, decoded,
                    cpu.registers.gpr[3], cpu.registers.gpr[4], cpu.registers.gpr[5], cpu.registers.lr);
            } else if i >= 65 && i <= 85 {
                // Initial boot sequence details
                if opcode == 32 {
                    // lwz - show source address and register values
                    let d = ((instr_word & 0xFFFF) as i16) as i32;
                    let addr = if ra == 0 { 0 } else { cpu.registers.gpr[ra as usize] as i32 };
                    let ea = (addr.wrapping_add(d)) as u32;
                    let value = emulator.memory().read_u32(ea).unwrap_or(0xDEADBEEF);
                    eprintln!("{:4}: PC=0x{:08X}  0x{:08X}  {} [r{}=0x{:08X}, [0x{:08X}]=0x{:08X}]", 
                        i, pc, instr_word, decoded, ra, cpu.registers.gpr[ra as usize], ea, value);
                } else if opcode == 19 && xo == 528 {
                    // bcctr - show CTR value
                    eprintln!("{:4}: PC=0x{:08X}  0x{:08X}  {} [CTR=0x{:08X}]", i, pc, instr_word, decoded, cpu.registers.ctr);
                } else if opcode == 31 && xo == 467 {
                    // mtspr CTR - show value being set
                    let rs = (instr_word >> 21) & 0x1F;
                    let spr = ((instr_word >> 16) & 0x1F) | ((instr_word >> 6) & 0x3E0);
                    if spr == 9 {
                        eprintln!("{:4}: PC=0x{:08X}  0x{:08X}  {} [r{}=0x{:08X}->CTR]", i, pc, instr_word, decoded, rs, cpu.registers.gpr[rs as usize]);
                    } else {
                        eprintln!("{:4}: PC=0x{:08X}  0x{:08X}  {}", i, pc, instr_word, decoded);
                    }
                } else {
                    eprintln!("{:4}: PC=0x{:08X}  0x{:08X}  {}", i, pc, instr_word, decoded);
                }
            } else {
                eprintln!("{:4}: PC=0x{:08X}  0x{:08X}  {}", i, pc, instr_word, decoded);
            }
        }

        // Step one instruction
        if let Err(e) = emulator.step() {
            tracing::error!("Error at instruction {}: {}", i, e);
            if let Some(cpu) = emulator.cpu() {
                dbg!(&cpu.registers.pc);
                dbg!(&cpu.registers.msr.bits());
                dbg!(&cpu.registers.lr);
                dbg!(&cpu.registers.ctr);
                dbg!(&cpu.registers.gpr[1]);
                dbg!(&cpu.registers.gpr[3]);
                dbg!(&cpu.registers.gpr[4]);
                dbg!(&cpu.registers.gpr[5]);
            }
            break;
        }

        // Check for OpenFirmware calls
        if let Some(cpu) = emulator.cpu() {
            if cpu.registers.pc == 0x3000 {
                tracing::info!("OpenFirmware client interface call detected");
                dbg!(&cpu.registers.gpr[3]); // args ptr
            }
        }
    }

    eprintln!("\n==================================================");
    eprintln!("Trace complete");
    eprintln!("==================================================");

    Ok(())
}
