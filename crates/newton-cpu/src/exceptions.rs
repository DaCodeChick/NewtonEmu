// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! PowerPC exception handling

use crate::registers::{Registers, MachineStateRegister, spr};
use newton_utils::Result;

/// PowerPC exception types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exception {
    /// System call (sc instruction)
    SystemCall,
    /// Program exception (illegal instruction, privilege violation, trap)
    Program { trap: bool },
    /// Machine check
    MachineCheck,
    /// Data storage (memory access violation)
    DataStorage,
    /// Instruction storage (instruction fetch violation)
    InstructionStorage,
    /// External interrupt
    External,
    /// Alignment
    Alignment,
    /// Decrementer
    Decrementer,
}

impl Exception {
    /// Get the exception vector offset
    pub fn vector_offset(&self) -> u32 {
        match self {
            Exception::SystemCall => 0x0C00,
            Exception::Program { .. } => 0x0700,
            Exception::MachineCheck => 0x0200,
            Exception::DataStorage => 0x0300,
            Exception::InstructionStorage => 0x0400,
            Exception::External => 0x0500,
            Exception::Alignment => 0x0600,
            Exception::Decrementer => 0x0900,
        }
    }
}

/// Take an exception
pub fn take_exception(regs: &mut Registers, exception: Exception, next_pc: u32) -> Result<()> {
    // Save current state
    regs.spr[spr::SRR0] = next_pc;
    regs.spr[spr::SRR1] = regs.msr.bits();
    
    // Update MSR - clear certain bits on exception
    // Keep IP bit to determine exception vector base
    let ip_bit = regs.msr.contains(MachineStateRegister::IP);
    
    // Clear: EE (external interrupts), PR (problem state), FP (floating point), 
    //        FE0/FE1 (floating point exception mode), SE (single step), BE (branch trace)
    // Keep: IP (exception prefix), ME (machine check enable), IR/DR (translation)
    regs.msr.remove(
        MachineStateRegister::EE | 
        MachineStateRegister::PR | 
        MachineStateRegister::FP | 
        MachineStateRegister::FE0 | 
        MachineStateRegister::FE1 | 
        MachineStateRegister::SE | 
        MachineStateRegister::BE
    );
    
    // Compute exception vector address
    let base = if ip_bit { 0xFFF0_0000 } else { 0x0000_0000 };
    let vector = base + exception.vector_offset();
    
    // Jump to exception handler
    regs.pc = vector;
    
    tracing::debug!(
        "Exception {:?}: SRR0=0x{:08X}, SRR1=0x{:08X}, Vector=0x{:08X}",
        exception, regs.spr[spr::SRR0], regs.spr[spr::SRR1], vector
    );
    
    Ok(())
}
