// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Condition Register logical operations
//!
//! These instructions perform bitwise logical operations on individual
//! CR bits. They're used for complex conditional logic in compiled code.

use crate::registers::{Registers, ConditionRegister};
use newton_utils::Result;

/// Get a single CR bit (0-31)
fn get_cr_bit(regs: &Registers, bit: u8) -> bool {
    let cr = regs.cr.bits();
    (cr & (1 << (31 - bit))) != 0
}

/// Set a single CR bit (0-31)
fn set_cr_bit(regs: &mut Registers, bit: u8, value: bool) {
    let mut cr = regs.cr.bits();
    let mask = 1 << (31 - bit);
    
    if value {
        cr |= mask;
    } else {
        cr &= !mask;
    }
    
    regs.cr = ConditionRegister::from_bits_truncate(cr);
}

/// Condition Register AND
/// crand BT, BA, BB
/// CR[BT] = CR[BA] & CR[BB]
pub fn crand(regs: &mut Registers, bt: u8, ba: u8, bb: u8) -> Result<()> {
    let a = get_cr_bit(regs, ba);
    let b = get_cr_bit(regs, bb);
    set_cr_bit(regs, bt, a & b);
    Ok(())
}

/// Condition Register AND with Complement
/// crandc BT, BA, BB
/// CR[BT] = CR[BA] & !CR[BB]
pub fn crandc(regs: &mut Registers, bt: u8, ba: u8, bb: u8) -> Result<()> {
    let a = get_cr_bit(regs, ba);
    let b = get_cr_bit(regs, bb);
    set_cr_bit(regs, bt, a & !b);
    Ok(())
}

/// Condition Register Equivalent
/// creqv BT, BA, BB
/// CR[BT] = CR[BA] == CR[BB]
pub fn creqv(regs: &mut Registers, bt: u8, ba: u8, bb: u8) -> Result<()> {
    let a = get_cr_bit(regs, ba);
    let b = get_cr_bit(regs, bb);
    set_cr_bit(regs, bt, !(a ^ b));
    Ok(())
}

/// Condition Register NAND
/// crnand BT, BA, BB
/// CR[BT] = !(CR[BA] & CR[BB])
pub fn crnand(regs: &mut Registers, bt: u8, ba: u8, bb: u8) -> Result<()> {
    let a = get_cr_bit(regs, ba);
    let b = get_cr_bit(regs, bb);
    set_cr_bit(regs, bt, !(a & b));
    Ok(())
}

/// Condition Register NOR
/// crnor BT, BA, BB
/// CR[BT] = !(CR[BA] | CR[BB])
pub fn crnor(regs: &mut Registers, bt: u8, ba: u8, bb: u8) -> Result<()> {
    let a = get_cr_bit(regs, ba);
    let b = get_cr_bit(regs, bb);
    set_cr_bit(regs, bt, !(a | b));
    Ok(())
}

/// Condition Register OR
/// cror BT, BA, BB
/// CR[BT] = CR[BA] | CR[BB]
pub fn cror(regs: &mut Registers, bt: u8, ba: u8, bb: u8) -> Result<()> {
    let a = get_cr_bit(regs, ba);
    let b = get_cr_bit(regs, bb);
    set_cr_bit(regs, bt, a | b);
    Ok(())
}

/// Condition Register OR with Complement
/// crorc BT, BA, BB
/// CR[BT] = CR[BA] | !CR[BB]
pub fn crorc(regs: &mut Registers, bt: u8, ba: u8, bb: u8) -> Result<()> {
    let a = get_cr_bit(regs, ba);
    let b = get_cr_bit(regs, bb);
    set_cr_bit(regs, bt, a | !b);
    Ok(())
}

/// Condition Register XOR
/// crxor BT, BA, BB
/// CR[BT] = CR[BA] ^ CR[BB]
pub fn crxor(regs: &mut Registers, bt: u8, ba: u8, bb: u8) -> Result<()> {
    let a = get_cr_bit(regs, ba);
    let b = get_cr_bit(regs, bb);
    set_cr_bit(regs, bt, a ^ b);
    Ok(())
}
