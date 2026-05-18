// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Logical operation instructions

use crate::registers::Registers;
use newton_utils::Result;
use super::update_cr0;

pub fn and(regs: &mut Registers, ra: u8, rs: u8, rb: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let b = regs.gpr[rb as usize];
    let result = s & b;
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn andc(regs: &mut Registers, ra: u8, rs: u8, rb: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let b = regs.gpr[rb as usize];
    let result = s & !b;
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn andi(regs: &mut Registers, ra: u8, rs: u8, uimm: u16) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let result = s & (uimm as u32);
    regs.gpr[ra as usize] = result;
    update_cr0(regs, result); // Always updates CR0
    Ok(())
}

pub fn andis(regs: &mut Registers, ra: u8, rs: u8, uimm: u16) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let result = s & ((uimm as u32) << 16);
    regs.gpr[ra as usize] = result;
    update_cr0(regs, result); // Always updates CR0
    Ok(())
}

pub fn cntlzw(regs: &mut Registers, ra: u8, rs: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let result = s.leading_zeros();
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn eqv(regs: &mut Registers, ra: u8, rs: u8, rb: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let b = regs.gpr[rb as usize];
    let result = !(s ^ b);
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn extsb(regs: &mut Registers, ra: u8, rs: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let result = (s as i8) as i32 as u32;
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn extsh(regs: &mut Registers, ra: u8, rs: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let result = (s as i16) as i32 as u32;
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn nand(regs: &mut Registers, ra: u8, rs: u8, rb: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let b = regs.gpr[rb as usize];
    let result = !(s & b);
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn nor(regs: &mut Registers, ra: u8, rs: u8, rb: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let b = regs.gpr[rb as usize];
    let result = !(s | b);
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn or(regs: &mut Registers, ra: u8, rs: u8, rb: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let b = regs.gpr[rb as usize];
    let result = s | b;
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn orc(regs: &mut Registers, ra: u8, rs: u8, rb: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let b = regs.gpr[rb as usize];
    let result = s | !b;
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn ori(regs: &mut Registers, ra: u8, rs: u8, uimm: u16) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let result = s | (uimm as u32);
    regs.gpr[ra as usize] = result;
    Ok(())
}

pub fn oris(regs: &mut Registers, ra: u8, rs: u8, uimm: u16) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let result = s | ((uimm as u32) << 16);
    regs.gpr[ra as usize] = result;
    Ok(())
}

pub fn xor(regs: &mut Registers, ra: u8, rs: u8, rb: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let b = regs.gpr[rb as usize];
    let result = s ^ b;
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn xori(regs: &mut Registers, ra: u8, rs: u8, uimm: u16) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let result = s ^ (uimm as u32);
    regs.gpr[ra as usize] = result;
    Ok(())
}

pub fn xoris(regs: &mut Registers, ra: u8, rs: u8, uimm: u16) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let result = s ^ ((uimm as u32) << 16);
    regs.gpr[ra as usize] = result;
    Ok(())
}
