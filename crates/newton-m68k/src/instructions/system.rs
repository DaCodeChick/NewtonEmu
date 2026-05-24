// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! System and privileged instructions

use crate::{Registers, MemoryInterface};
use newton_utils::{Result, Error};

/// NOP - No operation
pub fn execute_nop(registers: &mut Registers) -> Result<()> {
    registers.pc = registers.pc.wrapping_add(2);
    Ok(())
}

/// RTE - Return from exception
pub fn execute_rte(
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    if !registers.sr.supervisor_mode() {
        return Err(Error::Cpu("RTE in user mode".into()));
    }
    
    let sp = registers.sp();
    let sr = memory.read_u16(sp)?;
    let pc = memory.read_u32(sp.wrapping_add(2))?;
    
    let was_supervisor = registers.sr.supervisor_mode();
    registers.sr.set_value(sr);
    let is_supervisor = registers.sr.supervisor_mode();
    
    // Handle stack pointer switch if mode changed
    if was_supervisor != is_supervisor {
        if is_supervisor {
            registers.usp = registers.a[7];
            registers.a[7] = registers.ssp;
        } else {
            registers.ssp = registers.a[7];
            registers.a[7] = registers.usp;
        }
    }
    
    registers.set_sp(sp.wrapping_add(6));
    registers.pc = pc;
    
    Ok(())
}

/// TRAP - Trap
pub fn execute_trap(
    vector: u8,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    // Save current SR and PC
    let sr = registers.sr.value();
    let pc = registers.pc;
    
    // Switch to supervisor mode
    if !registers.sr.supervisor_mode() {
        registers.usp = registers.a[7];
        registers.a[7] = registers.ssp;
        registers.sr.set_supervisor_mode(true);
    }
    
    // Push SR and PC
    let sp = registers.sp();
    let new_sp = sp.wrapping_sub(6);
    memory.write_u16(new_sp, sr)?;
    memory.write_u32(new_sp.wrapping_add(2), pc)?;
    registers.set_sp(new_sp);
    
    // Read trap vector (vector 32 + vector number)
    let vector_addr = ((32 + vector as u32) * 4) as u32;
    let trap_handler = memory.read_u32(vector_addr)?;
    
    registers.pc = trap_handler;
    
    Ok(())
}

/// STOP - Stop and wait for interrupt
pub fn execute_stop(
    immediate_data: u16,
    registers: &mut Registers,
) -> Result<()> {
    if !registers.sr.supervisor_mode() {
        return Err(Error::Cpu("STOP in user mode".into()));
    }
    
    registers.sr.set_value(immediate_data);
    registers.pc = registers.pc.wrapping_add(4);
    
    // In a real emulator, this would halt the CPU until an interrupt
    // For now, we just update the SR and continue
    
    Ok(())
}

/// RESET - Reset external devices
pub fn execute_reset(registers: &mut Registers) -> Result<()> {
    if !registers.sr.supervisor_mode() {
        return Err(Error::Cpu("RESET in user mode".into()));
    }
    
    registers.pc = registers.pc.wrapping_add(2);
    
    // In a real emulator, this would reset external devices
    // For now, it's a NOP
    
    Ok(())
}

/// MOVE to/from SR
pub fn execute_move_to_sr(
    value: u16,
    registers: &mut Registers,
) -> Result<()> {
    if !registers.sr.supervisor_mode() {
        return Err(Error::Cpu("MOVE to SR in user mode".into()));
    }
    
    registers.sr.set_value(value);
    Ok(())
}

pub fn execute_move_from_sr(registers: &Registers) -> u16 {
    registers.sr.value()
}

/// MOVE to/from CCR (user mode)
pub fn execute_move_to_ccr(
    value: u8,
    registers: &mut Registers,
) -> Result<()> {
    registers.sr.set_ccr(value);
    Ok(())
}

pub fn execute_move_from_ccr(registers: &Registers) -> u8 {
    registers.sr.ccr()
}
