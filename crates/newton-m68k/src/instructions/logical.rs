// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Logical instructions (AND, OR, EOR, NOT, etc.)

use crate::{Registers, MemoryInterface, AddressingMode, Size};
use crate::addressing::{read_operand, write_operand, update_condition_codes};
use newton_utils::Result;

/// AND - Logical AND
pub fn execute_and(
    size: Size,
    src: &AddressingMode,
    dst: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let src_value = read_operand(src, size, registers, memory)?;
    let dst_value = read_operand(dst, size, registers, memory)?;
    
    let result = dst_value & src_value;
    
    write_operand(dst, size, result, registers, memory)?;
    update_condition_codes(registers, result, size, false, false);
    
    Ok(())
}

/// OR - Logical OR
pub fn execute_or(
    size: Size,
    src: &AddressingMode,
    dst: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let src_value = read_operand(src, size, registers, memory)?;
    let dst_value = read_operand(dst, size, registers, memory)?;
    
    let result = dst_value | src_value;
    
    write_operand(dst, size, result, registers, memory)?;
    update_condition_codes(registers, result, size, false, false);
    
    Ok(())
}

/// EOR - Logical exclusive OR
pub fn execute_eor(
    size: Size,
    src: &AddressingMode,
    dst: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let src_value = read_operand(src, size, registers, memory)?;
    let dst_value = read_operand(dst, size, registers, memory)?;
    
    let result = dst_value ^ src_value;
    
    write_operand(dst, size, result, registers, memory)?;
    update_condition_codes(registers, result, size, false, false);
    
    Ok(())
}

/// NOT - Logical complement
pub fn execute_not(
    size: Size,
    dst: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let value = read_operand(dst, size, registers, memory)?;
    
    let result = match size {
        Size::Byte => !value & 0xFF,
        Size::Word => !value & 0xFFFF,
        Size::Long => !value,
    };
    
    write_operand(dst, size, result, registers, memory)?;
    update_condition_codes(registers, result, size, false, false);
    
    Ok(())
}

/// TST - Test operand
pub fn execute_tst(
    size: Size,
    operand: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let value = read_operand(operand, size, registers, memory)?;
    
    // TST just updates condition codes without writing
    update_condition_codes(registers, value, size, false, false);
    
    Ok(())
}
