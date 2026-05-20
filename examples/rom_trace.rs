// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! ROM execution tracer
//!
//! Traces ROM execution with instruction disassembly and register dumps.

use anyhow::Result;
use newton_core::{Emulator, EmulatorConfig, EmulatorMode};
use newton_core::memory::MemoryInterface;
use newton_cpu::decoder::decode_instruction;
use newton_utils::logging;

fn main() -> Result<()> {
    // Initialize logging
    logging::init();
    
    println!("=== NewtonEmu ROM Execution Tracer ===\n");
    
    // Use a NewWorld ROM
    let rom_path = "roms/1999-09-27 - Mac OS ROM 3.0.rom";
    
    println!("Loading ROM: {}", rom_path);
    
    // Create emulator config with ROM
    let mut config = EmulatorConfig::default();
    config.memory.rom_path = Some(rom_path.into());
    
    // Use single-threaded mode for tracing
    let mut emulator = Emulator::with_mode(config, EmulatorMode::SingleThreaded)?;
    
    println!("ROM loaded successfully!\n");
    
    // Reset CPU to ROM entry point
    emulator.reset();
    
    // Get number of steps from args or default to 50
    let num_steps: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);
    
    println!("Tracing {} steps...\n", num_steps);
    println!("{:<6} {:<10} {:<10} {:<30} {:<20}", "Step", "PC", "Opcode", "Instruction", "Notes");
    println!("{}", "-".repeat(90));
    
    let mut branches = 0;
    let mut loads = 0;
    let mut stores = 0;
    let mut arithmetic = 0;
    let mut unknown = 0;
    
    for step in 0..num_steps {
        if let Some(cpu) = emulator.cpu() {
            let pc = cpu.registers.pc;
            
            // Read and decode instruction
            if let Ok(instr_word) = emulator.memory().read_u32(pc) {
                let instr_result = decode_instruction(instr_word);
                
                let (instr_str, category) = match &instr_result {
                    Ok(instr) => {
                        let cat = categorize_instruction(instr);
                        match cat {
                            "branch" => branches += 1,
                            "load" => loads += 1,
                            "store" => stores += 1,
                            "arith" => arithmetic += 1,
                            "unknown" => unknown += 1,
                            _ => {}
                        }
                        (format!("{:?}", instr), cat)
                    }
                    Err(_) => {
                        unknown += 1;
                        ("INVALID".to_string(), "unknown")
                    }
                };
                
                // Truncate long instruction strings
                let instr_display = if instr_str.len() > 28 {
                    format!("{}...", &instr_str[..25])
                } else {
                    instr_str
                };
                
                println!("{:<6} 0x{:08X} 0x{:08X} {:<30} {}", 
                    step + 1, pc, instr_word, instr_display, category);
            }
        }
        
        // Execute
        if let Err(e) = emulator.step() {
            eprintln!("\nExecution stopped: {}", e);
            break;
        }
    }
    
    println!("\n{}", "=".repeat(90));
    println!("Execution Statistics:");
    println!("  Branch instructions: {}", branches);
    println!("  Load instructions:   {}", loads);
    println!("  Store instructions:  {}", stores);
    println!("  Arithmetic/Logical:  {}", arithmetic);
    println!("  Unknown/Data:        {}", unknown);
    
    if let Some(cpu) = emulator.cpu() {
        println!("\nFinal CPU State:");
        println!("  PC:  0x{:08X}", cpu.registers.pc);
        println!("  LR:  0x{:08X}", cpu.registers.lr);
        println!("  CTR: 0x{:08X}", cpu.registers.ctr);
        println!("  CR:  0x{:08X}", cpu.registers.cr.bits());
        println!("  r1:  0x{:08X} (stack pointer)", cpu.registers.gpr[1]);
        println!("  r3:  0x{:08X}", cpu.registers.gpr[3]);
        println!("  r4:  0x{:08X}", cpu.registers.gpr[4]);
        println!("  r5:  0x{:08X} (OF entry)", cpu.registers.gpr[5]);
    }
    
    Ok(())
}

fn categorize_instruction(instr: &newton_cpu::decoder::Instruction) -> &'static str {
    use newton_cpu::decoder::Instruction::*;
    
    match instr {
        B { .. } | Bc { .. } | Bcctr { .. } | Bclr { .. } => "branch",
        
        Lwz { .. } | Lwzu { .. } | Lbz { .. } | Lhz { .. } | Lha { .. } |
        Lmw { .. } | Lswi { .. } | Lfd { .. } | Lfs { .. } |
        Lwzx { .. } | Lbzx { .. } | Lhzx { .. } | Lhax { .. } |
        Lwbrx { .. } | Lhbrx { .. } | Lwarx { .. } => "load",
        
        Stw { .. } | Stwu { .. } | Stb { .. } | Sth { .. } |
        Stmw { .. } | Stswi { .. } | Stfd { .. } | Stfs { .. } |
        Stwx { .. } | Stbx { .. } | Sthx { .. } |
        Stwbrx { .. } | Sthbrx { .. } | Stwcx { .. } => "store",
        
        Add { .. } | Addc { .. } | Adde { .. } | Addi { .. } | Addic { .. } |
        Addis { .. } | Subf { .. } | Subfc { .. } | Mulli { .. } | Mullw { .. } |
        Divw { .. } | And { .. } | Or { .. } | Xor { .. } | Nand { .. } | Nor { .. } |
        Slw { .. } | Srw { .. } | Sraw { .. } | Rlwinm { .. } => "arith",
        
        Mfspr { .. } | Mtspr { .. } | Mfcr { .. } | Mtcrf { .. } |
        Mfmsr { .. } | Mtmsr { .. } | Mftb { .. } => "spr",
        
        Cmp { .. } | Cmpi { .. } | Cmpl { .. } | Cmpli { .. } => "compare",
        
        Nop => "nop",
        Sync | Isync | Eieio => "sync",
        
        Unknown { .. } => "unknown",
        
        _ => "other"
    }
}
