// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Arithmetic instructions (ADD, SUB, MUL, DIV, etc.)

use crate::{Registers, MemoryInterface, AddressingMode, Size};
use crate::addressing::{read_operand, write_operand, update_condition_codes};
use newton_utils::Result;

/// ADD - Add
pub fn execute_add(
    size: Size,
    src: &AddressingMode,
    dst: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let src_value = read_operand(src, size, registers, memory)?;
    let dst_value = read_operand(dst, size, registers, memory)?;
    
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
    
    write_operand(dst, size, result, registers, memory)?;
    update_condition_codes(registers, result, size, carry, overflow);
    registers.sr.set_extend(carry);
    
    Ok(())
}

/// ADDA - Add to address register
pub fn execute_adda(
    size: Size,
    src: &AddressingMode,
    reg: u8,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let src_value = read_operand(src, size, registers, memory)?;
    
    // Sign-extend if word size
    let src_value = match size {
        Size::Word => (src_value as i16) as i32 as u32,
        Size::Long => src_value,
        _ => src_value,
    };
    
    registers.a[reg as usize] = registers.a[reg as usize].wrapping_add(src_value);
    
    // ADDA does not affect condition codes
    Ok(())
}

/// ADDI - Add immediate
pub fn execute_addi(
    size: Size,
    imm: u32,
    dst: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    execute_add(size, &AddressingMode::Immediate(imm), dst, registers, memory)
}

/// ADDQ - Add quick (1-8)
pub fn execute_addq(
    size: Size,
    data: u8,
    dst: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    execute_add(size, &AddressingMode::Immediate(data as u32), dst, registers, memory)
}

/// SUB - Subtract
pub fn execute_sub(
    size: Size,
    src: &AddressingMode,
    dst: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let src_value = read_operand(src, size, registers, memory)?;
    let dst_value = read_operand(dst, size, registers, memory)?;
    
    let (result, carry, overflow) = match size {
        Size::Byte => {
            let s = src_value as u8;
            let d = dst_value as u8;
            let (r, c) = d.overflowing_sub(s);
            let v = ((d ^ s) & (d ^ r) & 0x80) != 0;
            (r as u32, c, v)
        }
        Size::Word => {
            let s = src_value as u16;
            let d = dst_value as u16;
            let (r, c) = d.overflowing_sub(s);
            let v = ((d ^ s) & (d ^ r) & 0x8000) != 0;
            (r as u32, c, v)
        }
        Size::Long => {
            let (r, c) = dst_value.overflowing_sub(src_value);
            let v = ((dst_value ^ src_value) & (dst_value ^ r) & 0x8000_0000) != 0;
            (r, c, v)
        }
    };
    
    write_operand(dst, size, result, registers, memory)?;
    update_condition_codes(registers, result, size, carry, overflow);
    registers.sr.set_extend(carry);
    
    Ok(())
}

/// SUBA - Subtract from address register
pub fn execute_suba(
    size: Size,
    src: &AddressingMode,
    reg: u8,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let src_value = read_operand(src, size, registers, memory)?;
    
    let src_value = match size {
        Size::Word => (src_value as i16) as i32 as u32,
        Size::Long => src_value,
        _ => src_value,
    };
    
    registers.a[reg as usize] = registers.a[reg as usize].wrapping_sub(src_value);
    
    // SUBA does not affect condition codes
    Ok(())
}

/// SUBI - Subtract immediate
pub fn execute_subi(
    size: Size,
    imm: u32,
    dst: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    execute_sub(size, &AddressingMode::Immediate(imm), dst, registers, memory)
}

/// SUBQ - Subtract quick (1-8)
pub fn execute_subq(
    size: Size,
    data: u8,
    dst: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    execute_sub(size, &AddressingMode::Immediate(data as u32), dst, registers, memory)
}

/// CMP - Compare
pub fn execute_cmp(
    size: Size,
    src: &AddressingMode,
    dst: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let src_value = read_operand(src, size, registers, memory)?;
    let dst_value = read_operand(dst, size, registers, memory)?;
    
    let (result, carry, overflow) = match size {
        Size::Byte => {
            let s = src_value as u8;
            let d = dst_value as u8;
            let (r, c) = d.overflowing_sub(s);
            let v = ((d ^ s) & (d ^ r) & 0x80) != 0;
            (r as u32, c, v)
        }
        Size::Word => {
            let s = src_value as u16;
            let d = dst_value as u16;
            let (r, c) = d.overflowing_sub(s);
            let v = ((d ^ s) & (d ^ r) & 0x8000) != 0;
            (r as u32, c, v)
        }
        Size::Long => {
            let (r, c) = dst_value.overflowing_sub(src_value);
            let v = ((dst_value ^ src_value) & (dst_value ^ r) & 0x8000_0000) != 0;
            (r, c, v)
        }
    };
    
    // CMP updates condition codes but doesn't write result
    update_condition_codes(registers, result, size, carry, overflow);
    
    Ok(())
}

/// NEG - Negate
pub fn execute_neg(
    size: Size,
    dst: &AddressingMode,
    registers: &mut Registers,
    memory: &dyn MemoryInterface,
) -> Result<()> {
    let value = read_operand(dst, size, registers, memory)?;
    
    let (result, carry, overflow) = match size {
        Size::Byte => {
            let v = value as u8;
            let (r, c) = 0u8.overflowing_sub(v);
            let ov = (v & r & 0x80) != 0;
            (r as u32, c, ov)
        }
        Size::Word => {
            let v = value as u16;
            let (r, c) = 0u16.overflowing_sub(v);
            let ov = (v & r & 0x8000) != 0;
            (r as u32, c, ov)
        }
        Size::Long => {
            let (r, c) = 0u32.overflowing_sub(value);
            let ov = (value & r & 0x8000_0000) != 0;
            (r, c, ov)
        }
    };
    
    write_operand(dst, size, result, registers, memory)?;
    update_condition_codes(registers, result, size, carry, overflow);
    registers.sr.set_extend(carry);
    
    Ok(())
}
