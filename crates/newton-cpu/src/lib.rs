// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! PowerPC CPU emulation
//!
//! This crate implements the PowerPC G4 (7400/7450) CPU, including:
//! - Integer and floating-point instructions
//! - AltiVec SIMD instructions
//! - Supervisor mode and privilege levels
//! - Interpreter and JIT compiler

pub mod altivec;
pub mod decoder;
pub mod interpreter;
pub mod jit;
pub mod registers;

pub use registers::{Registers, PpcModel};
pub use interpreter::Interpreter;
pub use decoder::{Instruction, decode_instruction};

use newton_utils::Result;

/// Memory interface trait for CPU
/// 
/// This allows the CPU to access memory without depending on newton-core directly
pub trait MemoryInterface {
    fn read_u8(&self, addr: u32) -> Result<u8>;
    fn read_u16(&self, addr: u32) -> Result<u16>;
    fn read_u32(&self, addr: u32) -> Result<u32>;
    fn write_u8(&mut self, addr: u32, value: u8) -> Result<()>;
    fn write_u16(&mut self, addr: u32, value: u16) -> Result<()>;
    fn write_u32(&mut self, addr: u32, value: u32) -> Result<()>;
}

/// PowerPC CPU state and execution
pub struct Cpu {
    /// CPU registers
    pub registers: Registers,
    
    /// Interpreter for instruction execution
    interpreter: Interpreter,
    
    /// JIT compiler (optional, enabled later)
    jit_enabled: bool,
}

impl Cpu {
    /// Create a new PowerPC CPU
    pub fn new(model: PpcModel) -> Self {
        Self {
            registers: Registers::new(model),
            interpreter: Interpreter::new(),
            jit_enabled: false,
        }
    }

    /// Reset the CPU to initial state
    pub fn reset(&mut self) {
        self.registers.reset();
    }

    /// Execute a single instruction with memory access
    pub fn step(&mut self, memory: &mut dyn MemoryInterface) -> Result<()> {
        // Fetch instruction from memory at PC
        let instr_word = memory.read_u32(self.registers.pc)?;
        
        // Decode instruction
        let instr = decode_instruction(instr_word)?;
        
        // Execute instruction
        self.interpreter.execute_with_memory(instr, &mut self.registers, memory)?;
        
        // Advance PC (unless instruction modified it, like branches)
        // TODO: Track if instruction modified PC
        self.registers.pc = self.registers.pc.wrapping_add(4);
        
        Ok(())
    }

    /// Enable or disable JIT compilation
    pub fn set_jit_enabled(&mut self, enabled: bool) {
        self.jit_enabled = enabled;
    }
}
