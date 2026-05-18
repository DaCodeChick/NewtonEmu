// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Load/Store instructions with memory interface

use crate::registers::Registers;
use crate::MemoryInterface;
use newton_utils::Result;

/// Calculate effective address for load/store
#[inline]
fn effective_address(regs: &Registers, ra: u8, offset: i32) -> u32 {
    let base = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    base.wrapping_add(offset as u32)
}

/// Calculate effective address for indexed load/store
#[inline]
fn effective_address_indexed(regs: &Registers, ra: u8, rb: u8) -> u32 {
    let base = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let index = regs.gpr[rb as usize];
    base.wrapping_add(index)
}

// Word loads (32-bit)

pub fn lwz(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let value = memory.read_u32(ea)?;
    regs.gpr[rt as usize] = value;
    Ok(())
}

pub fn lwzu(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let value = memory.read_u32(ea)?;
    regs.gpr[rt as usize] = value;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

pub fn lwzx(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let value = memory.read_u32(ea)?;
    regs.gpr[rt as usize] = value;
    Ok(())
}

pub fn lwzux(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let value = memory.read_u32(ea)?;
    regs.gpr[rt as usize] = value;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

// Byte loads (8-bit, zero-extended)

pub fn lbz(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let value = memory.read_u8(ea)? as u32;
    regs.gpr[rt as usize] = value;
    Ok(())
}

pub fn lbzu(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let value = memory.read_u8(ea)? as u32;
    regs.gpr[rt as usize] = value;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

pub fn lbzx(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let value = memory.read_u8(ea)? as u32;
    regs.gpr[rt as usize] = value;
    Ok(())
}

pub fn lbzux(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let value = memory.read_u8(ea)? as u32;
    regs.gpr[rt as usize] = value;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

// Halfword loads (16-bit, zero-extended)

pub fn lhz(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let value = memory.read_u16(ea)? as u32;
    regs.gpr[rt as usize] = value;
    Ok(())
}

pub fn lhzu(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let value = memory.read_u16(ea)? as u32;
    regs.gpr[rt as usize] = value;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

pub fn lhzx(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let value = memory.read_u16(ea)? as u32;
    regs.gpr[rt as usize] = value;
    Ok(())
}

pub fn lhzux(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let value = memory.read_u16(ea)? as u32;
    regs.gpr[rt as usize] = value;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

// Halfword loads (16-bit, sign-extended)

pub fn lha(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let value = memory.read_u16(ea)? as i16 as i32 as u32;
    regs.gpr[rt as usize] = value;
    Ok(())
}

pub fn lhau(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let value = memory.read_u16(ea)? as i16 as i32 as u32;
    regs.gpr[rt as usize] = value;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

pub fn lhax(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let value = memory.read_u16(ea)? as i16 as i32 as u32;
    regs.gpr[rt as usize] = value;
    Ok(())
}

pub fn lhaux(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let value = memory.read_u16(ea)? as i16 as i32 as u32;
    regs.gpr[rt as usize] = value;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

// Word stores (32-bit)

pub fn stw(regs: &mut Registers, memory: &dyn MemoryInterface, rs: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let value = regs.gpr[rs as usize];
    memory.write_u32(ea, value)?;
    Ok(())
}

pub fn stwu(regs: &mut Registers, memory: &dyn MemoryInterface, rs: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let value = regs.gpr[rs as usize];
    memory.write_u32(ea, value)?;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

pub fn stwx(regs: &mut Registers, memory: &dyn MemoryInterface, rs: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let value = regs.gpr[rs as usize];
    memory.write_u32(ea, value)?;
    Ok(())
}

pub fn stwux(regs: &mut Registers, memory: &dyn MemoryInterface, rs: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let value = regs.gpr[rs as usize];
    memory.write_u32(ea, value)?;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

// Byte stores (8-bit)

pub fn stb(regs: &mut Registers, memory: &dyn MemoryInterface, rs: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let value = regs.gpr[rs as usize] as u8;
    memory.write_u8(ea, value)?;
    Ok(())
}

pub fn stbu(regs: &mut Registers, memory: &dyn MemoryInterface, rs: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let value = regs.gpr[rs as usize] as u8;
    memory.write_u8(ea, value)?;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

pub fn stbx(regs: &mut Registers, memory: &dyn MemoryInterface, rs: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let value = regs.gpr[rs as usize] as u8;
    memory.write_u8(ea, value)?;
    Ok(())
}

pub fn stbux(regs: &mut Registers, memory: &dyn MemoryInterface, rs: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let value = regs.gpr[rs as usize] as u8;
    memory.write_u8(ea, value)?;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

// Halfword stores (16-bit)

pub fn sth(regs: &mut Registers, memory: &dyn MemoryInterface, rs: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let value = regs.gpr[rs as usize] as u16;
    memory.write_u16(ea, value)?;
    Ok(())
}

pub fn sthu(regs: &mut Registers, memory: &dyn MemoryInterface, rs: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let value = regs.gpr[rs as usize] as u16;
    memory.write_u16(ea, value)?;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

pub fn sthx(regs: &mut Registers, memory: &dyn MemoryInterface, rs: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let value = regs.gpr[rs as usize] as u16;
    memory.write_u16(ea, value)?;
    Ok(())
}

pub fn sthux(regs: &mut Registers, memory: &dyn MemoryInterface, rs: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let value = regs.gpr[rs as usize] as u16;
    memory.write_u16(ea, value)?;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}
