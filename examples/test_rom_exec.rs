// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Test execution from a specific ROM address
//!
//! This example loads a Mac OS ROM and executes code from a specified address,
//! useful for testing execution of specific ROM code sections.

use anyhow::Result;
use newton_core::{Emulator, EmulatorConfig, EmulatorMode};
use newton_core::memory::MemoryInterface;
use newton_utils::logging;

fn main() -> Result<()> {
    // Initialize logging
    logging::init();
    
    let args: Vec<String> = std::env::args().collect();
    let start_addr = if args.len() > 1 {
        u32::from_str_radix(&args[1].trim_start_matches("0x"), 16)?
    } else {
        0xFFC04800  // Default: first 100% code section
    };
    
    let num_steps: usize = if args.len() > 2 {
        args[2].parse()?
    } else {
        1000
    };
    
    println!("=== NewtonEmu ROM Execution Test ===\n");
    println!("Starting address: 0x{:08X}", start_addr);
    println!("Steps to execute: {}\n", num_steps);
    
    // Use a NewWorld ROM
    let rom_path = "roms/1999-09-27 - Mac OS ROM 3.0.rom";
    
    // Create emulator config with ROM
    let mut config = EmulatorConfig::default();
    config.memory.rom_path = Some(rom_path.to_string());
    
    // Use single-threaded mode for easier debugging
    let mut emulator = Emulator::with_mode(config, EmulatorMode::SingleThreaded)?;
    
    // Reset CPU
    emulator.reset();
    
    // Manually set PC to desired start address
    if let Some(cpu) = emulator.cpu_mut() {
        cpu.registers.pc = start_addr;
        println!("Set PC to 0x{:08X}\n", start_addr);
    }
    
    // Execute instructions
    println!("Executing {} instructions...\n", num_steps);
    
    let mut successful_steps = 0;
    let mut unknown_instrs = 0;
    let mut branch_count = 0;
    let mut load_count = 0;
    let mut store_count = 0;
    let mut arith_count = 0;
    
    // Keep track of last few instructions for debugging
    let mut last_instrs: Vec<(u32, u32)> = Vec::new();
    let mut last_lr = 0u32;
    let mut last_r1 = 0u32;
    
    for i in 0..num_steps {
        if let Some(cpu) = emulator.cpu() {
            let pc = cpu.registers.pc;
            
            // Read instruction
            if let Ok(instr_word) = emulator.memory().read_u32(pc) {
                let opcode = (instr_word >> 26) & 0x3F;
                
                // Classify instruction
                match opcode {
                    16 | 18 | 19 => branch_count += 1,  // bc, b, bclr/bcctr
                    32..=43 => load_count += 1,          // loads
                    44..=47 => store_count += 1,          // stores
                    7..=15 | 20..=30 => arith_count += 1, // arithmetic/logical
                    _ => {}
                }
                
                // Print some instructions
                if i < 20 {
                    println!("Step {}: PC=0x{:08X}, Instruction=0x{:08X}", i + 1, pc, instr_word);
                }
                
                // Print around step 88
                if i >= 85 && i <= 92 {
                    println!("Step {}: PC=0x{:08X}, Instr=0x{:08X}", i, pc, instr_word);
                    println!("  r0=0x{:08X} r1=0x{:08X} LR=0x{:08X}", 
                             cpu.registers.gpr[0], cpu.registers.gpr[1], cpu.registers.lr);
                    
                    // Show what's on the stack
                    let r1 = cpu.registers.gpr[1];
                    if let Ok(val0) = emulator.memory().read_u32(r1) {
                        if let Ok(val8) = emulator.memory().read_u32(r1 + 8) {
                            println!("  [r1+0]=0x{:08X} [r1+8]=0x{:08X}", val0, val8);
                        }
                    }
                }
                
                // Print around step 1110-1114 (where the second failure happens)
                if i >= 1108 && i <= 1115 {
                    println!("Step {}: PC=0x{:08X}, Instr=0x{:08X}", i, pc, instr_word);
                    println!("  r0=0x{:08X} r1=0x{:08X} LR=0x{:08X}", 
                             cpu.registers.gpr[0], cpu.registers.gpr[1], cpu.registers.lr);
                    
                    // Show what's on the stack
                    let r1 = cpu.registers.gpr[1];
                    if let Ok(val0) = emulator.memory().read_u32(r1) {
                        if let Ok(val8) = emulator.memory().read_u32(r1 + 8) {
                            println!("  [r1+0]=0x{:08X} [r1+8]=0x{:08X}", val0, val8);
                        }
                    }
                }
                
                // Print critical instructions near the jump
                if pc >= 0xFFC10590 && pc <= 0xFFC105B0 {
                    println!("Step {}: PC=0x{:08X}, Instr=0x{:08X}", i, pc, instr_word);
                    println!("  r0=0x{:08X} r12=0x{:08X} CTR=0x{:08X}", 
                             cpu.registers.gpr[0], cpu.registers.gpr[12], cpu.registers.ctr);
                    
                    let r12 = cpu.registers.gpr[12];
                    if r12 != 0 {
                        if let Ok(val0) = emulator.memory().read_u32(r12) {
                            if let Ok(val4) = emulator.memory().read_u32(r12 + 4) {
                                println!("  [r12+0]=0x{:08X} [r12+4]=0x{:08X}", val0, val4);
                            }
                        }
                    }
                }
                
                // Keep track of last 20 instructions
                last_instrs.push((pc, instr_word));
                if last_instrs.len() > 20 {
                    last_instrs.remove(0);
                }
                
                // Track LR changes
                if cpu.registers.lr != last_lr && i > 0 {
                    println!("  Step {}: LR changed: 0x{:08X} -> 0x{:08X}", i, last_lr, cpu.registers.lr);
                }
                last_lr = cpu.registers.lr;
                
                // Track r1 (stack pointer) changes
                if cpu.registers.gpr[1] != last_r1 && i > 0 {
                    println!("  Step {}: r1 changed: 0x{:08X} -> 0x{:08X}", i, last_r1, cpu.registers.gpr[1]);
                }
                last_r1 = cpu.registers.gpr[1];
            }
        }
        
        match emulator.step() {
            Ok(_) => {
                successful_steps += 1;
            }
            Err(e) => {
                if e.to_string().contains("Unknown instruction") {
                    unknown_instrs += 1;
                    if unknown_instrs > 50 {
                        eprintln!("\nToo many unknown instructions ({}), stopping.", unknown_instrs);
                        break;
                    }
                } else {
                    eprintln!("\nFatal error: {}", e);
                    if let Some(cpu) = emulator.cpu() {
                        eprintln!("Final PC: 0x{:08X}", cpu.registers.pc);
                    }
                    break;
                }
            }
        }
        
        // Check if we've hit the infinite loop stub (success!)
        if let Some(cpu) = emulator.cpu() {
            let pc = cpu.registers.pc;
            if pc == 0x1004 {
                println!("\nReached infinite loop stub at 0x1004 - ROM execution completed successfully!");
                println!("This means the ROM code finished and returned properly.\n");
                break;
            }
        }
        
        // Check if we've left valid space
        if let Some(cpu) = emulator.cpu() {
            let pc = cpu.registers.pc;
            // Allow execution in ROM or low RAM (0x1000-0x2000 for stubs)
            if pc < 0x1000 || (pc >= 0x2000 && pc < 0xFFC00000) || pc >= 0xFFE20000 {
                println!("\nExecution left valid space (PC = 0x{:08X})", pc);
                println!("\nLast 10 instructions before leaving:");
                for (idx, (pc_val, instr)) in last_instrs.iter().rev().take(10).rev().enumerate() {
                    println!("  -{}: PC=0x{:08X}, Instr=0x{:08X}", 10 - idx, pc_val, instr);
                }
                println!("\nStopping.");
                break;
            }
        }
    }
    
    println!("\n{}", "=".repeat(60));
    println!("Execution Statistics:");
    println!("  Successful steps:    {}", successful_steps);
    println!("  Unknown instructions:{}", unknown_instrs);
    println!("  Branch instructions: {}", branch_count);
    println!("  Load instructions:   {}", load_count);
    println!("  Store instructions:  {}", store_count);
    println!("  Arithmetic/Logical:  {}", arith_count);
    
    println!("\nFinal CPU State:");
    if let Some(cpu) = emulator.cpu() {
        println!("  PC:  0x{:08X}", cpu.registers.pc);
        println!("  LR:  0x{:08X}", cpu.registers.lr);
        println!("  CTR: 0x{:08X}", cpu.registers.ctr);
        println!("  CR:  0x{:08X}", cpu.registers.cr.bits());
        println!("  r1:  0x{:08X} (stack pointer)", cpu.registers.gpr[1]);
        println!("  r3:  0x{:08X}", cpu.registers.gpr[3]);
        println!("  r5:  0x{:08X} (OF entry)", cpu.registers.gpr[5]);
    }
    
    println!("\n=== Test Complete ===");
    
    Ok(())
}
