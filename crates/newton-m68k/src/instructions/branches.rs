// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Branch and control flow instructions

use crate::{Registers, MemoryInterface, ConditionCode};
use newton_utils::Result;

/// BRA - Branch always
pub fn execute_bra(
    displacement: i32,
    registers: &mut Registers,
) -> Result<()> {
    let pc = registers.pc;
    registers.pc = (pc as i32 + 2 + displacement) as u32;
    Ok(())
}

/// BSR - Branch to subroutine
pub fn execute_bsr(
    displacement: i32,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let return_address = registers.pc.wrapping_add(2);
    
    // Push return address
    let sp = registers.sp().wrapping_sub(4);
    registers.set_sp(sp);
    memory.write_u32(sp, return_address)?;
    
    // Branch
    registers.pc = (registers.pc as i32 + 2 + displacement) as u32;
    
    Ok(())
}

/// Bcc - Branch conditionally
pub fn execute_bcc(
    condition: ConditionCode,
    displacement: i32,
    registers: &mut Registers,
) -> Result<()> {
    if registers.sr.test_condition(condition) {
        let pc = registers.pc;
        registers.pc = (pc as i32 + 2 + displacement) as u32;
    } else {
        registers.pc = registers.pc.wrapping_add(2);
    }
    Ok(())
}

/// DBcc - Test condition, decrement and branch
pub fn execute_dbcc(
    condition: ConditionCode,
    reg: u8,
    displacement: i16,
    registers: &mut Registers,
) -> Result<()> {
    if !registers.sr.test_condition(condition) {
        // Decrement counter
        let counter = (registers.d[reg as usize] as u16).wrapping_sub(1);
        registers.d[reg as usize] = (registers.d[reg as usize] & 0xFFFF0000) | (counter as u32);
        
        // Branch if counter != -1
        if counter != 0xFFFF {
            let pc = registers.pc;
            registers.pc = (pc as i32 + 2 + displacement as i32) as u32;
            return Ok(());
        }
    }
    
    // Fall through
    registers.pc = registers.pc.wrapping_add(4);
    Ok(())
}

/// JMP - Jump
pub fn execute_jmp(
    address: u32,
    registers: &mut Registers,
) -> Result<()> {
    registers.pc = address;
    Ok(())
}

/// JSR - Jump to subroutine
pub fn execute_jsr(
    address: u32,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let return_address = registers.pc;
    
    // Push return address
    let sp = registers.sp().wrapping_sub(4);
    registers.set_sp(sp);
    memory.write_u32(sp, return_address)?;
    
    // Jump
    registers.pc = address;
    
    Ok(())
}

/// RTS - Return from subroutine
pub fn execute_rts(
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let sp = registers.sp();
    let return_address = memory.read_u32(sp)?;
    registers.set_sp(sp.wrapping_add(4));
    registers.pc = return_address;
    Ok(())
}
