// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Branch instructions

use crate::registers::Registers;
use newton_utils::Result;

/// Check if branch condition is met
fn check_condition(regs: &Registers, bo: u8, bi: u8) -> bool {
    // BO field encoding:
    // bit 0: 0=decrement CTR, test CTR; 1=don't modify/test CTR
    // bit 1: 0=branch if CTR!=0; 1=branch if CTR==0 (ignored if bit 0=1)
    // bit 2: 0=branch if CR bit false; 1=branch if CR bit true (ignored if bit 3=1)
    // bit 3: 0=test CR bit; 1=don't test CR bit
    // bit 4: 0=reverse branch hint; 1=no hint or follow-through branch hint
    
    let ctr_ok = if (bo & 0b00100) != 0 {
        // Don't test CTR
        true
    } else {
        let ctr_zero = regs.ctr == 1; // After decrement
        let ctr_should_be_zero = (bo & 0b00010) != 0;
        ctr_zero == ctr_should_be_zero
    };
    
    let cond_ok = if (bo & 0b01000) != 0 {
        // Don't test condition
        true
    } else {
        let cr_bit_set = (regs.cr.bits() & (1 << (31 - bi))) != 0;
        let should_be_set = (bo & 0b10000) != 0;
        cr_bit_set == should_be_set
    };
    
    ctr_ok && cond_ok
}

pub fn b(regs: &mut Registers, li: i32, aa: bool, lk: bool) -> Result<()> {
    if lk {
        regs.lr = regs.pc + 4;
    }
    
    if aa {
        regs.pc = (li as u32) & !3;
    } else {
        regs.pc = regs.pc.wrapping_add(li as u32) & !3;
    }
    
    Ok(())
}

pub fn bc(regs: &mut Registers, bo: u8, bi: u8, bd: i16, aa: bool, lk: bool) -> Result<()> {
    // Decrement CTR if required
    if (bo & 0b00100) == 0 {
        regs.ctr = regs.ctr.wrapping_sub(1);
    }
    
    if check_condition(regs, bo, bi) {
        if lk {
            regs.lr = regs.pc + 4;
        }
        
        let target = if aa {
            (bd as i32) as u32
        } else {
            regs.pc.wrapping_add((bd as i32) as u32)
        };
        
        regs.pc = target & !3;
    } else {
        regs.pc = regs.pc.wrapping_add(4);
    }
    
    Ok(())
}

pub fn bcctr(regs: &mut Registers, bo: u8, bi: u8, lk: bool) -> Result<()> {
    // CTR is not decremented for bcctr
    if check_condition(regs, bo, bi) {
        let target = regs.ctr & !3;
        
        if lk {
            regs.lr = regs.pc + 4;
        }
        
        regs.pc = target;
    } else {
        regs.pc = regs.pc.wrapping_add(4);
    }
    
    Ok(())
}

pub fn bclr(regs: &mut Registers, bo: u8, bi: u8, lk: bool) -> Result<()> {
    // Decrement CTR if required
    if (bo & 0b00100) == 0 {
        regs.ctr = regs.ctr.wrapping_sub(1);
    }
    
    if check_condition(regs, bo, bi) {
        let target = regs.lr & !3;
        
        if lk {
            regs.lr = regs.pc + 4;
        }
        
        regs.pc = target;
    } else {
        regs.pc = regs.pc.wrapping_add(4);
    }
    
    Ok(())
}
