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

// Exception handling registers (not yet fully implemented)
#[allow(dead_code)]
pub const SPR_DSISR: u16 = 18;
#[allow(dead_code)]
pub const SPR_DAR: u16 = 19;
#[allow(dead_code)]
pub const SPR_DEC: u16 = 22;
#[allow(dead_code)]
pub const SPR_SDR1: u16 = 25;
#[allow(dead_code)]
pub const SPR_SRR0: u16 = 26;
#[allow(dead_code)]
pub const SPR_SRR1: u16 = 27;

// General purpose SPRs (not yet fully implemented)
#[allow(dead_code)]
pub const SPR_SPRG0: u16 = 272;
#[allow(dead_code)]
pub const SPR_SPRG1: u16 = 273;
#[allow(dead_code)]
pub const SPR_SPRG2: u16 = 274;
#[allow(dead_code)]
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
                crate::registers::PpcModel::G3 => 0x0008_0200, // PowerPC 750 (G3)
                crate::registers::PpcModel::G4 => 0x000C_1101, // PowerPC 7400 (G4)
                crate::registers::PpcModel::G5 => 0x0039_0202, // PowerPC 970 (G5)
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

// ============================================================================
// Cache Management Instructions
// ============================================================================
// These are all no-ops in an interpreter, but real PowerPC code uses them
// extensively for cache coherency. We implement them to avoid unknown
// instruction warnings.

/// Data Cache Block Flush
/// dcbf RA, RB
/// Flushes a cache block (no-op in interpreter)
pub fn dcbf(_regs: &mut Registers, _ra: u8, _rb: u8) -> Result<()> {
    // No-op: interpreter has no data cache
    Ok(())
}

/// Data Cache Block Store
/// dcbst RA, RB
/// Stores a cache block to memory (no-op in interpreter)
pub fn dcbst(_regs: &mut Registers, _ra: u8, _rb: u8) -> Result<()> {
    // No-op: interpreter has no data cache
    Ok(())
}

/// Data Cache Block Touch
/// dcbt RA, RB
/// Prefetch hint for data cache (no-op in interpreter)
pub fn dcbt(_regs: &mut Registers, _ra: u8, _rb: u8) -> Result<()> {
    // No-op: interpreter has no data cache to prefetch
    Ok(())
}

/// Data Cache Block Touch for Store
/// dcbtst RA, RB
/// Prefetch hint for data cache (no-op in interpreter)
pub fn dcbtst(_regs: &mut Registers, _ra: u8, _rb: u8) -> Result<()> {
    // No-op: interpreter has no data cache to prefetch
    Ok(())
}

/// Data Cache Block Zero
/// dcbz RA, RB
/// Zeros a cache block (needs memory interface)
/// This is a special case - it actually does write to memory
pub fn dcbz(regs: &Registers, ra: u8, rb: u8, memory: &dyn crate::MemoryInterface) -> Result<()> {
    // Calculate effective address
    let ea = if ra == 0 {
        regs.gpr[rb as usize]
    } else {
        regs.gpr[ra as usize].wrapping_add(regs.gpr[rb as usize])
    };
    
    // Zero a cache block (typically 32 bytes on G3/G4)
    let block_start = ea & !0x1F; // Align to 32-byte boundary
    for i in 0..32 {
        memory.write_u8(block_start + i, 0)?;
    }
    
    Ok(())
}

/// Instruction Cache Block Invalidate
/// icbi RA, RB
/// Invalidates an instruction cache block (no-op in interpreter)
pub fn icbi(_regs: &mut Registers, _ra: u8, _rb: u8) -> Result<()> {
    // No-op: interpreter has no instruction cache
    // In a JIT, this would need to invalidate compiled blocks
    Ok(())
}

// ============================================================================
// Segment Register Instructions
// ============================================================================
// PowerPC uses segment registers for memory translation (32-bit mode).
// These are privileged instructions typically used by the OS kernel.

/// Move from Segment Register
/// mfsr RT, SR
/// RT = SR[SR]
pub fn mfsr(regs: &mut Registers, rt: u8, sr: u8) -> Result<()> {
    // SR is 4 bits, so we mask to 0-15
    let sr_index = (sr & 0xF) as usize;
    regs.gpr[rt as usize] = regs.sr[sr_index];
    Ok(())
}

/// Move from Segment Register Indirect
/// mfsrin RT, RB
/// RT = SR[RB[0:3]]
/// Gets segment register indexed by high 4 bits of RB
pub fn mfsrin(regs: &mut Registers, rt: u8, rb: u8) -> Result<()> {
    let rb_val = regs.gpr[rb as usize];
    let sr_index = ((rb_val >> 28) & 0xF) as usize;
    regs.gpr[rt as usize] = regs.sr[sr_index];
    Ok(())
}

/// Move to Segment Register
/// mtsr SR, RS
/// SR[SR] = RS
pub fn mtsr(regs: &mut Registers, sr: u8, rs: u8) -> Result<()> {
    let sr_index = (sr & 0xF) as usize;
    regs.sr[sr_index] = regs.gpr[rs as usize];
    Ok(())
}

/// Move to Segment Register Indirect
/// mtsrin RS, RB
/// SR[RB[0:3]] = RS
/// Sets segment register indexed by high 4 bits of RB
pub fn mtsrin(regs: &mut Registers, rs: u8, rb: u8) -> Result<()> {
    let rb_val = regs.gpr[rb as usize];
    let sr_index = ((rb_val >> 28) & 0xF) as usize;
    regs.sr[sr_index] = regs.gpr[rs as usize];
    Ok(())
}

// ============================================================================
// TLB Management Instructions
// ============================================================================
// Translation Lookaside Buffer instructions for managing address translation
// cache. These are privileged instructions used by the OS kernel.

/// TLB Invalidate Entry
/// tlbie RB
/// Invalidates TLB entry for effective address in RB
pub fn tlbie(_regs: &mut Registers, _rb: u8) -> Result<()> {
    // No-op: interpreter doesn't have a TLB
    // In a real implementation, this would invalidate the TLB entry
    // for the address in RB
    Ok(())
}

/// TLB Invalidate All
/// tlbia
/// Invalidates all TLB entries
pub fn tlbia(_regs: &mut Registers) -> Result<()> {
    // No-op: interpreter doesn't have a TLB
    // In a real implementation, this would flush the entire TLB
    Ok(())
}

/// TLB Synchronize
/// tlbsync
/// Ensures TLB invalidations are complete on all processors
pub fn tlbsync(_regs: &mut Registers) -> Result<()> {
    // No-op: interpreter doesn't have a TLB
    // In a multiprocessor system, this ensures all processors have
    // seen TLB invalidations before proceeding
    Ok(())
}

// ============================================================================
// Time Base Register Access
// ============================================================================

/// Move From Time Base
/// mftb RT, TBR
/// Read time base register (TBL or TBU)
pub fn mftb(regs: &mut Registers, rt: u8, tbr: u16) -> Result<()> {
    // TBR encoding: bits 5-9 and 0-4 are swapped
    // TBL = 268 (0x10C in normal encoding, becomes 0x0CC in mftb encoding)
    // TBU = 269 (0x10D in normal encoding, becomes 0x0CD in mftb encoding)
    
    // For simplicity, we'll use a monotonic counter
    // In a real implementation, this would be tied to actual time
    let value = match tbr {
        268 | 0x0CC => {
            // TBL - lower 32 bits of time base
            // For now, return a simple incrementing value
            // TODO: Implement proper time base that increments with CPU ticks
            regs.pc // Temporary: use PC as a stand-in for time
        }
        269 | 0x0CD => {
            // TBU - upper 32 bits of time base
            0 // Always 0 for now
        }
        _ => {
            return Err(newton_utils::Error::Cpu(format!("Invalid TBR register: {}", tbr)));
        }
    };
    
    regs.gpr[rt as usize] = value;
    Ok(())
}

// ============================================================================
// Trap Instructions
// ============================================================================

/// Trap Word Immediate
/// twi TO, RA, SIMM
/// Trap if specified condition is met
pub fn twi(regs: &mut Registers, to: u8, ra: u8, simm: i16) -> Result<()> {
    let a = regs.gpr[ra as usize] as i32;
    let b = simm as i32;
    
    // TO field bits:
    // bit 0: LT - trap if a < b (signed)
    // bit 1: GT - trap if a > b (signed)
    // bit 2: EQ - trap if a == b
    // bit 3: LLT - trap if a < b (unsigned)
    // bit 4: LGT - trap if a > b (unsigned)
    
    let should_trap = 
        ((to & 0b10000) != 0 && (a < b)) ||
        ((to & 0b01000) != 0 && (a > b)) ||
        ((to & 0b00100) != 0 && (a == b)) ||
        ((to & 0b00010) != 0 && ((regs.gpr[ra as usize]) < (simm as u32))) ||
        ((to & 0b00001) != 0 && ((regs.gpr[ra as usize]) > (simm as u32)));
    
    if should_trap {
        // Return a trap indication - the CPU will handle the exception
        return Err(newton_utils::Error::Cpu("Trap condition met".to_string()));
    }
    
    Ok(())
}

// ============================================================================
// External Control Instructions
// ============================================================================

/// External Control In Word Indexed
/// eciwx RT, RA, RB
/// Load word using external control (device-specific)
pub fn eciwx(regs: &mut Registers, memory: &dyn crate::MemoryInterface, rt: u8, ra: u8, rb: u8) -> Result<()> {
    // Calculate effective address
    let base = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let index = regs.gpr[rb as usize];
    let ea = base.wrapping_add(index);
    
    // For now, treat as a regular load
    // Real hardware would use EAR register and external control bus
    let value = memory.read_u32(ea)?;
    regs.gpr[rt as usize] = value;
    
    Ok(())
}

/// External Control Out Word Indexed
/// ecowx RS, RA, RB
/// Store word using external control (device-specific)
pub fn ecowx(regs: &mut Registers, memory: &dyn crate::MemoryInterface, rs: u8, ra: u8, rb: u8) -> Result<()> {
    // Calculate effective address
    let base = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let index = regs.gpr[rb as usize];
    let ea = base.wrapping_add(index);
    
    // For now, treat as a regular store
    // Real hardware would use EAR register and external control bus
    let value = regs.gpr[rs as usize];
    memory.write_u32(ea, value)?;
    
    Ok(())
}

// ============================================================================
// Cache Management (dcbi)
// ============================================================================

/// Data Cache Block Invalidate
/// dcbi RA, RB
/// Invalidate data cache block
pub fn dcbi(_regs: &mut Registers, _ra: u8, _rb: u8) -> Result<()> {
    // No-op: interpreter doesn't have a data cache
    // In a real implementation with JIT, this might flush cached blocks
    Ok(())
}
