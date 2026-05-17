// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Rotate and shift instructions

use crate::registers::{Registers, Xer};
use newton_utils::Result;
use super::update_cr0;

/// Generate mask for rotate instructions
fn mask(mb: u8, me: u8) -> u32 {
    let mask_start = if mb <= me {
        !0u32 >> mb
    } else {
        !0u32
    };
    
    let mask_end = if mb <= me {
        !0u32 << (31 - me)
    } else {
        !(!0u32 << (31 - me))
    };
    
    if mb <= me {
        mask_start & mask_end
    } else {
        mask_start | mask_end
    }
}

pub fn rlwimi(regs: &mut Registers, ra: u8, rs: u8, sh: u8, mb: u8, me: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let a = regs.gpr[ra as usize];
    let rotated = s.rotate_left(sh as u32);
    let m = mask(mb, me);
    let result = (rotated & m) | (a & !m);
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn rlwinm(regs: &mut Registers, ra: u8, rs: u8, sh: u8, mb: u8, me: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let rotated = s.rotate_left(sh as u32);
    let m = mask(mb, me);
    let result = rotated & m;
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn rlwnm(regs: &mut Registers, ra: u8, rs: u8, rb: u8, mb: u8, me: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let b = regs.gpr[rb as usize];
    let sh = (b & 0x1F) as u32;
    let rotated = s.rotate_left(sh);
    let m = mask(mb, me);
    let result = rotated & m;
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn slw(regs: &mut Registers, ra: u8, rs: u8, rb: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let b = regs.gpr[rb as usize];
    let sh = b & 0x3F;
    
    let result = if sh < 32 {
        s << sh
    } else {
        0
    };
    
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn sraw(regs: &mut Registers, ra: u8, rs: u8, rb: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize] as i32;
    let b = regs.gpr[rb as usize];
    let sh = b & 0x3F;
    
    let (result, carry) = if sh < 32 {
        let result = s >> sh;
        let carry = s < 0 && (s & ((1 << sh) - 1)) != 0;
        (result, carry)
    } else {
        let result = if s < 0 { -1 } else { 0 };
        let carry = s < 0;
        (result, carry)
    };
    
    regs.gpr[ra as usize] = result as u32;
    
    if carry {
        regs.xer.insert(Xer::CA);
    } else {
        regs.xer.remove(Xer::CA);
    }
    
    if rc { update_cr0(regs, result as u32); }
    Ok(())
}

pub fn srawi(regs: &mut Registers, ra: u8, rs: u8, sh: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize] as i32;
    let result = s >> sh;
    let carry = s < 0 && (s & ((1 << sh) - 1)) != 0;
    
    regs.gpr[ra as usize] = result as u32;
    
    if carry {
        regs.xer.insert(Xer::CA);
    } else {
        regs.xer.remove(Xer::CA);
    }
    
    if rc { update_cr0(regs, result as u32); }
    Ok(())
}

pub fn srw(regs: &mut Registers, ra: u8, rs: u8, rb: u8, rc: bool) -> Result<()> {
    let s = regs.gpr[rs as usize];
    let b = regs.gpr[rb as usize];
    let sh = b & 0x3F;
    
    let result = if sh < 32 {
        s >> sh
    } else {
        0
    };
    
    regs.gpr[ra as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}
