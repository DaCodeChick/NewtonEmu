// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! PowerPC instruction interpreter

mod arithmetic;
mod logical;
mod shifts;
mod compare;
mod branches;
mod loadstore;

use crate::registers::{Registers, ConditionRegister, Xer};
use crate::decoder::Instruction;
use crate::MemoryInterface;
use crate::exec_result::ExecResult;
use newton_utils::Result;

/// Instruction interpreter
pub struct Interpreter {
    // State will be added as needed
}

impl Interpreter {
    /// Create a new interpreter
    pub fn new() -> Self {
        Self {}
    }

    /// Execute a decoded instruction without memory access (for non-load/store instructions)
    pub fn execute(&mut self, instr: Instruction, regs: &mut Registers) -> Result<ExecResult> {
        use Instruction::*;
        
        match instr {
            // Integer arithmetic
            Add { rt, ra, rb, oe, rc } => { arithmetic::add(regs, rt, ra, rb, oe, rc)?; Ok(ExecResult::Continue) }
            Addc { rt, ra, rb, oe, rc } => { arithmetic::addc(regs, rt, ra, rb, oe, rc)?; Ok(ExecResult::Continue) }
            Adde { rt, ra, rb, oe, rc } => { arithmetic::adde(regs, rt, ra, rb, oe, rc)?; Ok(ExecResult::Continue) }
            Addi { rt, ra, simm } => { arithmetic::addi(regs, rt, ra, simm)?; Ok(ExecResult::Continue) }
            Addic { rt, ra, simm } => { arithmetic::addic(regs, rt, ra, simm)?; Ok(ExecResult::Continue) }
            AddicDot { rt, ra, simm } => { arithmetic::addic_dot(regs, rt, ra, simm)?; Ok(ExecResult::Continue) }
            Addis { rt, ra, simm } => { arithmetic::addis(regs, rt, ra, simm)?; Ok(ExecResult::Continue) }
            Addme { rt, ra, oe, rc } => { arithmetic::addme(regs, rt, ra, oe, rc)?; Ok(ExecResult::Continue) }
            Addze { rt, ra, oe, rc } => { arithmetic::addze(regs, rt, ra, oe, rc)?; Ok(ExecResult::Continue) }
            Divw { rt, ra, rb, oe, rc } => { arithmetic::divw(regs, rt, ra, rb, oe, rc)?; Ok(ExecResult::Continue) }
            Divwu { rt, ra, rb, oe, rc } => { arithmetic::divwu(regs, rt, ra, rb, oe, rc)?; Ok(ExecResult::Continue) }
            Mulhw { rt, ra, rb, rc } => { arithmetic::mulhw(regs, rt, ra, rb, rc)?; Ok(ExecResult::Continue) }
            Mulhwu { rt, ra, rb, rc } => { arithmetic::mulhwu(regs, rt, ra, rb, rc)?; Ok(ExecResult::Continue) }
            Mulli { rt, ra, simm } => { arithmetic::mulli(regs, rt, ra, simm)?; Ok(ExecResult::Continue) }
            Mullw { rt, ra, rb, oe, rc } => { arithmetic::mullw(regs, rt, ra, rb, oe, rc)?; Ok(ExecResult::Continue) }
            Neg { rt, ra, oe, rc } => { arithmetic::neg(regs, rt, ra, oe, rc)?; Ok(ExecResult::Continue) }
            Subf { rt, ra, rb, oe, rc } => { arithmetic::subf(regs, rt, ra, rb, oe, rc)?; Ok(ExecResult::Continue) }
            Subfc { rt, ra, rb, oe, rc } => { arithmetic::subfc(regs, rt, ra, rb, oe, rc)?; Ok(ExecResult::Continue) }
            Subfe { rt, ra, rb, oe, rc } => { arithmetic::subfe(regs, rt, ra, rb, oe, rc)?; Ok(ExecResult::Continue) }
            Subfic { rt, ra, simm } => { arithmetic::subfic(regs, rt, ra, simm)?; Ok(ExecResult::Continue) }
            Subfme { rt, ra, oe, rc } => { arithmetic::subfme(regs, rt, ra, oe, rc)?; Ok(ExecResult::Continue) }
            Subfze { rt, ra, oe, rc } => { arithmetic::subfze(regs, rt, ra, oe, rc)?; Ok(ExecResult::Continue) }
            
            // Logical operations
            And { ra, rs, rb, rc } => { logical::and(regs, ra, rs, rb, rc)?; Ok(ExecResult::Continue) }
            Andc { ra, rs, rb, rc } => { logical::andc(regs, ra, rs, rb, rc)?; Ok(ExecResult::Continue) }
            Andi { ra, rs, uimm } => { logical::andi(regs, ra, rs, uimm)?; Ok(ExecResult::Continue) }
            Andis { ra, rs, uimm } => { logical::andis(regs, ra, rs, uimm)?; Ok(ExecResult::Continue) }
            Cntlzw { ra, rs, rc } => { logical::cntlzw(regs, ra, rs, rc)?; Ok(ExecResult::Continue) }
            Eqv { ra, rs, rb, rc } => { logical::eqv(regs, ra, rs, rb, rc)?; Ok(ExecResult::Continue) }
            Extsb { ra, rs, rc } => { logical::extsb(regs, ra, rs, rc)?; Ok(ExecResult::Continue) }
            Extsh { ra, rs, rc } => { logical::extsh(regs, ra, rs, rc)?; Ok(ExecResult::Continue) }
            Nand { ra, rs, rb, rc } => { logical::nand(regs, ra, rs, rb, rc)?; Ok(ExecResult::Continue) }
            Nor { ra, rs, rb, rc } => { logical::nor(regs, ra, rs, rb, rc)?; Ok(ExecResult::Continue) }
            Or { ra, rs, rb, rc } => { logical::or(regs, ra, rs, rb, rc)?; Ok(ExecResult::Continue) }
            Orc { ra, rs, rb, rc } => { logical::orc(regs, ra, rs, rb, rc)?; Ok(ExecResult::Continue) }
            Ori { ra, rs, uimm } => { logical::ori(regs, ra, rs, uimm)?; Ok(ExecResult::Continue) }
            Oris { ra, rs, uimm } => { logical::oris(regs, ra, rs, uimm)?; Ok(ExecResult::Continue) }
            Xor { ra, rs, rb, rc } => { logical::xor(regs, ra, rs, rb, rc)?; Ok(ExecResult::Continue) }
            Xori { ra, rs, uimm } => { logical::xori(regs, ra, rs, uimm)?; Ok(ExecResult::Continue) }
            Xoris { ra, rs, uimm } => { logical::xoris(regs, ra, rs, uimm)?; Ok(ExecResult::Continue) }
            
            // Rotate and shift
            Rlwimi { ra, rs, sh, mb, me, rc } => { shifts::rlwimi(regs, ra, rs, sh, mb, me, rc)?; Ok(ExecResult::Continue) }
            Rlwinm { ra, rs, sh, mb, me, rc } => { shifts::rlwinm(regs, ra, rs, sh, mb, me, rc)?; Ok(ExecResult::Continue) }
            Rlwnm { ra, rs, rb, mb, me, rc } => { shifts::rlwnm(regs, ra, rs, rb, mb, me, rc)?; Ok(ExecResult::Continue) }
            Slw { ra, rs, rb, rc } => { shifts::slw(regs, ra, rs, rb, rc)?; Ok(ExecResult::Continue) }
            Sraw { ra, rs, rb, rc } => { shifts::sraw(regs, ra, rs, rb, rc)?; Ok(ExecResult::Continue) }
            Srawi { ra, rs, sh, rc } => { shifts::srawi(regs, ra, rs, sh, rc)?; Ok(ExecResult::Continue) }
            Srw { ra, rs, rb, rc } => { shifts::srw(regs, ra, rs, rb, rc)?; Ok(ExecResult::Continue) }
            
            // Comparison
            Cmp { crfd, l, ra, rb } => { compare::cmp(regs, crfd, l, ra, rb)?; Ok(ExecResult::Continue) }
            Cmpi { crfd, l, ra, simm } => { compare::cmpi(regs, crfd, l, ra, simm)?; Ok(ExecResult::Continue) }
            Cmpl { crfd, l, ra, rb } => { compare::cmpl(regs, crfd, l, ra, rb)?; Ok(ExecResult::Continue) }
            Cmpli { crfd, l, ra, uimm } => { compare::cmpli(regs, crfd, l, ra, uimm)?; Ok(ExecResult::Continue) }
            
            // Branches - These modify PC directly
            B { li, aa, lk } => { branches::b(regs, li, aa, lk)?; Ok(ExecResult::BranchTaken) }
            Bc { bo, bi, bd, aa, lk } => branches::bc(regs, bo, bi, bd, aa, lk),
            Bcctr { bo, bi, lk } => branches::bcctr(regs, bo, bi, lk),
            Bclr { bo, bi, lk } => branches::bclr(regs, bo, bi, lk),
            
            // System instructions (simplified)
            Nop => Ok(ExecResult::Continue),
            Sync | Isync | Eieio => Ok(ExecResult::Continue), // Memory barriers - no-op for now
            Sc => Ok(ExecResult::Syscall),
            Tw { .. } | Twi { .. } => Ok(ExecResult::Trap),
            
            // Load/Store instructions need memory interface
            _ if matches!(instr, Lwz { .. } | Lwzu { .. } | Lbz { .. } | Lhz { .. } |
                                  Stw { .. } | Stwu { .. } | Stb { .. } | Sth { .. }) => {
                tracing::warn!("Load/store instruction called without memory interface: {:?}", instr);
                Ok(ExecResult::Continue)
            }
            
            // Unimplemented
            Unknown { opcode } => {
                tracing::warn!("Unknown instruction: 0x{:08X} at PC 0x{:08X}", opcode, regs.pc);
                Ok(ExecResult::Continue)
            }
            _ => {
                tracing::debug!("Unimplemented instruction: {:?} at PC 0x{:08X}", instr, regs.pc);
                Ok(ExecResult::Continue)
            }
        }
    }

    /// Execute a decoded instruction with memory access
    pub fn execute_with_memory(&mut self, instr: Instruction, regs: &mut Registers, memory: &mut dyn MemoryInterface) -> Result<ExecResult> {
        use Instruction::*;
        
        // Handle load/store instructions that need memory
        match instr {
            // Word loads
            Lwz { rt, ra, d } => { loadstore::lwz(regs, memory, rt, ra, d)?; Ok(ExecResult::Continue) }
            Lwzu { rt, ra, d } => { loadstore::lwzu(regs, memory, rt, ra, d)?; Ok(ExecResult::Continue) }
            Lwzx { rt, ra, rb } => { loadstore::lwzx(regs, memory, rt, ra, rb)?; Ok(ExecResult::Continue) }
            Lwzux { rt, ra, rb } => { loadstore::lwzux(regs, memory, rt, ra, rb)?; Ok(ExecResult::Continue) }
            
            // Byte loads
            Lbz { rt, ra, d } => { loadstore::lbz(regs, memory, rt, ra, d)?; Ok(ExecResult::Continue) }
            Lbzu { rt, ra, d } => { loadstore::lbzu(regs, memory, rt, ra, d)?; Ok(ExecResult::Continue) }
            Lbzx { rt, ra, rb } => { loadstore::lbzx(regs, memory, rt, ra, rb)?; Ok(ExecResult::Continue) }
            Lbzux { rt, ra, rb } => { loadstore::lbzux(regs, memory, rt, ra, rb)?; Ok(ExecResult::Continue) }
            
            // Halfword loads
            Lhz { rt, ra, d } => { loadstore::lhz(regs, memory, rt, ra, d)?; Ok(ExecResult::Continue) }
            Lhzu { rt, ra, d } => { loadstore::lhzu(regs, memory, rt, ra, d)?; Ok(ExecResult::Continue) }
            Lhzx { rt, ra, rb } => { loadstore::lhzx(regs, memory, rt, ra, rb)?; Ok(ExecResult::Continue) }
            Lhzux { rt, ra, rb } => { loadstore::lhzux(regs, memory, rt, ra, rb)?; Ok(ExecResult::Continue) }
            Lha { rt, ra, d } => { loadstore::lha(regs, memory, rt, ra, d)?; Ok(ExecResult::Continue) }
            Lhau { rt, ra, d } => { loadstore::lhau(regs, memory, rt, ra, d)?; Ok(ExecResult::Continue) }
            Lhax { rt, ra, rb } => { loadstore::lhax(regs, memory, rt, ra, rb)?; Ok(ExecResult::Continue) }
            Lhaux { rt, ra, rb } => { loadstore::lhaux(regs, memory, rt, ra, rb)?; Ok(ExecResult::Continue) }
            
            // Word stores
            Stw { rs, ra, d } => { loadstore::stw(regs, memory, rs, ra, d)?; Ok(ExecResult::Continue) }
            Stwu { rs, ra, d } => { loadstore::stwu(regs, memory, rs, ra, d)?; Ok(ExecResult::Continue) }
            Stwx { rs, ra, rb } => { loadstore::stwx(regs, memory, rs, ra, rb)?; Ok(ExecResult::Continue) }
            Stwux { rs, ra, rb } => { loadstore::stwux(regs, memory, rs, ra, rb)?; Ok(ExecResult::Continue) }
            
            // Byte stores
            Stb { rs, ra, d } => { loadstore::stb(regs, memory, rs, ra, d)?; Ok(ExecResult::Continue) }
            Stbu { rs, ra, d } => { loadstore::stbu(regs, memory, rs, ra, d)?; Ok(ExecResult::Continue) }
            Stbx { rs, ra, rb } => { loadstore::stbx(regs, memory, rs, ra, rb)?; Ok(ExecResult::Continue) }
            Stbux { rs, ra, rb } => { loadstore::stbux(regs, memory, rs, ra, rb)?; Ok(ExecResult::Continue) }
            
            // Halfword stores
            Sth { rs, ra, d } => { loadstore::sth(regs, memory, rs, ra, d)?; Ok(ExecResult::Continue) }
            Sthu { rs, ra, d } => { loadstore::sthu(regs, memory, rs, ra, d)?; Ok(ExecResult::Continue) }
            Sthx { rs, ra, rb } => { loadstore::sthx(regs, memory, rs, ra, rb)?; Ok(ExecResult::Continue) }
            Sthux { rs, ra, rb } => { loadstore::sthux(regs, memory, rs, ra, rb)?; Ok(ExecResult::Continue) }
            
            // All other instructions don't need memory
            _ => self.execute(instr, regs)
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions used by multiple modules
pub(crate) fn update_cr0(regs: &mut Registers, result: u32) {
    let lt = (result as i32) < 0;
    let gt = (result as i32) > 0;
    let eq = result == 0;
    let so = regs.xer.contains(Xer::SO);
    
    regs.cr.remove(ConditionRegister::CR0_LT | ConditionRegister::CR0_GT | 
                    ConditionRegister::CR0_EQ | ConditionRegister::CR0_SO);
    
    if lt { regs.cr.insert(ConditionRegister::CR0_LT); }
    if gt { regs.cr.insert(ConditionRegister::CR0_GT); }
    if eq { regs.cr.insert(ConditionRegister::CR0_EQ); }
    if so { regs.cr.insert(ConditionRegister::CR0_SO); }
}

pub(crate) fn set_cr_field(regs: &mut Registers, field: u8, lt: bool, gt: bool, eq: bool, so: bool) {
    let shift = 28 - (field * 4);
    let mask = 0xF << shift;
    let mut value = 0u32;
    if lt { value |= 0x8 << shift; }
    if gt { value |= 0x4 << shift; }
    if eq { value |= 0x2 << shift; }
    if so { value |= 0x1 << shift; }
    
    let cr_val = regs.cr.bits();
    regs.cr = ConditionRegister::from_bits_truncate((cr_val & !mask) | value);
}
