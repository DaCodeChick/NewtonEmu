// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Floating-point arithmetic instructions

use crate::registers::Registers;
use newton_utils::Result;

/// Floating-point add (double precision)
/// fadd FRT, FRA, FRB
pub fn fadd(regs: &mut Registers, frt: u8, fra: u8, frb: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize];
    let b = regs.fpr[frb as usize];
    let result = a + b;
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point add (single precision)
/// fadds FRT, FRA, FRB
pub fn fadds(regs: &mut Registers, frt: u8, fra: u8, frb: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize];
    let b = regs.fpr[frb as usize];
    let result = (a as f32 + b as f32) as f64;
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point subtract (double precision)
/// fsub FRT, FRA, FRB
pub fn fsub(regs: &mut Registers, frt: u8, fra: u8, frb: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize];
    let b = regs.fpr[frb as usize];
    let result = a - b;
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point subtract (single precision)
/// fsubs FRT, FRA, FRB
pub fn fsubs(regs: &mut Registers, frt: u8, fra: u8, frb: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize];
    let b = regs.fpr[frb as usize];
    let result = (a as f32 - b as f32) as f64;
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point multiply (double precision)
/// fmul FRT, FRA, FRC
pub fn fmul(regs: &mut Registers, frt: u8, fra: u8, frc: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize];
    let c = regs.fpr[frc as usize];
    let result = a * c;
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point multiply (single precision)
/// fmuls FRT, FRA, FRC
pub fn fmuls(regs: &mut Registers, frt: u8, fra: u8, frc: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize];
    let c = regs.fpr[frc as usize];
    let result = (a as f32 * c as f32) as f64;
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point divide (double precision)
/// fdiv FRT, FRA, FRB
pub fn fdiv(regs: &mut Registers, frt: u8, fra: u8, frb: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize];
    let b = regs.fpr[frb as usize];
    let result = a / b;
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point divide (single precision)
/// fdivs FRT, FRA, FRB
pub fn fdivs(regs: &mut Registers, frt: u8, fra: u8, frb: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize];
    let b = regs.fpr[frb as usize];
    let result = (a as f32 / b as f32) as f64;
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point negate
/// fneg FRT, FRB
pub fn fneg(regs: &mut Registers, frt: u8, frb: u8, rc: bool) -> Result<()> {
    let b = regs.fpr[frb as usize];
    let result = -b;
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point absolute value
/// fabs FRT, FRB
pub fn fabs(regs: &mut Registers, frt: u8, frb: u8, rc: bool) -> Result<()> {
    let b = regs.fpr[frb as usize];
    let result = b.abs();
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point move register
/// fmr FRT, FRB
pub fn fmr(regs: &mut Registers, frt: u8, frb: u8, rc: bool) -> Result<()> {
    let b = regs.fpr[frb as usize];
    
    regs.fpr[frt as usize] = b;
    
    if rc {
        update_fpscr_cr1(regs, b);
    }
    
    Ok(())
}

/// Floating-point square root (double precision)
/// fsqrt FRT, FRB
pub fn fsqrt(regs: &mut Registers, frt: u8, frb: u8, rc: bool) -> Result<()> {
    let b = regs.fpr[frb as usize];
    let result = b.sqrt();
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point square root (single precision)
/// fsqrts FRT, FRB
pub fn fsqrts(regs: &mut Registers, frt: u8, frb: u8, rc: bool) -> Result<()> {
    let b = regs.fpr[frb as usize];
    let result = ((b as f32).sqrt()) as f64;
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point multiply-add (double precision)
/// fmadd FRT, FRA, FRC, FRB  ->  FRT = (FRA * FRC) + FRB
pub fn fmadd(regs: &mut Registers, frt: u8, fra: u8, frc: u8, frb: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize];
    let c = regs.fpr[frc as usize];
    let b = regs.fpr[frb as usize];
    let result = a.mul_add(c, b); // (a * c) + b with better precision
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point multiply-add (single precision)
/// fmadds FRT, FRA, FRC, FRB
pub fn fmadds(regs: &mut Registers, frt: u8, fra: u8, frc: u8, frb: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize] as f32;
    let c = regs.fpr[frc as usize] as f32;
    let b = regs.fpr[frb as usize] as f32;
    let result = a.mul_add(c, b) as f64;
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point multiply-subtract (double precision)
/// fmsub FRT, FRA, FRC, FRB  ->  FRT = (FRA * FRC) - FRB
pub fn fmsub(regs: &mut Registers, frt: u8, fra: u8, frc: u8, frb: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize];
    let c = regs.fpr[frc as usize];
    let b = regs.fpr[frb as usize];
    let result = a.mul_add(c, -b); // (a * c) - b
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point multiply-subtract (single precision)
/// fmsubs FRT, FRA, FRC, FRB
pub fn fmsubs(regs: &mut Registers, frt: u8, fra: u8, frc: u8, frb: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize] as f32;
    let c = regs.fpr[frc as usize] as f32;
    let b = regs.fpr[frb as usize] as f32;
    let result = a.mul_add(c, -b) as f64;
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point negative multiply-add (double precision)
/// fnmadd FRT, FRA, FRC, FRB  ->  FRT = -((FRA * FRC) + FRB)
pub fn fnmadd(regs: &mut Registers, frt: u8, fra: u8, frc: u8, frb: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize];
    let c = regs.fpr[frc as usize];
    let b = regs.fpr[frb as usize];
    let result = -a.mul_add(c, b);
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point negative multiply-add (single precision)
/// fnmadds FRT, FRA, FRC, FRB
pub fn fnmadds(regs: &mut Registers, frt: u8, fra: u8, frc: u8, frb: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize] as f32;
    let c = regs.fpr[frc as usize] as f32;
    let b = regs.fpr[frb as usize] as f32;
    let result = -(a.mul_add(c, b)) as f64;
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point negative multiply-subtract (double precision)
/// fnmsub FRT, FRA, FRC, FRB  ->  FRT = -((FRA * FRC) - FRB)
pub fn fnmsub(regs: &mut Registers, frt: u8, fra: u8, frc: u8, frb: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize];
    let c = regs.fpr[frc as usize];
    let b = regs.fpr[frb as usize];
    let result = -a.mul_add(c, -b);
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Floating-point negative multiply-subtract (single precision)
/// fnmsubs FRT, FRA, FRC, FRB
pub fn fnmsubs(regs: &mut Registers, frt: u8, fra: u8, frc: u8, frb: u8, rc: bool) -> Result<()> {
    let a = regs.fpr[fra as usize] as f32;
    let c = regs.fpr[frc as usize] as f32;
    let b = regs.fpr[frb as usize] as f32;
    let result = -(a.mul_add(c, -b)) as f64;
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}

/// Update FPSCR and CR1 based on floating-point result
/// This is a simplified version - real implementation would set all FPSCR flags
fn update_fpscr_cr1(regs: &mut Registers, result: f64) {
    // Simplified: Just update CR1 based on result class
    // Real implementation would update FPSCR flags and copy to CR1
    
    let mut cr1 = 0u32;
    
    if result.is_nan() {
        cr1 |= 0x1; // FU (Floating-point Unordered or NaN)
    } else if result == 0.0 {
        cr1 |= 0x2; // FE (Floating-point Equal or Zero)
    } else if result > 0.0 {
        cr1 |= 0x4; // FG (Floating-point Greater than Zero)
    } else {
        cr1 |= 0x8; // FL (Floating-point Less than Zero)
    }
    
    // Update CR1 (bits 4-7 of CR)
    let mut cr = regs.cr.bits();
    cr = (cr & 0xF0FF_FFFF) | (cr1 << 24);
    regs.cr = crate::registers::ConditionRegister::from_bits_truncate(cr);
}

/// Floating-point compare unordered
/// fcmpu CRFD, FRA, FRB
pub fn fcmpu(regs: &mut Registers, crfd: u8, fra: u8, frb: u8) -> Result<()> {
    let a = regs.fpr[fra as usize];
    let b = regs.fpr[frb as usize];
    
    // Determine comparison result
    let mut cr_field = 0u32;
    
    if a.is_nan() || b.is_nan() {
        cr_field = 0x1; // FU (Unordered)
    } else if a == b {
        cr_field = 0x2; // EQ
    } else if a < b {
        cr_field = 0x8; // LT
    } else {
        cr_field = 0x4; // GT
    }
    
    // Update the specified CR field
    let shift = 28 - (crfd * 4);
    let mask = !(0xF << shift);
    let mut cr = regs.cr.bits();
    cr = (cr & mask) | (cr_field << shift);
    regs.cr = crate::registers::ConditionRegister::from_bits_truncate(cr);
    
    Ok(())
}

/// Floating-point compare ordered
/// fcmpo CRFD, FRA, FRB
pub fn fcmpo(regs: &mut Registers, crfd: u8, fra: u8, frb: u8) -> Result<()> {
    let a = regs.fpr[fra as usize];
    let b = regs.fpr[frb as usize];
    
    // Determine comparison result
    let mut cr_field = 0u32;
    
    if a.is_nan() || b.is_nan() {
        cr_field = 0x1; // FU (Unordered)
        // fcmpo would also set VXSNAN exception if either is SNaN
        // For now, simplified implementation
    } else if a == b {
        cr_field = 0x2; // EQ
    } else if a < b {
        cr_field = 0x8; // LT
    } else {
        cr_field = 0x4; // GT
    }
    
    // Update the specified CR field
    let shift = 28 - (crfd * 4);
    let mask = !(0xF << shift);
    let mut cr = regs.cr.bits();
    cr = (cr & mask) | (cr_field << shift);
    regs.cr = crate::registers::ConditionRegister::from_bits_truncate(cr);
    
    Ok(())
}

/// Floating-point convert to integer word with round toward zero
/// fctiwz FRT, FRB
pub fn fctiwz(regs: &mut Registers, frt: u8, frb: u8, rc: bool) -> Result<()> {
    let b = regs.fpr[frb as usize];
    
    // Convert to i32 with truncation (round toward zero)
    let int_result = if b.is_nan() || b.is_infinite() {
        // Undefined result for NaN/Inf - use 0x8000_0000
        0x8000_0000u32
    } else if b > i32::MAX as f64 {
        0x7FFF_FFFF
    } else if b < i32::MIN as f64 {
        0x8000_0000u32
    } else {
        b.trunc() as i32 as u32
    };
    
    // Store as double with integer in lower 32 bits
    // Upper 32 bits undefined but typically 0xFFF8_0000
    regs.fpr[frt as usize] = f64::from_bits(0xFFF8_0000_0000_0000u64 | int_result as u64);
    
    if rc {
        update_fpscr_cr1(regs, regs.fpr[frt as usize]);
    }
    
    Ok(())
}

/// Floating-point round to single precision
/// frsp FRT, FRB
pub fn frsp(regs: &mut Registers, frt: u8, frb: u8, rc: bool) -> Result<()> {
    let b = regs.fpr[frb as usize];
    
    // Round to single precision and back to double
    let result = (b as f32) as f64;
    
    regs.fpr[frt as usize] = result;
    
    if rc {
        update_fpscr_cr1(regs, result);
    }
    
    Ok(())
}
