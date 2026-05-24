// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Simple 68k emulator test - executes a small program

use newton_m68k::{M68k, M68kModel, MemoryInterface};
use newton_utils::Result;

struct SimpleMemory {
    data: Vec<u8>,
}

impl SimpleMemory {
    fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }
    
    fn write_u16_direct(&mut self, addr: u32, value: u16) {
        let addr = addr as usize;
        self.data[addr] = (value >> 8) as u8;
        self.data[addr + 1] = value as u8;
    }
    
    fn write_u32_direct(&mut self, addr: u32, value: u32) {
        let addr = addr as usize;
        self.data[addr] = (value >> 24) as u8;
        self.data[addr + 1] = (value >> 16) as u8;
        self.data[addr + 2] = (value >> 8) as u8;
        self.data[addr + 3] = value as u8;
    }
}

impl MemoryInterface for SimpleMemory {
    fn read_u8(&self, addr: u32) -> Result<u8> {
        Ok(self.data[addr as usize])
    }
    
    fn read_u16(&self, addr: u32) -> Result<u16> {
        let addr = addr as usize;
        Ok(u16::from_be_bytes([self.data[addr], self.data[addr + 1]]))
    }
    
    fn read_u32(&self, addr: u32) -> Result<u32> {
        let addr = addr as usize;
        Ok(u32::from_be_bytes([
            self.data[addr],
            self.data[addr + 1],
            self.data[addr + 2],
            self.data[addr + 3],
        ]))
    }
    
    fn write_u8(&self, addr: u32, value: u8) -> Result<()> {
        // Unsafe interior mutability for simplicity
        unsafe {
            let ptr = self.data.as_ptr() as *mut u8;
            *ptr.add(addr as usize) = value;
        }
        Ok(())
    }
    
    fn write_u16(&self, addr: u32, value: u16) -> Result<()> {
        unsafe {
            let ptr = self.data.as_ptr() as *mut u8;
            *ptr.add(addr as usize) = (value >> 8) as u8;
            *ptr.add(addr as usize + 1) = value as u8;
        }
        Ok(())
    }
    
    fn write_u32(&self, addr: u32, value: u32) -> Result<()> {
        unsafe {
            let ptr = self.data.as_ptr() as *mut u8;
            *ptr.add(addr as usize) = (value >> 24) as u8;
            *ptr.add(addr as usize + 1) = (value >> 16) as u8;
            *ptr.add(addr as usize + 2) = (value >> 8) as u8;
            *ptr.add(addr as usize + 3) = value as u8;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    println!("68k Emulator Test\n");
    
    // Create memory
    let mut memory = SimpleMemory::new(64 * 1024);
    
    // Set up reset vectors
    memory.write_u32_direct(0x0, 0x00010000); // Initial SSP
    memory.write_u32_direct(0x4, 0x00001000); // Initial PC
    
    // Write a simple program at 0x1000
    let mut pc = 0x1000;
    
    // MOVEQ #42, D0    (set D0 to 42)
    memory.write_u16_direct(pc, 0x7000 | 42);
    pc += 2;
    
    // MOVEQ #7, D1     (set D1 to 7)
    memory.write_u16_direct(pc, 0x7200 | 7);
    pc += 2;
    
    // ADD.L D1, D0     (D0 = D0 + D1)
    // This would be more complex to encode, so let's use ADDQ instead
    // ADDQ.L #7, D0    (D0 = D0 + 7)
    memory.write_u16_direct(pc, 0x5E80);  // ADDQ.L #7, D0
    pc += 2;
    
    // NOP
    memory.write_u16_direct(pc, 0x4E71);
    pc += 2;
    
    // NOP
    memory.write_u16_direct(pc, 0x4E71);
    pc += 2;
    
    // STOP (will cause an error, ending our test)
    memory.write_u16_direct(pc, 0x4E72);
    memory.write_u16_direct(pc + 2, 0x2000);
    
    // Create CPU
    let mut cpu = M68k::new(M68kModel::M68000);
    
    // Reset CPU
    cpu.reset(&memory)?;
    println!("CPU reset:");
    println!("  PC:  0x{:08X}", cpu.registers.pc);
    println!("  SSP: 0x{:08X}", cpu.registers.ssp);
    println!("  SR:  0x{:04X}", cpu.registers.sr.value());
    println!();
    
    // Execute instructions
    for i in 0..10 {
        let pc = cpu.registers.pc;
        let opcode = memory.read_u16(pc)?;
        
        println!("Step {}: PC=0x{:08X} Opcode=0x{:04X}", i, pc, opcode);
        
        match cpu.step(&memory) {
            Ok(_) => {
                println!("  D0=0x{:08X} D1=0x{:08X}", 
                         cpu.registers.d[0], 
                         cpu.registers.d[1]);
            }
            Err(e) => {
                println!("  Error: {}", e);
                break;
            }
        }
        
        println!();
    }
    
    println!("Final CPU state:");
    println!("  D0: 0x{:08X} ({})", cpu.registers.d[0], cpu.registers.d[0] as i32);
    println!("  D1: 0x{:08X} ({})", cpu.registers.d[1], cpu.registers.d[1] as i32);
    println!("  PC: 0x{:08X}", cpu.registers.pc);
    println!("  SR: 0x{:04X}", cpu.registers.sr.value());
    println!("  Instructions executed: {}", cpu.instruction_count);
    
    Ok(())
}
