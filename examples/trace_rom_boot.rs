// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use newton_core::{Emulator};
use newton_core::config::{EmulatorConfig, CpuConfig, MemoryConfig, DisplayConfig, CpuModel};
use newton_utils::Result;
use std::collections::HashMap;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== ROM Boot Trace ===\n");

    let rom_path = "roms/1999-04-05 - Mac OS ROM 1.4.rom";
    println!("Using ROM: {}\n", rom_path);

    let config = EmulatorConfig {
        cpu: CpuConfig {
            model: CpuModel::G4,
            clock_speed: 450,
            enable_jit: false,
        },
        memory: MemoryConfig {
            ram_size_mb: 256,
            rom_path: Some(rom_path.into()),
        },
        display: DisplayConfig {
            width: 800,
            height: 600,
            color_depth: 32,
        },
    };

    let mut emu = Emulator::new(config)?;
    emu.reset();

    println!("Tracing first 1000 instructions...\n");

    let mut pc_counts: HashMap<u32, usize> = HashMap::new();
    let mut last_pc = 0;
    let mut stuck_count = 0;
    const MAX_STUCK: usize = 100;

    for i in 0..100_000 {
        let cpu = emu.cpu_mut().unwrap();
        let pc = cpu.registers.pc;
        
        // Count PC frequency
        *pc_counts.entry(pc).or_insert(0) += 1;
        
        // Detect if we're stuck in a tight loop
        if pc == last_pc {
            stuck_count += 1;
            if stuck_count > MAX_STUCK {
                println!("\n⚠️  STUCK in tight loop at PC=0x{:08X} (repeated {} times)", pc, stuck_count);
                println!("   This might be a spin-wait or unimplemented hardware access");
                break;
            }
        } else {
            stuck_count = 0;
        }
        last_pc = pc;
        
        // Log every 1000th instruction
        if i % 1000 == 0 {
            println!("[{:6}] PC=0x{:08X}", i, pc);
        }
        
        // Log framebuffer writes
        if pc >= 0x00800000 && pc < 0x00A00000 {
            println!("✓ Framebuffer access at PC=0x{:08X}!", pc);
        }
        
        match emu.step() {
            Ok(_) => {}
            Err(e) => {
                println!("\n❌ Error at instruction {}: {}", i, e);
                println!("   PC = 0x{:08X}", pc);
                break;
            }
        }
    }

    println!("\n=== Execution Analysis ===");
    println!("Total unique PCs visited: {}", pc_counts.len());
    
    // Find hot spots
    let mut sorted: Vec<_> = pc_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    
    println!("\nTop 10 hot spots:");
    for (pc, count) in sorted.iter().take(10) {
        println!("  0x{:08X}: {} executions", pc, count);
    }

    Ok(())
}
