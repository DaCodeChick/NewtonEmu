// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Motorola 68000 CPU emulation
//!
//! This crate implements a Motorola 68000/68020/68030/68040 CPU emulator
//! for running classic Mac OS ROM code.

pub mod registers;
pub mod decoder;
pub mod addressing;

pub use registers::{Registers, StatusRegister, ConditionCode};
pub use decoder::{Instruction, AddressingMode, Size, decode_instruction};
pub use addressing::{calculate_ea, read_operand, write_operand, update_condition_codes};

use newton_utils::{Result, Error};

/// Memory interface for 68k CPU
pub trait MemoryInterface {
    fn read_u8(&self, addr: u32) -> Result<u8>;
    fn read_u16(&self, addr: u32) -> Result<u16>;
    fn read_u32(&self, addr: u32) -> Result<u32>;
    fn write_u8(&self, addr: u32, value: u8) -> Result<()>;
    fn write_u16(&self, addr: u32, value: u16) -> Result<()>;
    fn write_u32(&self, addr: u32, value: u32) -> Result<()>;
}

/// Motorola 68000 CPU
pub struct M68k {
    /// CPU registers
    pub registers: Registers,
    
    /// CPU model variant
    model: M68kModel,
    
    /// Instruction count (for debugging/profiling)
    pub instruction_count: u64,
}

/// 68k CPU model variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M68kModel {
    /// MC68000 - Original 68000 (16-bit data bus)
    M68000,
    /// MC68010 - Added virtual memory support
    M68010,
    /// MC68020 - 32-bit data bus, more instructions
    M68020,
    /// MC68030 - Added MMU
    M68030,
    /// MC68040 - Added FPU
    M68040,
}

impl M68k {
    /// Create a new 68k CPU with the specified model
    pub fn new(model: M68kModel) -> Self {
        Self {
            registers: Registers::new(),
            model,
            instruction_count: 0,
        }
    }
    
    /// Reset the CPU to initial state
    pub fn reset(&mut self, memory: &dyn MemoryInterface) -> Result<()> {
        // Read reset vector from memory
        // Vector table is at address 0x0
        let initial_ssp = memory.read_u32(0x0)?;
        let initial_pc = memory.read_u32(0x4)?;
        
        tracing::info!("68k reset: SSP=0x{:08X}, PC=0x{:08X}", initial_ssp, initial_pc);
        
        self.registers = Registers::new();
        self.registers.ssp = initial_ssp;
        self.registers.a[7] = initial_ssp;
        self.registers.pc = initial_pc;
        self.registers.sr.set_supervisor_mode(true);
        self.instruction_count = 0;
        
        Ok(())
    }
    
    /// Execute one instruction
    pub fn step(&mut self, memory: &dyn MemoryInterface) -> Result<()> {
        let pc = self.registers.pc;
        
        // Fetch instruction
        let opcode = memory.read_u16(pc)?;
        
        // Decode
        let instruction = decode_instruction(opcode);
        
        // Execute
        match instruction {
            Instruction::Nop => {
                self.registers.pc = pc.wrapping_add(2);
            }
            
            Instruction::Moveq { data, reg } => {
                // MOVEQ sign-extends the 8-bit immediate to 32 bits
                let value = data as i32 as u32;
                self.registers.d[reg as usize] = value;
                
                // Update condition codes (N, Z; clear V, C)
                self.registers.sr.set_negative((value & 0x8000_0000) != 0);
                self.registers.sr.set_zero(value == 0);
                self.registers.sr.set_overflow(false);
                self.registers.sr.set_carry(false);
                
                self.registers.pc = pc.wrapping_add(2);
            }
            
            Instruction::Rts => {
                // Pop return address from stack
                let sp = self.registers.sp();
                let return_addr = memory.read_u32(sp)?;
                self.registers.set_sp(sp.wrapping_add(4));
                self.registers.pc = return_addr;
            }
            
            Instruction::Rte => {
                // Return from exception
                if !self.registers.sr.supervisor_mode() {
                    return Err(Error::Cpu("RTE in user mode".into()));
                }
                
                let sp = self.registers.sp();
                let sr = memory.read_u16(sp)?;
                let pc = memory.read_u32(sp.wrapping_add(2))?;
                
                self.registers.sr.set_value(sr);
                self.registers.set_sp(sp.wrapping_add(6));
                self.registers.pc = pc;
            }
            
            Instruction::Bra { displacement } => {
                self.registers.pc = (pc as i32 + 2 + displacement) as u32;
            }
            
            Instruction::Addq { size, data, dst } => {
                use crate::addressing::{read_operand, write_operand};
                
                let src_value = data as u32;
                let dst_value = read_operand(&dst, size, &mut self.registers, memory)?;
                
                let (result, carry, overflow) = match size {
                    Size::Byte => {
                        let s = src_value as u8;
                        let d = dst_value as u8;
                        let (r, c) = d.overflowing_add(s);
                        let v = ((d ^ r) & (s ^ r) & 0x80) != 0;
                        (r as u32, c, v)
                    }
                    Size::Word => {
                        let s = src_value as u16;
                        let d = dst_value as u16;
                        let (r, c) = d.overflowing_add(s);
                        let v = ((d ^ r) & (s ^ r) & 0x8000) != 0;
                        (r as u32, c, v)
                    }
                    Size::Long => {
                        let (r, c) = dst_value.overflowing_add(src_value);
                        let v = ((dst_value ^ r) & (src_value ^ r) & 0x8000_0000) != 0;
                        (r, c, v)
                    }
                };
                
                write_operand(&dst, size, result, &mut self.registers, memory)?;
                
                // Update condition codes
                use crate::addressing::update_condition_codes;
                update_condition_codes(&mut self.registers, result, size, carry, overflow);
                self.registers.sr.set_extend(carry);
                
                self.registers.pc = pc.wrapping_add(2);
            }
            
            Instruction::Illegal { opcode } => {
                tracing::warn!("68k: Illegal instruction 0x{:04X} at PC 0x{:08X}", opcode, pc);
                return Err(Error::Cpu(format!("Illegal 68k instruction: 0x{:04X} at PC 0x{:08X}", opcode, pc)));
            }
            
            _ => {
                tracing::warn!("68k: Unimplemented instruction {:?} at PC 0x{:08X}", instruction, pc);
                return Err(Error::Cpu(format!("Unimplemented 68k instruction: {:?}", instruction)));
            }
        }
        
        self.instruction_count += 1;
        Ok(())
    }
    
    /// Get the CPU model
    pub fn model(&self) -> M68kModel {
        self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestMemory {
        data: Vec<u8>,
    }
    
    impl MemoryInterface for TestMemory {
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
        
        fn write_u8(&self, _addr: u32, _value: u8) -> Result<()> {
            Ok(())
        }
        
        fn write_u16(&self, _addr: u32, _value: u16) -> Result<()> {
            Ok(())
        }
        
        fn write_u32(&self, _addr: u32, _value: u32) -> Result<()> {
            Ok(())
        }
    }
    
    #[test]
    fn test_nop() {
        let mut cpu = M68k::new(M68kModel::M68000);
        let mut mem_data = vec![0u8; 4096];
        
        // Set up reset vectors
        mem_data[0..4].copy_from_slice(&0x00001000u32.to_be_bytes()); // SSP
        mem_data[4..8].copy_from_slice(&0x00000100u32.to_be_bytes()); // PC
        
        // Put a NOP at 0x100
        mem_data[0x100..0x102].copy_from_slice(&0x4E71u16.to_be_bytes());
        
        let memory = TestMemory { data: mem_data };
        
        cpu.reset(&memory).unwrap();
        assert_eq!(cpu.registers.pc, 0x100);
        
        cpu.step(&memory).unwrap();
        assert_eq!(cpu.registers.pc, 0x102);
    }
}
