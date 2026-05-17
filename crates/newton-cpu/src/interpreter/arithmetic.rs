// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Integer arithmetic instructions

use crate::registers::{Registers, Xer};
use newton_utils::Result;
use super::update_cr0;

pub fn add(regs: &mut Registers, rt: u8, ra: u8, rb: u8, oe: bool, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize];
    let b = regs.gpr[rb as usize];
    let result = a.wrapping_add(b);
    regs.gpr[rt as usize] = result;
    
    if oe {
        // Check for signed overflow
        let overflow = ((a as i32) > 0 && (b as i32) > 0 && (result as i32) < 0) ||
                      ((a as i32) < 0 && (b as i32) < 0 && (result as i32) > 0);
        if overflow {
            regs.xer.insert(Xer::SO | Xer::OV);
        } else {
            regs.xer.remove(Xer::OV);
        }
    }
    
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn addc(regs: &mut Registers, rt: u8, ra: u8, rb: u8, oe: bool, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize];
    let b = regs.gpr[rb as usize];
    let (result, carry) = a.overflowing_add(b);
    regs.gpr[rt as usize] = result;
    
    if carry {
        regs.xer.insert(Xer::CA);
    } else {
        regs.xer.remove(Xer::CA);
    }
    
    if oe {
        let overflow = ((a as i32) > 0 && (b as i32) > 0 && (result as i32) < 0) ||
                      ((a as i32) < 0 && (b as i32) < 0 && (result as i32) > 0);
        if overflow {
            regs.xer.insert(Xer::SO | Xer::OV);
        } else {
            regs.xer.remove(Xer::OV);
        }
    }
    
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn adde(regs: &mut Registers, rt: u8, ra: u8, rb: u8, oe: bool, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize];
    let b = regs.gpr[rb as usize];
    let ca = if regs.xer.contains(Xer::CA) { 1 } else { 0 };
    
    let (temp, carry1) = a.overflowing_add(b);
    let (result, carry2) = temp.overflowing_add(ca);
    regs.gpr[rt as usize] = result;
    
    if carry1 || carry2 {
        regs.xer.insert(Xer::CA);
    } else {
        regs.xer.remove(Xer::CA);
    }
    
    if oe {
        let overflow = ((a as i32) > 0 && (b as i32) > 0 && (result as i32) < 0) ||
                      ((a as i32) < 0 && (b as i32) < 0 && (result as i32) > 0);
        if overflow {
            regs.xer.insert(Xer::SO | Xer::OV);
        } else {
            regs.xer.remove(Xer::OV);
        }
    }
    
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn addi(regs: &mut Registers, rt: u8, ra: u8, simm: i16) -> Result<()> {
    let a = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    regs.gpr[rt as usize] = a.wrapping_add(simm as i32 as u32);
    Ok(())
}

pub fn addic(regs: &mut Registers, rt: u8, ra: u8, simm: i16) -> Result<()> {
    let a = regs.gpr[ra as usize];
    let (result, carry) = a.overflowing_add(simm as i32 as u32);
    regs.gpr[rt as usize] = result;
    
    if carry {
        regs.xer.insert(Xer::CA);
    } else {
        regs.xer.remove(Xer::CA);
    }
    Ok(())
}

pub fn addic_dot(regs: &mut Registers, rt: u8, ra: u8, simm: i16) -> Result<()> {
    addic(regs, rt, ra, simm)?;
    update_cr0(regs, regs.gpr[rt as usize]);
    Ok(())
}

pub fn addis(regs: &mut Registers, rt: u8, ra: u8, simm: i16) -> Result<()> {
    let a = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    regs.gpr[rt as usize] = a.wrapping_add(((simm as i32) << 16) as u32);
    Ok(())
}

pub fn addme(regs: &mut Registers, rt: u8, ra: u8, oe: bool, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize];
    let ca = if regs.xer.contains(Xer::CA) { 1 } else { 0 };
    let (result, carry) = a.overflowing_add(!0u32);
    let (result, carry2) = result.overflowing_add(ca);
    regs.gpr[rt as usize] = result;
    
    if carry || carry2 {
        regs.xer.insert(Xer::CA);
    } else {
        regs.xer.remove(Xer::CA);
    }
    
    if oe {
        let overflow = (a as i32) == i32::MIN;
        if overflow {
            regs.xer.insert(Xer::SO | Xer::OV);
        } else {
            regs.xer.remove(Xer::OV);
        }
    }
    
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn addze(regs: &mut Registers, rt: u8, ra: u8, oe: bool, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize];
    let ca = if regs.xer.contains(Xer::CA) { 1 } else { 0 };
    let (result, carry) = a.overflowing_add(ca);
    regs.gpr[rt as usize] = result;
    
    if carry {
        regs.xer.insert(Xer::CA);
    } else {
        regs.xer.remove(Xer::CA);
    }
    
    if oe {
        let overflow = (a as i32) == i32::MAX && ca != 0;
        if overflow {
            regs.xer.insert(Xer::SO | Xer::OV);
        } else {
            regs.xer.remove(Xer::OV);
        }
    }
    
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn divw(regs: &mut Registers, rt: u8, ra: u8, rb: u8, oe: bool, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize] as i32;
    let b = regs.gpr[rb as usize] as i32;
    
    let result = if b == 0 || (a == i32::MIN && b == -1) {
        if oe {
            regs.xer.insert(Xer::SO | Xer::OV);
        }
        0 // Undefined result
    } else {
        if oe {
            regs.xer.remove(Xer::OV);
        }
        a / b
    };
    
    regs.gpr[rt as usize] = result as u32;
    if rc { update_cr0(regs, result as u32); }
    Ok(())
}

pub fn divwu(regs: &mut Registers, rt: u8, ra: u8, rb: u8, oe: bool, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize];
    let b = regs.gpr[rb as usize];
    
    let result = if b == 0 {
        if oe {
            regs.xer.insert(Xer::SO | Xer::OV);
        }
        0 // Undefined result
    } else {
        if oe {
            regs.xer.remove(Xer::OV);
        }
        a / b
    };
    
    regs.gpr[rt as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn mulhw(regs: &mut Registers, rt: u8, ra: u8, rb: u8, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize] as i32 as i64;
    let b = regs.gpr[rb as usize] as i32 as i64;
    let result = ((a * b) >> 32) as u32;
    regs.gpr[rt as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn mulhwu(regs: &mut Registers, rt: u8, ra: u8, rb: u8, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize] as u64;
    let b = regs.gpr[rb as usize] as u64;
    let result = ((a * b) >> 32) as u32;
    regs.gpr[rt as usize] = result;
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn mulli(regs: &mut Registers, rt: u8, ra: u8, simm: i16) -> Result<()> {
    let a = regs.gpr[ra as usize] as i32;
    let result = a.wrapping_mul(simm as i32);
    regs.gpr[rt as usize] = result as u32;
    Ok(())
}

pub fn mullw(regs: &mut Registers, rt: u8, ra: u8, rb: u8, oe: bool, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize] as i32;
    let b = regs.gpr[rb as usize] as i32;
    let result_64 = (a as i64) * (b as i64);
    let result = result_64 as i32;
    regs.gpr[rt as usize] = result as u32;
    
    if oe {
        let overflow = result_64 != (result as i64);
        if overflow {
            regs.xer.insert(Xer::SO | Xer::OV);
        } else {
            regs.xer.remove(Xer::OV);
        }
    }
    
    if rc { update_cr0(regs, result as u32); }
    Ok(())
}

pub fn neg(regs: &mut Registers, rt: u8, ra: u8, oe: bool, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize] as i32;
    let result = a.wrapping_neg();
    regs.gpr[rt as usize] = result as u32;
    
    if oe {
        let overflow = a == i32::MIN;
        if overflow {
            regs.xer.insert(Xer::SO | Xer::OV);
        } else {
            regs.xer.remove(Xer::OV);
        }
    }
    
    if rc { update_cr0(regs, result as u32); }
    Ok(())
}

pub fn subf(regs: &mut Registers, rt: u8, ra: u8, rb: u8, oe: bool, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize];
    let b = regs.gpr[rb as usize];
    let result = b.wrapping_sub(a);
    regs.gpr[rt as usize] = result;
    
    if oe {
        let overflow = ((b as i32) >= 0 && (a as i32) < 0 && (result as i32) < 0) ||
                      ((b as i32) < 0 && (a as i32) >= 0 && (result as i32) >= 0);
        if overflow {
            regs.xer.insert(Xer::SO | Xer::OV);
        } else {
            regs.xer.remove(Xer::OV);
        }
    }
    
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn subfc(regs: &mut Registers, rt: u8, ra: u8, rb: u8, oe: bool, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize];
    let b = regs.gpr[rb as usize];
    let (result, borrow) = b.overflowing_sub(a);
    regs.gpr[rt as usize] = result;
    
    if borrow {
        regs.xer.remove(Xer::CA);
    } else {
        regs.xer.insert(Xer::CA);
    }
    
    if oe {
        let overflow = ((b as i32) >= 0 && (a as i32) < 0 && (result as i32) < 0) ||
                      ((b as i32) < 0 && (a as i32) >= 0 && (result as i32) >= 0);
        if overflow {
            regs.xer.insert(Xer::SO | Xer::OV);
        } else {
            regs.xer.remove(Xer::OV);
        }
    }
    
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn subfe(regs: &mut Registers, rt: u8, ra: u8, rb: u8, oe: bool, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize];
    let b = regs.gpr[rb as usize];
    let ca = if regs.xer.contains(Xer::CA) { 1 } else { 0 };
    
    let (temp, borrow1) = b.overflowing_sub(a);
    let (result, borrow2) = temp.overflowing_add(ca);
    regs.gpr[rt as usize] = result;
    
    if borrow1 && borrow2 {
        regs.xer.remove(Xer::CA);
    } else {
        regs.xer.insert(Xer::CA);
    }
    
    if oe {
        let overflow = ((b as i32) >= 0 && (a as i32) < 0 && (result as i32) < 0) ||
                      ((b as i32) < 0 && (a as i32) >= 0 && (result as i32) >= 0);
        if overflow {
            regs.xer.insert(Xer::SO | Xer::OV);
        } else {
            regs.xer.remove(Xer::OV);
        }
    }
    
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn subfic(regs: &mut Registers, rt: u8, ra: u8, simm: i16) -> Result<()> {
    let a = regs.gpr[ra as usize];
    let imm = simm as i32 as u32;
    let (result, borrow) = imm.overflowing_sub(a);
    regs.gpr[rt as usize] = result;
    
    if borrow {
        regs.xer.remove(Xer::CA);
    } else {
        regs.xer.insert(Xer::CA);
    }
    Ok(())
}

pub fn subfme(regs: &mut Registers, rt: u8, ra: u8, oe: bool, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize];
    let ca = if regs.xer.contains(Xer::CA) { 1 } else { 0 };
    let (temp, borrow) = (!0u32).overflowing_sub(a);
    let (result, borrow2) = temp.overflowing_add(ca);
    regs.gpr[rt as usize] = result;
    
    if borrow || borrow2 {
        regs.xer.insert(Xer::CA);
    } else {
        regs.xer.remove(Xer::CA);
    }
    
    if oe {
        let overflow = (a as i32) == i32::MIN && ca == 0;
        if overflow {
            regs.xer.insert(Xer::SO | Xer::OV);
        } else {
            regs.xer.remove(Xer::OV);
        }
    }
    
    if rc { update_cr0(regs, result); }
    Ok(())
}

pub fn subfze(regs: &mut Registers, rt: u8, ra: u8, oe: bool, rc: bool) -> Result<()> {
    let a = regs.gpr[ra as usize];
    let ca = if regs.xer.contains(Xer::CA) { 1 } else { 0 };
    let (result, borrow) = (!a).overflowing_add(ca);
    regs.gpr[rt as usize] = result;
    
    if borrow {
        regs.xer.insert(Xer::CA);
    } else {
        regs.xer.remove(Xer::CA);
    }
    
    if oe {
        let overflow = (a as i32) == i32::MIN && ca == 0;
        if overflow {
            regs.xer.insert(Xer::SO | Xer::OV);
        } else {
            regs.xer.remove(Xer::OV);
        }
    }
    
    if rc { update_cr0(regs, result); }
    Ok(())
}
