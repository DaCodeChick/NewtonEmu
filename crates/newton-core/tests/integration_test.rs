// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Integration tests for the full emulator

use newton_core::{Emulator, EmulatorMode, EmulatorConfig};
use newton_core::memory::MemoryInterface;

#[test]
fn test_emulator_creation() {
    let config = EmulatorConfig::default();
    let emulator = Emulator::new(config).unwrap();
    
    assert!(!emulator.is_running());
}

#[test]
fn test_emulator_with_modes() {
    let config = EmulatorConfig::default();
    
    // Test single-threaded mode
    let emu_st = Emulator::with_mode(config.clone(), EmulatorMode::SingleThreaded).unwrap();
    assert_eq!(emu_st.mode(), EmulatorMode::SingleThreaded);
    
    // Test multi-threaded mode
    let emu_mt = Emulator::with_mode(config, EmulatorMode::MultiThreaded).unwrap();
    assert_eq!(emu_mt.mode(), EmulatorMode::MultiThreaded);
}

#[test]
fn test_simple_program_execution() {
    // Create emulator in single-threaded mode for easier testing
    let config = EmulatorConfig::default();
    let mut emulator = Emulator::with_mode(config, EmulatorMode::SingleThreaded).unwrap();
    
    // Write a simple program to RAM (not ROM)
    // We'll write at address 0x1000 and set PC there
    // addi r3, r0, 42     // r3 = 0 + 42
    // addi r4, r3, 8      // r4 = r3 + 8 = 50
    // add  r5, r3, r4     // r5 = r3 + r4 = 92
    
    {
        let memory = emulator.memory();
        
        // addi r3, r0, 42  =>  38 60 00 2A
        memory.write_u32(0x1000, 0x3860_002A).unwrap();
        
        // addi r4, r3, 8   =>  38 83 00 08
        memory.write_u32(0x1004, 0x3883_0008).unwrap();
        
        // add r5, r3, r4   =>  7C A3 22 14
        memory.write_u32(0x1008, 0x7CA3_2214).unwrap();
    }
    
    // Set PC to 0x1000 (our program start)
    if let Some(cpu) = emulator.cpu_mut() {
        cpu.registers.pc = 0x1000;
    }
    
    // Execute the three instructions
    emulator.step().unwrap();
    emulator.step().unwrap();
    emulator.step().unwrap();
    
    // Check results
    if let Some(cpu) = emulator.cpu() {
        assert_eq!(cpu.registers.gpr[3], 42);
        assert_eq!(cpu.registers.gpr[4], 50);
        assert_eq!(cpu.registers.gpr[5], 92);
        assert_eq!(cpu.registers.pc, 0x100C);
    } else {
        panic!("CPU not available in single-threaded mode");
    }
}

#[test]
fn test_branch_execution() {
    let config = EmulatorConfig::default();
    let mut emulator = Emulator::with_mode(config, EmulatorMode::SingleThreaded).unwrap();
    
    // Write a program that uses branches
    // addi r3, r0, 1      // r3 = 1
    // cmpwi r3, 1         // Compare r3 with 1
    // beq label           // Branch if equal
    // addi r4, r0, 99     // r4 = 99 (should be skipped)
    // label:
    // addi r5, r0, 100    // r5 = 100
    
    {
        let memory = emulator.memory();
        
        // addi r3, r0, 1   =>  38 60 00 01
        memory.write_u32(0x2000, 0x3860_0001).unwrap();
        
        // cmpwi cr0, r3, 1  =>  2C 03 00 01
        memory.write_u32(0x2004, 0x2C03_0001).unwrap();
        
        // beq +8 (skip one instruction) =>  41 82 00 08
        memory.write_u32(0x2008, 0x4182_0008).unwrap();
        
        // addi r4, r0, 99  =>  38 80 00 63
        memory.write_u32(0x200C, 0x3880_0063).unwrap();
        
        // addi r5, r0, 100 =>  38 A0 00 64
        memory.write_u32(0x2010, 0x38A0_0064).unwrap();
    }
    
    if let Some(cpu) = emulator.cpu_mut() {
        cpu.registers.pc = 0x2000;
    }
    
    // Execute: addi, cmpwi, beq (taken), addi r5
    emulator.step().unwrap();  // addi r3, r0, 1
    emulator.step().unwrap();  // cmpwi r3, 1
    emulator.step().unwrap();  // beq (taken, jumps to 0x2010)
    emulator.step().unwrap();  // addi r5, r0, 100
    
    if let Some(cpu) = emulator.cpu() {
        assert_eq!(cpu.registers.gpr[3], 1);
        assert_eq!(cpu.registers.gpr[4], 0);  // Should not be set
        assert_eq!(cpu.registers.gpr[5], 100);
    }
}

#[test]
fn test_load_store_integration() {
    let config = EmulatorConfig::default();
    let mut emulator = Emulator::with_mode(config, EmulatorMode::SingleThreaded).unwrap();
    
    // Write a program that tests load/store
    // li r3, 0x5000       // r3 = 0x5000 (address)
    // li r4, 0xDEADBEEF   // r4 = 0xDEADBEEF (value)
    // stw r4, 0(r3)       // Store r4 to memory[r3]
    // li r5, 0            // r5 = 0
    // lwz r5, 0(r3)       // Load from memory[r3] to r5
    
    {
        let memory = emulator.memory();
        
        // li r3, 0x5000 => addis r3, r0, 0 + ori r3, r3, 0x5000
        // addis r3, r0, 0  =>  3C 60 00 00
        memory.write_u32(0x3000, 0x3C60_0000).unwrap();
        // ori r3, r3, 0x5000  =>  60 63 50 00
        memory.write_u32(0x3004, 0x6063_5000).unwrap();
        
        // li r4, 0xDEADBEEF => addis r4, r0, 0xDEAD + ori r4, r4, 0xBEEF
        // addis r4, r0, 0xDEAD  =>  3C 80 DE AD
        memory.write_u32(0x3008, 0x3C80_DEAD).unwrap();
        // ori r4, r4, 0xBEEF  =>  60 84 BE EF
        memory.write_u32(0x300C, 0x6084_BEEF).unwrap();
        
        // stw r4, 0(r3)  =>  90 83 00 00
        memory.write_u32(0x3010, 0x9083_0000).unwrap();
        
        // li r5, 0 => addi r5, r0, 0  =>  38 A0 00 00
        memory.write_u32(0x3014, 0x38A0_0000).unwrap();
        
        // lwz r5, 0(r3)  =>  80 A3 00 00
        memory.write_u32(0x3018, 0x80A3_0000).unwrap();
    }
    
    if let Some(cpu) = emulator.cpu_mut() {
        cpu.registers.pc = 0x3000;
    }
    
    // Execute all instructions
    for _ in 0..7 {
        emulator.step().unwrap();
    }
    
    if let Some(cpu) = emulator.cpu() {
        assert_eq!(cpu.registers.gpr[3], 0x5000);
        assert_eq!(cpu.registers.gpr[4], 0xDEAD_BEEF);
        assert_eq!(cpu.registers.gpr[5], 0xDEAD_BEEF);
    }
    
    // Verify the value was actually written to memory
    assert_eq!(emulator.memory().read_u32(0x5000).unwrap(), 0xDEAD_BEEF);
}
