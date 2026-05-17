// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Comparison instructions

use crate::registers::{Registers, Xer};
use newton_utils::Result;
use super::set_cr_field;

pub fn cmp(regs: &mut Registers, crfd: u8, _l: bool, ra: u8, rb: u8) -> Result<()> {
    let a = regs.gpr[ra as usize] as i32;
    let b = regs.gpr[rb as usize] as i32;
    
    let lt = a < b;
    let gt = a > b;
    let eq = a == b;
    let so = regs.xer.contains(Xer::SO);
    
    set_cr_field(regs, crfd, lt, gt, eq, so);
    Ok(())
}

pub fn cmpi(regs: &mut Registers, crfd: u8, _l: bool, ra: u8, simm: i16) -> Result<()> {
    let a = regs.gpr[ra as usize] as i32;
    let b = simm as i32;
    
    let lt = a < b;
    let gt = a > b;
    let eq = a == b;
    let so = regs.xer.contains(Xer::SO);
    
    set_cr_field(regs, crfd, lt, gt, eq, so);
    Ok(())
}

pub fn cmpl(regs: &mut Registers, crfd: u8, _l: bool, ra: u8, rb: u8) -> Result<()> {
    let a = regs.gpr[ra as usize];
    let b = regs.gpr[rb as usize];
    
    let lt = a < b;
    let gt = a > b;
    let eq = a == b;
    let so = regs.xer.contains(Xer::SO);
    
    set_cr_field(regs, crfd, lt, gt, eq, so);
    Ok(())
}

pub fn cmpli(regs: &mut Registers, crfd: u8, _l: bool, ra: u8, uimm: u16) -> Result<()> {
    let a = regs.gpr[ra as usize];
    let b = uimm as u32;
    
    let lt = a < b;
    let gt = a > b;
    let eq = a == b;
    let so = regs.xer.contains(Xer::SO);
    
    set_cr_field(regs, crfd, lt, gt, eq, so);
    Ok(())
}
