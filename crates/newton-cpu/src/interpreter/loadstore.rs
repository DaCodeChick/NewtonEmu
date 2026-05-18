// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
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

// Floating-point loads (64-bit double precision)

/// Load floating-point double
/// lfd FRT, d(RA)
pub fn lfd(regs: &mut Registers, memory: &dyn MemoryInterface, frt: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let bits = memory.read_u64(ea)?;
    regs.fpr[frt as usize] = f64::from_bits(bits);
    Ok(())
}

/// Load floating-point double with update
/// lfdu FRT, d(RA)
pub fn lfdu(regs: &mut Registers, memory: &dyn MemoryInterface, frt: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let bits = memory.read_u64(ea)?;
    regs.fpr[frt as usize] = f64::from_bits(bits);
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

// Floating-point loads (32-bit single precision)

/// Load floating-point single
/// lfs FRT, d(RA)
pub fn lfs(regs: &mut Registers, memory: &dyn MemoryInterface, frt: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let bits = memory.read_u32(ea)?;
    let single = f32::from_bits(bits);
    regs.fpr[frt as usize] = single as f64; // Convert to double
    Ok(())
}

/// Load floating-point single with update
/// lfsu FRT, d(RA)
pub fn lfsu(regs: &mut Registers, memory: &dyn MemoryInterface, frt: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let bits = memory.read_u32(ea)?;
    let single = f32::from_bits(bits);
    regs.fpr[frt as usize] = single as f64; // Convert to double
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

// Floating-point stores (64-bit double precision)

/// Store floating-point double
/// stfd FRS, d(RA)
pub fn stfd(regs: &mut Registers, memory: &dyn MemoryInterface, frs: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let bits = regs.fpr[frs as usize].to_bits();
    memory.write_u64(ea, bits)?;
    Ok(())
}

/// Store floating-point double with update
/// stfdu FRS, d(RA)
pub fn stfdu(regs: &mut Registers, memory: &dyn MemoryInterface, frs: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let bits = regs.fpr[frs as usize].to_bits();
    memory.write_u64(ea, bits)?;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

// Floating-point stores (32-bit single precision)

/// Store floating-point single
/// stfs FRS, d(RA)
pub fn stfs(regs: &mut Registers, memory: &dyn MemoryInterface, frs: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let single = regs.fpr[frs as usize] as f32;
    let bits = single.to_bits();
    memory.write_u32(ea, bits)?;
    Ok(())
}

/// Store floating-point single with update
/// stfsu FRS, d(RA)
pub fn stfsu(regs: &mut Registers, memory: &dyn MemoryInterface, frs: u8, ra: u8, d: i16) -> Result<()> {
    let ea = effective_address(regs, ra, d as i32);
    let single = regs.fpr[frs as usize] as f32;
    let bits = single.to_bits();
    memory.write_u32(ea, bits)?;
    regs.gpr[ra as usize] = ea; // Update base register
    Ok(())
}

// ============================================================================
// Multiple/String Load-Store Instructions
// ============================================================================

/// Load Multiple Word
/// lmw RT, d(RA)
/// Load words starting at RT through r31 from consecutive memory addresses
pub fn lmw(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, d: i16) -> Result<()> {
    let mut ea = effective_address(regs, ra, d as i32);
    
    // Load RT through r31
    for r in rt..32 {
        let value = memory.read_u32(ea)?;
        regs.gpr[r as usize] = value;
        ea = ea.wrapping_add(4);
    }
    
    Ok(())
}

/// Store Multiple Word
/// stmw RS, d(RA)
/// Store words from RS through r31 to consecutive memory addresses
pub fn stmw(regs: &mut Registers, memory: &dyn MemoryInterface, rs: u8, ra: u8, d: i16) -> Result<()> {
    let mut ea = effective_address(regs, ra, d as i32);
    
    // Store RS through r31
    for r in rs..32 {
        memory.write_u32(ea, regs.gpr[r as usize])?;
        ea = ea.wrapping_add(4);
    }
    
    Ok(())
}

/// Load String Word Immediate
/// lswi RT, RA, NB
/// Load NB bytes starting at RT, wrapping around registers
pub fn lswi(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, nb: u8) -> Result<()> {
    let ea = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let count = if nb == 0 { 32 } else { nb as u32 };
    
    let mut addr = ea;
    let mut reg = rt as usize;
    let mut shift = 24; // Start at high byte
    let mut current_word = 0u32;
    
    for _ in 0..count {
        let byte = memory.read_u8(addr)?;
        current_word |= (byte as u32) << shift;
        
        if shift == 0 {
            // Word complete, store it
            regs.gpr[reg] = current_word;
            reg = (reg + 1) % 32;
            shift = 24;
            current_word = 0;
        } else {
            shift -= 8;
        }
        
        addr = addr.wrapping_add(1);
    }
    
    // Store partial word if any
    if shift != 24 {
        regs.gpr[reg] = current_word;
    }
    
    Ok(())
}

/// Load String Word Indexed
/// lswx RT, RA, RB
/// Load XER[25-31] bytes starting at RT
pub fn lswx(regs: &mut Registers, memory: &dyn MemoryInterface, rt: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let count = regs.xer.bits() & 0x7F; // XER bits 25-31
    
    let mut addr = ea;
    let mut reg = rt as usize;
    let mut shift = 24;
    let mut current_word = 0u32;
    
    for _ in 0..count {
        let byte = memory.read_u8(addr)?;
        current_word |= (byte as u32) << shift;
        
        if shift == 0 {
            regs.gpr[reg] = current_word;
            reg = (reg + 1) % 32;
            shift = 24;
            current_word = 0;
        } else {
            shift -= 8;
        }
        
        addr = addr.wrapping_add(1);
    }
    
    if shift != 24 {
        regs.gpr[reg] = current_word;
    }
    
    Ok(())
}

/// Store String Word Immediate
/// stswi RS, RA, NB
/// Store NB bytes from RS onwards, wrapping around registers
pub fn stswi(regs: &mut Registers, memory: &dyn MemoryInterface, rs: u8, ra: u8, nb: u8) -> Result<()> {
    let ea = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let count = if nb == 0 { 32 } else { nb as u32 };
    
    let mut addr = ea;
    let mut reg = rs as usize;
    let mut shift = 24;
    let mut current_word = regs.gpr[reg];
    
    for _ in 0..count {
        let byte = ((current_word >> shift) & 0xFF) as u8;
        memory.write_u8(addr, byte)?;
        
        if shift == 0 {
            reg = (reg + 1) % 32;
            current_word = regs.gpr[reg];
            shift = 24;
        } else {
            shift -= 8;
        }
        
        addr = addr.wrapping_add(1);
    }
    
    Ok(())
}

/// Store String Word Indexed
/// stswx RS, RA, RB
/// Store XER[25-31] bytes from RS onwards
pub fn stswx(regs: &mut Registers, memory: &dyn MemoryInterface, rs: u8, ra: u8, rb: u8) -> Result<()> {
    let ea = effective_address_indexed(regs, ra, rb);
    let count = regs.xer.bits() & 0x7F;
    
    let mut addr = ea;
    let mut reg = rs as usize;
    let mut shift = 24;
    let mut current_word = regs.gpr[reg];
    
    for _ in 0..count {
        let byte = ((current_word >> shift) & 0xFF) as u8;
        memory.write_u8(addr, byte)?;
        
        if shift == 0 {
            reg = (reg + 1) % 32;
            current_word = regs.gpr[reg];
            shift = 24;
        } else {
            shift -= 8;
        }
        
        addr = addr.wrapping_add(1);
    }
    
    Ok(())
}
