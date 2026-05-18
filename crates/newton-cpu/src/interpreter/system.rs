// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! System and special-purpose register instructions

use crate::registers::Registers;
use newton_utils::Result;

/// SPR register numbers (most common ones)
pub const SPR_XER: u16 = 1;
pub const SPR_LR: u16 = 8;
pub const SPR_CTR: u16 = 9;
pub const SPR_DSISR: u16 = 18;
pub const SPR_DAR: u16 = 19;
pub const SPR_DEC: u16 = 22;
pub const SPR_SDR1: u16 = 25;
pub const SPR_SRR0: u16 = 26;
pub const SPR_SRR1: u16 = 27;
pub const SPR_SPRG0: u16 = 272;
pub const SPR_SPRG1: u16 = 273;
pub const SPR_SPRG2: u16 = 274;
pub const SPR_SPRG3: u16 = 275;
pub const SPR_PVR: u16 = 287;

/// Move from Special Purpose Register
/// mfspr RT, SPR
pub fn mfspr(regs: &mut Registers, rt: u8, spr: u16) -> Result<()> {
    let value = match spr {
        SPR_XER => regs.xer.bits(),
        SPR_LR => regs.lr,
        SPR_CTR => regs.ctr,
        SPR_PVR => {
            // Processor Version Register - identify CPU
            match regs.model {
                crate::registers::PpcModel::G3 => 0x0008_0200, // 750
                crate::registers::PpcModel::G4 => 0x800C_1101, // 7400
                crate::registers::PpcModel::G5 => 0x0039_0202, // 970
            }
        }
        _ => {
            // Generic SPR access
            regs.spr[spr as usize]
        }
    };
    
    regs.gpr[rt as usize] = value;
    Ok(())
}

/// Move to Special Purpose Register
/// mtspr SPR, RS
pub fn mtspr(regs: &mut Registers, spr: u16, rs: u8) -> Result<()> {
    let value = regs.gpr[rs as usize];
    
    match spr {
        SPR_XER => {
            regs.xer = crate::registers::Xer::from_bits_truncate(value);
        }
        SPR_LR => {
            regs.lr = value;
        }
        SPR_CTR => {
            regs.ctr = value;
        }
        SPR_PVR => {
            // PVR is read-only, ignore writes
        }
        _ => {
            // Generic SPR access
            regs.spr[spr as usize] = value;
        }
    }
    
    Ok(())
}

/// Move from Machine State Register
/// mfmsr RT
pub fn mfmsr(regs: &mut Registers, rt: u8) -> Result<()> {
    regs.gpr[rt as usize] = regs.msr.bits();
    Ok(())
}

/// Move to Machine State Register
/// mtmsr RS
pub fn mtmsr(regs: &mut Registers, rs: u8) -> Result<()> {
    let value = regs.gpr[rs as usize];
    regs.msr = crate::registers::MachineStateRegister::from_bits_truncate(value);
    Ok(())
}

/// Move from Condition Register
/// mfcr RT
pub fn mfcr(regs: &mut Registers, rt: u8) -> Result<()> {
    regs.gpr[rt as usize] = regs.cr.bits();
    Ok(())
}

/// Move to Condition Register Fields
/// mtcrf FXM, RS
pub fn mtcrf(regs: &mut Registers, fxm: u8, rs: u8) -> Result<()> {
    let value = regs.gpr[rs as usize];
    let mut cr = regs.cr.bits();
    
    // FXM is a field mask - each bit represents a 4-bit CR field
    for i in 0..8 {
        if (fxm & (1 << (7 - i))) != 0 {
            // Update this field
            let shift = (7 - i) * 4;
            let mask = 0xF << shift;
            cr = (cr & !mask) | (value & mask);
        }
    }
    
    regs.cr = crate::registers::ConditionRegister::from_bits_truncate(cr);
    Ok(())
}
