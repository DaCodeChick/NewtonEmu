// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
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
mod floating;
mod system;
mod crlogical;
mod altivec;

use crate::registers::{Registers, ConditionRegister, Xer};
use crate::decoder::Instruction;
use crate::MemoryInterface;
use crate::exec_result::ExecResult;
use newton_utils::{Result, Error};

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
            
            // CR logical operations
            Crand { bt, ba, bb } => { crlogical::crand(regs, bt, ba, bb)?; Ok(ExecResult::Continue) }
            Crandc { bt, ba, bb } => { crlogical::crandc(regs, bt, ba, bb)?; Ok(ExecResult::Continue) }
            Creqv { bt, ba, bb } => { crlogical::creqv(regs, bt, ba, bb)?; Ok(ExecResult::Continue) }
            Crnand { bt, ba, bb } => { crlogical::crnand(regs, bt, ba, bb)?; Ok(ExecResult::Continue) }
            Crnor { bt, ba, bb } => { crlogical::crnor(regs, bt, ba, bb)?; Ok(ExecResult::Continue) }
            Cror { bt, ba, bb } => { crlogical::cror(regs, bt, ba, bb)?; Ok(ExecResult::Continue) }
            Crorc { bt, ba, bb } => { crlogical::crorc(regs, bt, ba, bb)?; Ok(ExecResult::Continue) }
            Crxor { bt, ba, bb } => { crlogical::crxor(regs, bt, ba, bb)?; Ok(ExecResult::Continue) }
            
            // Floating-point arithmetic
            Fadd { frt, fra, frb, rc } => { floating::fadd(regs, frt, fra, frb, rc)?; Ok(ExecResult::Continue) }
            Fadds { frt, fra, frb, rc } => { floating::fadds(regs, frt, fra, frb, rc)?; Ok(ExecResult::Continue) }
            Fsub { frt, fra, frb, rc } => { floating::fsub(regs, frt, fra, frb, rc)?; Ok(ExecResult::Continue) }
            Fsubs { frt, fra, frb, rc } => { floating::fsubs(regs, frt, fra, frb, rc)?; Ok(ExecResult::Continue) }
            Fmul { frt, fra, frc, rc } => { floating::fmul(regs, frt, fra, frc, rc)?; Ok(ExecResult::Continue) }
            Fmuls { frt, fra, frc, rc } => { floating::fmuls(regs, frt, fra, frc, rc)?; Ok(ExecResult::Continue) }
            Fdiv { frt, fra, frb, rc } => { floating::fdiv(regs, frt, fra, frb, rc)?; Ok(ExecResult::Continue) }
            Fdivs { frt, fra, frb, rc } => { floating::fdivs(regs, frt, fra, frb, rc)?; Ok(ExecResult::Continue) }
            
            // Floating-point unary
            Fneg { frt, frb, rc } => { floating::fneg(regs, frt, frb, rc)?; Ok(ExecResult::Continue) }
            Fabs { frt, frb, rc } => { floating::fabs(regs, frt, frb, rc)?; Ok(ExecResult::Continue) }
            Fmr { frt, frb, rc } => { floating::fmr(regs, frt, frb, rc)?; Ok(ExecResult::Continue) }
            Fsqrt { frt, frb, rc } => { floating::fsqrt(regs, frt, frb, rc)?; Ok(ExecResult::Continue) }
            Fsqrts { frt, frb, rc } => { floating::fsqrts(regs, frt, frb, rc)?; Ok(ExecResult::Continue) }
            
            // Floating-point multiply-add
            Fmadd { frt, fra, frc, frb, rc } => { floating::fmadd(regs, frt, fra, frc, frb, rc)?; Ok(ExecResult::Continue) }
            Fmadds { frt, fra, frc, frb, rc } => { floating::fmadds(regs, frt, fra, frc, frb, rc)?; Ok(ExecResult::Continue) }
            Fmsub { frt, fra, frc, frb, rc } => { floating::fmsub(regs, frt, fra, frc, frb, rc)?; Ok(ExecResult::Continue) }
            Fmsubs { frt, fra, frc, frb, rc } => { floating::fmsubs(regs, frt, fra, frc, frb, rc)?; Ok(ExecResult::Continue) }
            Fnmadd { frt, fra, frc, frb, rc } => { floating::fnmadd(regs, frt, fra, frc, frb, rc)?; Ok(ExecResult::Continue) }
            Fnmadds { frt, fra, frc, frb, rc } => { floating::fnmadds(regs, frt, fra, frc, frb, rc)?; Ok(ExecResult::Continue) }
            Fnmsub { frt, fra, frc, frb, rc } => { floating::fnmsub(regs, frt, fra, frc, frb, rc)?; Ok(ExecResult::Continue) }
            Fnmsubs { frt, fra, frc, frb, rc } => { floating::fnmsubs(regs, frt, fra, frc, frb, rc)?; Ok(ExecResult::Continue) }
            
            // Floating-point compare
            Fcmpu { crfd, fra, frb } => { floating::fcmpu(regs, crfd, fra, frb)?; Ok(ExecResult::Continue) }
            Fcmpo { crfd, fra, frb } => { floating::fcmpo(regs, crfd, fra, frb)?; Ok(ExecResult::Continue) }
            
            // Floating-point conversion
            Fctiwz { frt, frb, rc } => { floating::fctiwz(regs, frt, frb, rc)?; Ok(ExecResult::Continue) }
            Frsp { frt, frb, rc } => { floating::frsp(regs, frt, frb, rc)?; Ok(ExecResult::Continue) }
            
            // Floating-point load/store need memory interface
            Lfd { .. } | Lfdu { .. } | Lfs { .. } | Lfsu { .. } |
            Stfd { .. } | Stfdu { .. } | Stfs { .. } | Stfsu { .. } => {
                tracing::warn!("FP load/store instruction called without memory interface: {:?}", instr);
                Ok(ExecResult::Continue)
            }
            
            // Branches - These modify PC directly
            B { li, aa, lk } => { branches::b(regs, li, aa, lk)?; Ok(ExecResult::BranchTaken) }
            Bc { bo, bi, bd, aa, lk } => branches::bc(regs, bo, bi, bd, aa, lk),
            Bcctr { bo, bi, lk } => branches::bcctr(regs, bo, bi, lk),
            Bclr { bo, bi, lk } => branches::bclr(regs, bo, bi, lk),
            
            // System instructions (simplified)
            Nop => Ok(ExecResult::Continue),
            Sync | Isync | Eieio => Ok(ExecResult::Continue), // Memory barriers - no-op for now
            Sc => Ok(ExecResult::Syscall),
            Tw { .. } => Ok(ExecResult::Trap), // Trap word (register-based)
            Twi { to, ra, simm } => {
                // Trap word immediate - actually check condition
                system::twi(regs, to, ra, simm)?;
                Ok(ExecResult::Continue)
            }
            Rfi => Ok(ExecResult::Continue), // Return from interrupt - simplified
            
            // SPR instructions
            Mfspr { rt, spr } => { system::mfspr(regs, rt, spr)?; Ok(ExecResult::Continue) }
            Mtspr { spr, rs } => { system::mtspr(regs, spr, rs)?; Ok(ExecResult::Continue) }
            Mftb { rt, tbr } => { system::mftb(regs, rt, tbr)?; Ok(ExecResult::Continue) }
            Mfmsr { rt } => { system::mfmsr(regs, rt)?; Ok(ExecResult::Continue) }
            Mtmsr { rs } => { system::mtmsr(regs, rs)?; Ok(ExecResult::Continue) }
            Mfcr { rt } => { system::mfcr(regs, rt)?; Ok(ExecResult::Continue) }
            Mtcrf { fxm, rs } => { system::mtcrf(regs, fxm, rs)?; Ok(ExecResult::Continue) }
            
            // CR field operations
            Mcrf { crfd, crfs } => { crlogical::mcrf(regs, crfd, crfs)?; Ok(ExecResult::Continue) }
            Mcrxr { crfd } => { crlogical::mcrxr(regs, crfd)?; Ok(ExecResult::Continue) }
            
            // Cache management instructions (no-ops in interpreter)
            Dcbf { ra, rb } => { system::dcbf(regs, ra, rb)?; Ok(ExecResult::Continue) }
            Dcbst { ra, rb } => { system::dcbst(regs, ra, rb)?; Ok(ExecResult::Continue) }
            Dcbt { ra, rb } => { system::dcbt(regs, ra, rb)?; Ok(ExecResult::Continue) }
            Dcbtst { ra, rb } => { system::dcbtst(regs, ra, rb)?; Ok(ExecResult::Continue) }
            Icbi { ra, rb } => { system::icbi(regs, ra, rb)?; Ok(ExecResult::Continue) }
            
            // Segment register instructions
            Mfsr { rt, sr } => { system::mfsr(regs, rt, sr)?; Ok(ExecResult::Continue) }
            Mfsrin { rt, rb } => { system::mfsrin(regs, rt, rb)?; Ok(ExecResult::Continue) }
            Mtsr { sr, rs } => { system::mtsr(regs, sr, rs)?; Ok(ExecResult::Continue) }
            Mtsrin { rs, rb } => { system::mtsrin(regs, rs, rb)?; Ok(ExecResult::Continue) }
            
            // TLB management instructions (no-ops in interpreter)
            Tlbie { rb } => { system::tlbie(regs, rb)?; Ok(ExecResult::Continue) }
            Tlbia => { system::tlbia(regs)?; Ok(ExecResult::Continue) }
            Tlbsync => { system::tlbsync(regs)?; Ok(ExecResult::Continue) },
            
            // AltiVec arithmetic - float
            Vaddfp { vd, va, vb } => { altivec::vaddfp(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vsubfp { vd, va, vb } => { altivec::vsubfp(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vmaddfp { vd, va, vb, vc } => { altivec::vmaddfp(regs, vd, va, vb, vc)?; Ok(ExecResult::Continue) }
            Vnmsubfp { vd, va, vb, vc } => { altivec::vnmsubfp(regs, vd, va, vb, vc)?; Ok(ExecResult::Continue) }
            
            // AltiVec arithmetic - integer modulo
            Vaddubm { vd, va, vb } => { altivec::vaddubm(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vadduhm { vd, va, vb } => { altivec::vadduhm(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vadduwm { vd, va, vb } => { altivec::vadduwm(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vsububm { vd, va, vb } => { altivec::vsububm(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vsubuhm { vd, va, vb } => { altivec::vsubuhm(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vsubuwm { vd, va, vb } => { altivec::vsubuwm(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            
            // AltiVec arithmetic - integer saturate
            Vaddubs { vd, va, vb } => { altivec::vaddubs(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vadduhs { vd, va, vb } => { altivec::vadduhs(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vadduws { vd, va, vb } => { altivec::vadduws(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vaddsbs { vd, va, vb } => { altivec::vaddsbs(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vaddshs { vd, va, vb } => { altivec::vaddshs(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vaddsws { vd, va, vb } => { altivec::vaddsws(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            
            // AltiVec min/max
            Vmaxsb { vd, va, vb } => { altivec::vmaxsb(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vmaxsh { vd, va, vb } => { altivec::vmaxsh(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vmaxsw { vd, va, vb } => { altivec::vmaxsw(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vmaxub { vd, va, vb } => { altivec::vmaxub(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vmaxuh { vd, va, vb } => { altivec::vmaxuh(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vmaxuw { vd, va, vb } => { altivec::vmaxuw(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vmaxfp { vd, va, vb } => { altivec::vmaxfp(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vminsb { vd, va, vb } => { altivec::vminsb(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vminsh { vd, va, vb } => { altivec::vminsh(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vminsw { vd, va, vb } => { altivec::vminsw(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vminub { vd, va, vb } => { altivec::vminub(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vminuh { vd, va, vb } => { altivec::vminuh(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vminuw { vd, va, vb } => { altivec::vminuw(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vminfp { vd, va, vb } => { altivec::vminfp(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            
            // AltiVec logical
            Vand { vd, va, vb } => { altivec::vand(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vandc { vd, va, vb } => { altivec::vandc(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vor { vd, va, vb } => { altivec::vor(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vnor { vd, va, vb } => { altivec::vnor(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vxor { vd, va, vb } => { altivec::vxor(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            
            // AltiVec comparison
            Vcmpequb { vd, va, vb, rc } => { altivec::vcmpequb(regs, vd, va, vb, rc)?; Ok(ExecResult::Continue) }
            Vcmpequh { vd, va, vb, rc } => { altivec::vcmpequh(regs, vd, va, vb, rc)?; Ok(ExecResult::Continue) }
            Vcmpequw { vd, va, vb, rc } => { altivec::vcmpequw(regs, vd, va, vb, rc)?; Ok(ExecResult::Continue) }
            Vcmpeqfp { vd, va, vb, rc } => { altivec::vcmpeqfp(regs, vd, va, vb, rc)?; Ok(ExecResult::Continue) }
            Vcmpgtsb { vd, va, vb, rc } => { altivec::vcmpgtsb(regs, vd, va, vb, rc)?; Ok(ExecResult::Continue) }
            Vcmpgtsh { vd, va, vb, rc } => { altivec::vcmpgtsh(regs, vd, va, vb, rc)?; Ok(ExecResult::Continue) }
            Vcmpgtsw { vd, va, vb, rc } => { altivec::vcmpgtsw(regs, vd, va, vb, rc)?; Ok(ExecResult::Continue) }
            Vcmpgtub { vd, va, vb, rc } => { altivec::vcmpgtub(regs, vd, va, vb, rc)?; Ok(ExecResult::Continue) }
            Vcmpgtuh { vd, va, vb, rc } => { altivec::vcmpgtuh(regs, vd, va, vb, rc)?; Ok(ExecResult::Continue) }
            Vcmpgtuw { vd, va, vb, rc } => { altivec::vcmpgtuw(regs, vd, va, vb, rc)?; Ok(ExecResult::Continue) }
            Vcmpgtfp { vd, va, vb, rc } => { altivec::vcmpgtfp(regs, vd, va, vb, rc)?; Ok(ExecResult::Continue) }
            Vcmpgefp { vd, va, vb, rc } => { altivec::vcmpgefp(regs, vd, va, vb, rc)?; Ok(ExecResult::Continue) }
            
            // AltiVec shift/rotate
            Vslb { vd, va, vb } => { altivec::vslb(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vslh { vd, va, vb } => { altivec::vslh(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vslw { vd, va, vb } => { altivec::vslw(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vsrb { vd, va, vb } => { altivec::vsrb(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vsrh { vd, va, vb } => { altivec::vsrh(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vsrw { vd, va, vb } => { altivec::vsrw(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vsrab { vd, va, vb } => { altivec::vsrab(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vsrah { vd, va, vb } => { altivec::vsrah(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vsraw { vd, va, vb } => { altivec::vsraw(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vrlb { vd, va, vb } => { altivec::vrlb(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vrlh { vd, va, vb } => { altivec::vrlh(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vrlw { vd, va, vb } => { altivec::vrlw(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vsl { vd, va, vb } => { altivec::vsl(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vsr { vd, va, vb } => { altivec::vsr(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            
            // AltiVec permute/merge
            Vperm { vd, va, vb, vc } => { altivec::vperm(regs, vd, va, vb, vc)?; Ok(ExecResult::Continue) }
            Vsel { vd, va, vb, vc } => { altivec::vsel(regs, vd, va, vb, vc)?; Ok(ExecResult::Continue) }
            Vspltb { vd, vb, uimm } => { altivec::vspltb(regs, vd, vb, uimm)?; Ok(ExecResult::Continue) }
            Vsplth { vd, vb, uimm } => { altivec::vsplth(regs, vd, vb, uimm)?; Ok(ExecResult::Continue) }
            Vspltw { vd, vb, uimm } => { altivec::vspltw(regs, vd, vb, uimm)?; Ok(ExecResult::Continue) }
            Vspltisb { vd, simm } => { altivec::vspltisb(regs, vd, simm)?; Ok(ExecResult::Continue) }
            Vspltish { vd, simm } => { altivec::vspltish(regs, vd, simm)?; Ok(ExecResult::Continue) }
            Vspltisw { vd, simm } => { altivec::vspltisw(regs, vd, simm)?; Ok(ExecResult::Continue) }
            Vmrghb { vd, va, vb } => { altivec::vmrghb(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vmrghh { vd, va, vb } => { altivec::vmrghh(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vmrghw { vd, va, vb } => { altivec::vmrghw(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vmrglb { vd, va, vb } => { altivec::vmrglb(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vmrglh { vd, va, vb } => { altivec::vmrglh(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vmrglw { vd, va, vb } => { altivec::vmrglw(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            
            // AltiVec pack/unpack
            Vpkuhus { vd, va, vb } => { altivec::vpkuhus(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vpkuwus { vd, va, vb } => { altivec::vpkuwus(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vpkshss { vd, va, vb } => { altivec::vpkshss(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vpkshus { vd, va, vb } => { altivec::vpkshus(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vpkswss { vd, va, vb } => { altivec::vpkswss(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vpkswus { vd, va, vb } => { altivec::vpkswus(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vpkpx { vd, va, vb } => { altivec::vpkpx(regs, vd, va, vb)?; Ok(ExecResult::Continue) }
            Vupkhsb { vd, vb } => { altivec::vupkhsb(regs, vd, vb)?; Ok(ExecResult::Continue) }
            Vupklsb { vd, vb } => { altivec::vupklsb(regs, vd, vb)?; Ok(ExecResult::Continue) }
            Vupkhsh { vd, vb } => { altivec::vupkhsh(regs, vd, vb)?; Ok(ExecResult::Continue) }
            Vupklsh { vd, vb } => { altivec::vupklsh(regs, vd, vb)?; Ok(ExecResult::Continue) }
            Vupkhpx { vd, vb } => { altivec::vupkhpx(regs, vd, vb)?; Ok(ExecResult::Continue) }
            Vupklpx { vd, vb } => { altivec::vupklpx(regs, vd, vb)?; Ok(ExecResult::Continue) }
            
            // Load/Store instructions need memory interface
            _ if matches!(instr, Lwz { .. } | Lwzu { .. } | Lbz { .. } | Lhz { .. } |
                                  Stw { .. } | Stwu { .. } | Stb { .. } | Sth { .. } |
                                  Lvx { .. } | Stvx { .. } | Lvxl { .. } | Stvxl { .. } |
                                  Lvebx { .. } | Lvehx { .. } | Lvewx { .. } |
                                  Stvebx { .. } | Stvehx { .. } | Stvewx { .. } |
                                  Lvsl { .. } | Lvsr { .. }) => {
                tracing::warn!("Load/store instruction called without memory interface: {:?}", instr);
                Ok(ExecResult::Continue)
            }
            
            // Unimplemented
            Unknown { opcode } => {
                tracing::error!("Unknown instruction: 0x{:08X} at PC 0x{:08X}", opcode, regs.pc);
                Err(Error::Cpu(format!("Unknown instruction: 0x{:08X} at PC 0x{:08X}", opcode, regs.pc)))
            }
            _ => {
                tracing::debug!("Unimplemented instruction: {:?} at PC 0x{:08X}", instr, regs.pc);
                Ok(ExecResult::Continue)
            }
        }
    }

    /// Execute a decoded instruction with memory access
    pub fn execute_with_memory(&mut self, instr: Instruction, regs: &mut Registers, memory: &dyn MemoryInterface) -> Result<ExecResult> {
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
            
            // Floating-point loads
            Lfd { frt, ra, d } => { loadstore::lfd(regs, memory, frt, ra, d)?; Ok(ExecResult::Continue) }
            Lfdu { frt, ra, d } => { loadstore::lfdu(regs, memory, frt, ra, d)?; Ok(ExecResult::Continue) }
            Lfs { frt, ra, d } => { loadstore::lfs(regs, memory, frt, ra, d)?; Ok(ExecResult::Continue) }
            Lfsu { frt, ra, d } => { loadstore::lfsu(regs, memory, frt, ra, d)?; Ok(ExecResult::Continue) }
            
            // Floating-point stores
            Stfd { frs, ra, d } => { loadstore::stfd(regs, memory, frs, ra, d)?; Ok(ExecResult::Continue) }
            Stfdu { frs, ra, d } => { loadstore::stfdu(regs, memory, frs, ra, d)?; Ok(ExecResult::Continue) }
            Stfs { frs, ra, d } => { loadstore::stfs(regs, memory, frs, ra, d)?; Ok(ExecResult::Continue) }
            Stfsu { frs, ra, d } => { loadstore::stfsu(regs, memory, frs, ra, d)?; Ok(ExecResult::Continue) }
            
            // Multiple/String load-store
            Lmw { rt, ra, d } => { loadstore::lmw(regs, memory, rt, ra, d)?; Ok(ExecResult::Continue) }
            Stmw { rs, ra, d } => { loadstore::stmw(regs, memory, rs, ra, d)?; Ok(ExecResult::Continue) }
            Lswi { rt, ra, nb } => { loadstore::lswi(regs, memory, rt, ra, nb)?; Ok(ExecResult::Continue) }
            Lswx { rt, ra, rb } => { loadstore::lswx(regs, memory, rt, ra, rb)?; Ok(ExecResult::Continue) }
            Stswi { rs, ra, nb } => { loadstore::stswi(regs, memory, rs, ra, nb)?; Ok(ExecResult::Continue) }
            Stswx { rs, ra, rb } => { loadstore::stswx(regs, memory, rs, ra, rb)?; Ok(ExecResult::Continue) }
            
            // Byte-reversed loads/stores
            Lwbrx { rt, ra, rb } => { loadstore::lwbrx(regs, memory, rt, ra, rb)?; Ok(ExecResult::Continue) }
            Lhbrx { rt, ra, rb } => { loadstore::lhbrx(regs, memory, rt, ra, rb)?; Ok(ExecResult::Continue) }
            Stwbrx { rs, ra, rb } => { loadstore::stwbrx(regs, memory, rs, ra, rb)?; Ok(ExecResult::Continue) }
            Sthbrx { rs, ra, rb } => { loadstore::sthbrx(regs, memory, rs, ra, rb)?; Ok(ExecResult::Continue) }
            
            // Atomic load/store with reservation
            Lwarx { rt, ra, rb } => { loadstore::lwarx(regs, memory, rt, ra, rb)?; Ok(ExecResult::Continue) }
            Stwcx { rs, ra, rb } => { loadstore::stwcx(regs, memory, rs, ra, rb)?; Ok(ExecResult::Continue) }
            
            // External control in/out
            Eciwx { rt, ra, rb } => { system::eciwx(regs, memory, rt, ra, rb)?; Ok(ExecResult::Continue) }
            Ecowx { rs, ra, rb } => { system::ecowx(regs, memory, rs, ra, rb)?; Ok(ExecResult::Continue) }
            
            // Cache management (dcbz, dcbi write memory or have side effects)
            Dcbz { ra, rb } => { system::dcbz(regs, ra, rb, memory)?; Ok(ExecResult::Continue) }
            Dcbi { ra, rb } => { system::dcbi(regs, ra, rb)?; Ok(ExecResult::Continue) }
            
            // AltiVec memory operations
            Lvx { vd, ra, rb } => { altivec::lvx(regs, memory, vd, ra, rb)?; Ok(ExecResult::Continue) }
            Stvx { vs, ra, rb } => { altivec::stvx(regs, memory, vs, ra, rb)?; Ok(ExecResult::Continue) }
            Lvxl { vd, ra, rb } => { altivec::lvxl(regs, memory, vd, ra, rb)?; Ok(ExecResult::Continue) }
            Stvxl { vs, ra, rb } => { altivec::stvxl(regs, memory, vs, ra, rb)?; Ok(ExecResult::Continue) }
            Lvebx { vd, ra, rb } => { altivec::lvebx(regs, memory, vd, ra, rb)?; Ok(ExecResult::Continue) }
            Lvehx { vd, ra, rb } => { altivec::lvehx(regs, memory, vd, ra, rb)?; Ok(ExecResult::Continue) }
            Lvewx { vd, ra, rb } => { altivec::lvewx(regs, memory, vd, ra, rb)?; Ok(ExecResult::Continue) }
            Stvebx { vs, ra, rb } => { altivec::stvebx(regs, memory, vs, ra, rb)?; Ok(ExecResult::Continue) }
            Stvehx { vs, ra, rb } => { altivec::stvehx(regs, memory, vs, ra, rb)?; Ok(ExecResult::Continue) }
            Stvewx { vs, ra, rb } => { altivec::stvewx(regs, memory, vs, ra, rb)?; Ok(ExecResult::Continue) }
            Lvsl { vd, ra, rb } => { altivec::lvsl(regs, memory, vd, ra, rb)?; Ok(ExecResult::Continue) }
            Lvsr { vd, ra, rb } => { altivec::lvsr(regs, memory, vd, ra, rb)?; Ok(ExecResult::Continue) }
            
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
