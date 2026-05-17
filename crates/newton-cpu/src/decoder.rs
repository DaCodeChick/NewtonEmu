// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! PowerPC instruction decoder

use newton_utils::Result;

/// Decoded PowerPC instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    // Integer arithmetic
    Add { rt: u8, ra: u8, rb: u8, oe: bool, rc: bool },
    Addi { rt: u8, ra: u8, simm: i16 },
    Addis { rt: u8, ra: u8, simm: i16 },
    Subf { rt: u8, ra: u8, rb: u8, oe: bool, rc: bool },
    
    // Logical operations
    And { ra: u8, rs: u8, rb: u8, rc: bool },
    Or { ra: u8, rs: u8, rb: u8, rc: bool },
    Xor { ra: u8, rs: u8, rb: u8, rc: bool },
    
    // Load/Store
    Lwz { rt: u8, ra: u8, d: i16 },
    Lwzu { rt: u8, ra: u8, d: i16 },
    Stw { rs: u8, ra: u8, d: i16 },
    Stwu { rs: u8, ra: u8, d: i16 },
    
    // Branch
    B { li: i32, aa: bool, lk: bool },
    Bc { bo: u8, bi: u8, bd: i16, aa: bool, lk: bool },
    Bcctr { bo: u8, bi: u8, lk: bool },
    Bclr { bo: u8, bi: u8, lk: bool },
    
    // Special
    Nop,
    Unknown { opcode: u32 },
}

/// Decode a PowerPC instruction
pub fn decode_instruction(instr: u32) -> Result<Instruction> {
    let opcode = (instr >> 26) & 0x3F;
    
    match opcode {
        // Primary opcodes
        14 => decode_addi(instr),
        15 => decode_addis(instr),
        24 => decode_ori(instr),
        32 => decode_lwz(instr),
        33 => decode_lwzu(instr),
        36 => decode_stw(instr),
        37 => decode_stwu(instr),
        18 => decode_b(instr),
        16 => decode_bc(instr),
        
        // Extended opcodes (opcode 31)
        31 => decode_extended_31(instr),
        
        // Extended opcodes (opcode 19)
        19 => decode_extended_19(instr),
        
        _ => Ok(Instruction::Unknown { opcode: instr }),
    }
}

// Helper functions to extract instruction fields
#[inline]
fn field_rt(instr: u32) -> u8 { ((instr >> 21) & 0x1F) as u8 }
#[inline]
fn field_rs(instr: u32) -> u8 { ((instr >> 21) & 0x1F) as u8 }
#[inline]
fn field_ra(instr: u32) -> u8 { ((instr >> 16) & 0x1F) as u8 }
#[inline]
fn field_rb(instr: u32) -> u8 { ((instr >> 11) & 0x1F) as u8 }
#[inline]
fn field_simm(instr: u32) -> i16 { (instr & 0xFFFF) as i16 }
#[inline]
fn field_rc(instr: u32) -> bool { (instr & 1) != 0 }
#[inline]
fn field_oe(instr: u32) -> bool { ((instr >> 10) & 1) != 0 }
#[inline]
fn field_bo(instr: u32) -> u8 { ((instr >> 21) & 0x1F) as u8 }
#[inline]
fn field_bi(instr: u32) -> u8 { ((instr >> 16) & 0x1F) as u8 }
#[inline]
fn field_aa(instr: u32) -> bool { ((instr >> 1) & 1) != 0 }
#[inline]
fn field_lk(instr: u32) -> bool { (instr & 1) != 0 }

fn decode_addi(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Addi {
        rt: field_rt(instr),
        ra: field_ra(instr),
        simm: field_simm(instr),
    })
}

fn decode_addis(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Addis {
        rt: field_rt(instr),
        ra: field_ra(instr),
        simm: field_simm(instr),
    })
}

fn decode_ori(instr: u32) -> Result<Instruction> {
    // ori is actually encoded as Or with immediate
    // For now, treat nop (ori 0,0,0) specially
    if instr == 0x60000000 {
        return Ok(Instruction::Nop);
    }
    Ok(Instruction::Unknown { opcode: instr })
}

fn decode_lwz(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Lwz {
        rt: field_rt(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_lwzu(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Lwzu {
        rt: field_rt(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_stw(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Stw {
        rs: field_rs(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_stwu(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Stwu {
        rs: field_rs(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_b(instr: u32) -> Result<Instruction> {
    let li = ((instr & 0x03FFFFFC) as i32) << 6 >> 6; // Sign extend
    Ok(Instruction::B {
        li,
        aa: field_aa(instr),
        lk: field_lk(instr),
    })
}

fn decode_bc(instr: u32) -> Result<Instruction> {
    let bd = (((instr & 0xFFFC) as i16) << 2) >> 2; // Sign extend
    Ok(Instruction::Bc {
        bo: field_bo(instr),
        bi: field_bi(instr),
        bd,
        aa: field_aa(instr),
        lk: field_lk(instr),
    })
}

fn decode_extended_31(instr: u32) -> Result<Instruction> {
    let xo = (instr >> 1) & 0x3FF;
    
    match xo {
        266 => Ok(Instruction::Add {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
            oe: field_oe(instr),
            rc: field_rc(instr),
        }),
        40 => Ok(Instruction::Subf {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
            oe: field_oe(instr),
            rc: field_rc(instr),
        }),
        28 => Ok(Instruction::And {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rb: field_rb(instr),
            rc: field_rc(instr),
        }),
        444 => Ok(Instruction::Or {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rb: field_rb(instr),
            rc: field_rc(instr),
        }),
        316 => Ok(Instruction::Xor {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rb: field_rb(instr),
            rc: field_rc(instr),
        }),
        _ => Ok(Instruction::Unknown { opcode: instr }),
    }
}

fn decode_extended_19(instr: u32) -> Result<Instruction> {
    let xo = (instr >> 1) & 0x3FF;
    
    match xo {
        16 => Ok(Instruction::Bclr {
            bo: field_bo(instr),
            bi: field_bi(instr),
            lk: field_lk(instr),
        }),
        528 => Ok(Instruction::Bcctr {
            bo: field_bo(instr),
            bi: field_bi(instr),
            lk: field_lk(instr),
        }),
        _ => Ok(Instruction::Unknown { opcode: instr }),
    }
}
