// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Data movement instructions (MOVE, LEA, PEA, etc.)

use crate::{Registers, MemoryInterface, AddressingMode, Size};
use crate::addressing::{read_operand, write_operand, calculate_ea, update_condition_codes};
use newton_utils::Result;

/// MOVE - Move data from source to destination
pub fn execute_move(
    size: Size,
    src: &AddressingMode,
    dst: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let value = read_operand(src, size, registers, memory)?;
    write_operand(dst, size, value, registers, memory)?;
    
    // Update condition codes (MOVE affects N, Z; clears V, C)
    update_condition_codes(registers, value, size, false, false);
    
    Ok(())
}

/// MOVEA - Move to address register
pub fn execute_movea(
    size: Size,
    src: &AddressingMode,
    reg: u8,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let value = read_operand(src, size, registers, memory)?;
    
    // Sign-extend if word size
    let value = match size {
        Size::Word => (value as i16) as i32 as u32,
        Size::Long => value,
        _ => value, // Should never be Byte for MOVEA
    };
    
    registers.a[reg as usize] = value;
    
    // MOVEA does not affect condition codes
    Ok(())
}

/// LEA - Load effective address
pub fn execute_lea(
    src: &AddressingMode,
    reg: u8,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let ea = calculate_ea(src, registers, memory)?;
    registers.a[reg as usize] = ea;
    
    // LEA does not affect condition codes
    Ok(())
}

/// PEA - Push effective address
pub fn execute_pea(
    src: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let ea = calculate_ea(src, registers, memory)?;
    
    // Push to stack
    let sp = registers.sp().wrapping_sub(4);
    registers.set_sp(sp);
    memory.write_u32(sp, ea)?;
    
    // PEA does not affect condition codes
    Ok(())
}

/// MOVEQ - Move quick (8-bit immediate to data register)
pub fn execute_moveq(
    data: i8,
    reg: u8,
    registers: &mut Registers,
) -> Result<()> {
    let value = data as i32 as u32;
    registers.d[reg as usize] = value;
    
    // Update condition codes
    update_condition_codes(registers, value, Size::Long, false, false);
    
    Ok(())
}

/// EXG - Exchange registers
pub fn execute_exg(
    rx: u8,
    ry: u8,
    mode: u8,
    registers: &mut Registers,
) -> Result<()> {
    match mode {
        0b01000 => {
            // Exchange data registers
            let temp = registers.d[rx as usize];
            registers.d[rx as usize] = registers.d[ry as usize];
            registers.d[ry as usize] = temp;
        }
        0b01001 => {
            // Exchange address registers
            let temp = registers.a[rx as usize];
            registers.a[rx as usize] = registers.a[ry as usize];
            registers.a[ry as usize] = temp;
        }
        0b10001 => {
            // Exchange data and address registers
            let temp = registers.d[rx as usize];
            registers.d[rx as usize] = registers.a[ry as usize];
            registers.a[ry as usize] = temp;
        }
        _ => {}
    }
    
    // EXG does not affect condition codes
    Ok(())
}

/// SWAP - Swap register halves
pub fn execute_swap(
    reg: u8,
    registers: &mut Registers,
) -> Result<()> {
    let value = registers.d[reg as usize];
    let result = (value << 16) | (value >> 16);
    registers.d[reg as usize] = result;
    
    // Update condition codes
    update_condition_codes(registers, result, Size::Long, false, false);
    
    Ok(())
}

/// CLR - Clear operand
pub fn execute_clr(
    size: Size,
    dst: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    write_operand(dst, size, 0, registers, memory)?;
    
    // CLR always sets Z=1, clears N, V, C
    registers.sr.set_negative(false);
    registers.sr.set_zero(true);
    registers.sr.set_overflow(false);
    registers.sr.set_carry(false);
    
    Ok(())
}
