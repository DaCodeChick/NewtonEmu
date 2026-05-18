// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
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
    Addc { rt: u8, ra: u8, rb: u8, oe: bool, rc: bool },
    Adde { rt: u8, ra: u8, rb: u8, oe: bool, rc: bool },
    Addi { rt: u8, ra: u8, simm: i16 },
    Addic { rt: u8, ra: u8, simm: i16 },
    AddicDot { rt: u8, ra: u8, simm: i16 },
    Addis { rt: u8, ra: u8, simm: i16 },
    Addme { rt: u8, ra: u8, oe: bool, rc: bool },
    Addze { rt: u8, ra: u8, oe: bool, rc: bool },
    Divw { rt: u8, ra: u8, rb: u8, oe: bool, rc: bool },
    Divwu { rt: u8, ra: u8, rb: u8, oe: bool, rc: bool },
    Mulhw { rt: u8, ra: u8, rb: u8, rc: bool },
    Mulhwu { rt: u8, ra: u8, rb: u8, rc: bool },
    Mulli { rt: u8, ra: u8, simm: i16 },
    Mullw { rt: u8, ra: u8, rb: u8, oe: bool, rc: bool },
    Neg { rt: u8, ra: u8, oe: bool, rc: bool },
    Subf { rt: u8, ra: u8, rb: u8, oe: bool, rc: bool },
    Subfc { rt: u8, ra: u8, rb: u8, oe: bool, rc: bool },
    Subfe { rt: u8, ra: u8, rb: u8, oe: bool, rc: bool },
    Subfic { rt: u8, ra: u8, simm: i16 },
    Subfme { rt: u8, ra: u8, oe: bool, rc: bool },
    Subfze { rt: u8, ra: u8, oe: bool, rc: bool },
    
    // Logical operations
    And { ra: u8, rs: u8, rb: u8, rc: bool },
    Andc { ra: u8, rs: u8, rb: u8, rc: bool },
    Andi { ra: u8, rs: u8, uimm: u16 },
    Andis { ra: u8, rs: u8, uimm: u16 },
    Cntlzw { ra: u8, rs: u8, rc: bool },
    Eqv { ra: u8, rs: u8, rb: u8, rc: bool },
    Extsb { ra: u8, rs: u8, rc: bool },
    Extsh { ra: u8, rs: u8, rc: bool },
    Nand { ra: u8, rs: u8, rb: u8, rc: bool },
    Nor { ra: u8, rs: u8, rb: u8, rc: bool },
    Or { ra: u8, rs: u8, rb: u8, rc: bool },
    Orc { ra: u8, rs: u8, rb: u8, rc: bool },
    Ori { ra: u8, rs: u8, uimm: u16 },
    Oris { ra: u8, rs: u8, uimm: u16 },
    Xor { ra: u8, rs: u8, rb: u8, rc: bool },
    Xori { ra: u8, rs: u8, uimm: u16 },
    Xoris { ra: u8, rs: u8, uimm: u16 },
    
    // Rotate and shift
    Rlwimi { ra: u8, rs: u8, sh: u8, mb: u8, me: u8, rc: bool },
    Rlwinm { ra: u8, rs: u8, sh: u8, mb: u8, me: u8, rc: bool },
    Rlwnm { ra: u8, rs: u8, rb: u8, mb: u8, me: u8, rc: bool },
    Slw { ra: u8, rs: u8, rb: u8, rc: bool },
    Sraw { ra: u8, rs: u8, rb: u8, rc: bool },
    Srawi { ra: u8, rs: u8, sh: u8, rc: bool },
    Srw { ra: u8, rs: u8, rb: u8, rc: bool },
    
    // Comparison
    Cmp { crfd: u8, l: bool, ra: u8, rb: u8 },
    Cmpi { crfd: u8, l: bool, ra: u8, simm: i16 },
    Cmpl { crfd: u8, l: bool, ra: u8, rb: u8 },
    Cmpli { crfd: u8, l: bool, ra: u8, uimm: u16 },
    
    // Load/Store - Word
    Lbz { rt: u8, ra: u8, d: i16 },
    Lbzu { rt: u8, ra: u8, d: i16 },
    Lbzux { rt: u8, ra: u8, rb: u8 },
    Lbzx { rt: u8, ra: u8, rb: u8 },
    Lwarx { rt: u8, ra: u8, rb: u8 },
    Stwcx { rs: u8, ra: u8, rb: u8 },
    Lwbrx { rt: u8, ra: u8, rb: u8 },
    Lhbrx { rt: u8, ra: u8, rb: u8 },
    Stwbrx { rs: u8, ra: u8, rb: u8 },
    Sthbrx { rs: u8, ra: u8, rb: u8 },
    Lha { rt: u8, ra: u8, d: i16 },
    Lhau { rt: u8, ra: u8, d: i16 },
    Lhaux { rt: u8, ra: u8, rb: u8 },
    Lhax { rt: u8, ra: u8, rb: u8 },
    Lhz { rt: u8, ra: u8, d: i16 },
    Lhzu { rt: u8, ra: u8, d: i16 },
    Lhzux { rt: u8, ra: u8, rb: u8 },
    Lhzx { rt: u8, ra: u8, rb: u8 },
    Lwz { rt: u8, ra: u8, d: i16 },
    Lwzu { rt: u8, ra: u8, d: i16 },
    Lwzux { rt: u8, ra: u8, rb: u8 },
    Lwzx { rt: u8, ra: u8, rb: u8 },
    Stb { rs: u8, ra: u8, d: i16 },
    Stbu { rs: u8, ra: u8, d: i16 },
    Stbux { rs: u8, ra: u8, rb: u8 },
    Stbx { rs: u8, ra: u8, rb: u8 },
    Sth { rs: u8, ra: u8, d: i16 },
    Sthu { rs: u8, ra: u8, d: i16 },
    Sthux { rs: u8, ra: u8, rb: u8 },
    Sthx { rs: u8, ra: u8, rb: u8 },
    Stw { rs: u8, ra: u8, d: i16 },
    Stwu { rs: u8, ra: u8, d: i16 },
    Stwux { rs: u8, ra: u8, rb: u8 },
    Stwx { rs: u8, ra: u8, rb: u8 },
    
    // Load/Store Multiple
    Lmw { rt: u8, ra: u8, d: i16 },
    Stmw { rs: u8, ra: u8, d: i16 },
    
    // Load/Store String
    Lswi { rt: u8, ra: u8, nb: u8 },
    Lswx { rt: u8, ra: u8, rb: u8 },
    Stswi { rs: u8, ra: u8, nb: u8 },
    Stswx { rs: u8, ra: u8, rb: u8 },
    
    // Branch
    B { li: i32, aa: bool, lk: bool },
    Bc { bo: u8, bi: u8, bd: i16, aa: bool, lk: bool },
    Bcctr { bo: u8, bi: u8, lk: bool },
    Bclr { bo: u8, bi: u8, lk: bool },
    
    // Condition Register
    Crand { bt: u8, ba: u8, bb: u8 },
    Crandc { bt: u8, ba: u8, bb: u8 },
    Creqv { bt: u8, ba: u8, bb: u8 },
    Crnand { bt: u8, ba: u8, bb: u8 },
    Crnor { bt: u8, ba: u8, bb: u8 },
    Cror { bt: u8, ba: u8, bb: u8 },
    Crorc { bt: u8, ba: u8, bb: u8 },
    Crxor { bt: u8, ba: u8, bb: u8 },
    Mcrf { crfd: u8, crfs: u8 },
    
    // System/Special Purpose Registers
    Mcrxr { crfd: u8 },
    Mfcr { rt: u8 },
    Mfmsr { rt: u8 },
    Mfspr { rt: u8, spr: u16 },
    Mfsr { rt: u8, sr: u8 },
    Mfsrin { rt: u8, rb: u8 },
    Mftb { rt: u8, tbr: u16 },
    Mtcrf { fxm: u8, rs: u8 },
    Mtmsr { rs: u8 },
    Mtspr { spr: u16, rs: u8 },
    Mtsr { sr: u8, rs: u8 },
    Mtsrin { rs: u8, rb: u8 },
    
    // Cache/TLB/External Control
    Dcbf { ra: u8, rb: u8 },
    Dcbi { ra: u8, rb: u8 },
    Dcbst { ra: u8, rb: u8 },
    Dcbt { ra: u8, rb: u8 },
    Dcbtst { ra: u8, rb: u8 },
    Dcbz { ra: u8, rb: u8 },
    Icbi { ra: u8, rb: u8 },
    Tlbie { rb: u8 },
    Tlbia,
    Tlbsync,
    Eciwx { rt: u8, ra: u8, rb: u8 },
    Ecowx { rs: u8, ra: u8, rb: u8 },
    
    // Synchronization
    Eieio,
    Isync,
    Sync,
    
    // Trap
    Tw { to: u8, ra: u8, rb: u8 },
    Twi { to: u8, ra: u8, simm: i16 },
    
    // System Call
    Sc,
    
    // Return from interrupt
    Rfi,
    
    // Floating Point Arithmetic
    Fadd { frt: u8, fra: u8, frb: u8, rc: bool },
    Fadds { frt: u8, fra: u8, frb: u8, rc: bool },
    Fdiv { frt: u8, fra: u8, frb: u8, rc: bool },
    Fdivs { frt: u8, fra: u8, frb: u8, rc: bool },
    Fmul { frt: u8, fra: u8, frc: u8, rc: bool },
    Fmuls { frt: u8, fra: u8, frc: u8, rc: bool },
    Fsub { frt: u8, fra: u8, frb: u8, rc: bool },
    Fsubs { frt: u8, fra: u8, frb: u8, rc: bool },
    
    // Floating Point Unary
    Fneg { frt: u8, frb: u8, rc: bool },
    Fabs { frt: u8, frb: u8, rc: bool },
    Fmr { frt: u8, frb: u8, rc: bool },
    Fsqrt { frt: u8, frb: u8, rc: bool },
    Fsqrts { frt: u8, frb: u8, rc: bool },
    
    // Floating Point Multiply-Add
    Fmadd { frt: u8, fra: u8, frc: u8, frb: u8, rc: bool },
    Fmadds { frt: u8, fra: u8, frc: u8, frb: u8, rc: bool },
    Fmsub { frt: u8, fra: u8, frc: u8, frb: u8, rc: bool },
    Fmsubs { frt: u8, fra: u8, frc: u8, frb: u8, rc: bool },
    Fnmadd { frt: u8, fra: u8, frc: u8, frb: u8, rc: bool },
    Fnmadds { frt: u8, fra: u8, frc: u8, frb: u8, rc: bool },
    Fnmsub { frt: u8, fra: u8, frc: u8, frb: u8, rc: bool },
    Fnmsubs { frt: u8, fra: u8, frc: u8, frb: u8, rc: bool },
    
    // Floating Point Compare
    Fcmpu { crfd: u8, fra: u8, frb: u8 },
    Fcmpo { crfd: u8, fra: u8, frb: u8 },
    
    // Floating Point Conversion
    Fctiwz { frt: u8, frb: u8, rc: bool },
    Frsp { frt: u8, frb: u8, rc: bool },
    
    // Floating Point Load/Store
    Lfd { frt: u8, ra: u8, d: i16 },
    Lfdu { frt: u8, ra: u8, d: i16 },
    Lfs { frt: u8, ra: u8, d: i16 },
    Lfsu { frt: u8, ra: u8, d: i16 },
    Stfd { frs: u8, ra: u8, d: i16 },
    Stfdu { frs: u8, ra: u8, d: i16 },
    Stfs { frs: u8, ra: u8, d: i16 },
    Stfsu { frs: u8, ra: u8, d: i16 },
    
    // AltiVec Arithmetic
    Vaddfp { vd: u8, va: u8, vb: u8 },
    Vsubfp { vd: u8, va: u8, vb: u8 },
    Vmaddfp { vd: u8, va: u8, vb: u8, vc: u8 },
    Vnmsubfp { vd: u8, va: u8, vb: u8, vc: u8 },
    Vaddubm { vd: u8, va: u8, vb: u8 },
    Vadduhm { vd: u8, va: u8, vb: u8 },
    Vadduwm { vd: u8, va: u8, vb: u8 },
    Vaddubs { vd: u8, va: u8, vb: u8 },
    Vadduhs { vd: u8, va: u8, vb: u8 },
    Vadduws { vd: u8, va: u8, vb: u8 },
    Vaddsbs { vd: u8, va: u8, vb: u8 },
    Vaddshs { vd: u8, va: u8, vb: u8 },
    Vaddsws { vd: u8, va: u8, vb: u8 },
    Vsububm { vd: u8, va: u8, vb: u8 },
    Vsubuhm { vd: u8, va: u8, vb: u8 },
    Vsubuwm { vd: u8, va: u8, vb: u8 },
    Vmaxsb { vd: u8, va: u8, vb: u8 },
    Vmaxsh { vd: u8, va: u8, vb: u8 },
    Vmaxsw { vd: u8, va: u8, vb: u8 },
    Vmaxub { vd: u8, va: u8, vb: u8 },
    Vmaxuh { vd: u8, va: u8, vb: u8 },
    Vmaxuw { vd: u8, va: u8, vb: u8 },
    Vmaxfp { vd: u8, va: u8, vb: u8 },
    Vminsb { vd: u8, va: u8, vb: u8 },
    Vminsh { vd: u8, va: u8, vb: u8 },
    Vminsw { vd: u8, va: u8, vb: u8 },
    Vminub { vd: u8, va: u8, vb: u8 },
    Vminuh { vd: u8, va: u8, vb: u8 },
    Vminuw { vd: u8, va: u8, vb: u8 },
    Vminfp { vd: u8, va: u8, vb: u8 },
    Vavgub { vd: u8, va: u8, vb: u8 },
    Vavguh { vd: u8, va: u8, vb: u8 },
    Vavguw { vd: u8, va: u8, vb: u8 },
    Vavgsb { vd: u8, va: u8, vb: u8 },
    Vavgsh { vd: u8, va: u8, vb: u8 },
    Vavgsw { vd: u8, va: u8, vb: u8 },
    Vmuleub { vd: u8, va: u8, vb: u8 },
    Vmuloub { vd: u8, va: u8, vb: u8 },
    Vmulesb { vd: u8, va: u8, vb: u8 },
    Vmulosb { vd: u8, va: u8, vb: u8 },
    Vmuleuh { vd: u8, va: u8, vb: u8 },
    Vmulouh { vd: u8, va: u8, vb: u8 },
    Vmulesh { vd: u8, va: u8, vb: u8 },
    Vmulosh { vd: u8, va: u8, vb: u8 },
    
    // AltiVec Logical
    Vand { vd: u8, va: u8, vb: u8 },
    Vandc { vd: u8, va: u8, vb: u8 },
    Vor { vd: u8, va: u8, vb: u8 },
    Vnor { vd: u8, va: u8, vb: u8 },
    Vxor { vd: u8, va: u8, vb: u8 },
    
    // AltiVec Comparison
    Vcmpequb { vd: u8, va: u8, vb: u8, rc: bool },
    Vcmpequh { vd: u8, va: u8, vb: u8, rc: bool },
    Vcmpequw { vd: u8, va: u8, vb: u8, rc: bool },
    Vcmpeqfp { vd: u8, va: u8, vb: u8, rc: bool },
    Vcmpgtsb { vd: u8, va: u8, vb: u8, rc: bool },
    Vcmpgtsh { vd: u8, va: u8, vb: u8, rc: bool },
    Vcmpgtsw { vd: u8, va: u8, vb: u8, rc: bool },
    Vcmpgtub { vd: u8, va: u8, vb: u8, rc: bool },
    Vcmpgtuh { vd: u8, va: u8, vb: u8, rc: bool },
    Vcmpgtuw { vd: u8, va: u8, vb: u8, rc: bool },
    Vcmpgtfp { vd: u8, va: u8, vb: u8, rc: bool },
    Vcmpgefp { vd: u8, va: u8, vb: u8, rc: bool },
    
    // AltiVec Shift/Rotate
    Vslb { vd: u8, va: u8, vb: u8 },
    Vslh { vd: u8, va: u8, vb: u8 },
    Vslw { vd: u8, va: u8, vb: u8 },
    Vsrb { vd: u8, va: u8, vb: u8 },
    Vsrh { vd: u8, va: u8, vb: u8 },
    Vsrw { vd: u8, va: u8, vb: u8 },
    Vsrab { vd: u8, va: u8, vb: u8 },
    Vsrah { vd: u8, va: u8, vb: u8 },
    Vsraw { vd: u8, va: u8, vb: u8 },
    Vrlb { vd: u8, va: u8, vb: u8 },
    Vrlh { vd: u8, va: u8, vb: u8 },
    Vrlw { vd: u8, va: u8, vb: u8 },
    Vsl { vd: u8, va: u8, vb: u8 },
    Vsr { vd: u8, va: u8, vb: u8 },
    
    // AltiVec Permute/Merge
    Vperm { vd: u8, va: u8, vb: u8, vc: u8 },
    Vsel { vd: u8, va: u8, vb: u8, vc: u8 },
    Vspltb { vd: u8, vb: u8, uimm: u8 },
    Vsplth { vd: u8, vb: u8, uimm: u8 },
    Vspltw { vd: u8, vb: u8, uimm: u8 },
    Vspltisb { vd: u8, simm: i8 },
    Vspltish { vd: u8, simm: i8 },
    Vspltisw { vd: u8, simm: i8 },
    Vmrghb { vd: u8, va: u8, vb: u8 },
    Vmrghh { vd: u8, va: u8, vb: u8 },
    Vmrghw { vd: u8, va: u8, vb: u8 },
    Vmrglb { vd: u8, va: u8, vb: u8 },
    Vmrglh { vd: u8, va: u8, vb: u8 },
    Vmrglw { vd: u8, va: u8, vb: u8 },
    
    // AltiVec Pack/Unpack
    Vpkuhus { vd: u8, va: u8, vb: u8 },
    Vpkuwus { vd: u8, va: u8, vb: u8 },
    Vpkshss { vd: u8, va: u8, vb: u8 },
    Vpkshus { vd: u8, va: u8, vb: u8 },
    Vpkswss { vd: u8, va: u8, vb: u8 },
    Vpkswus { vd: u8, va: u8, vb: u8 },
    Vpkpx { vd: u8, va: u8, vb: u8 },
    Vupkhsb { vd: u8, vb: u8 },
    Vupklsb { vd: u8, vb: u8 },
    Vupkhsh { vd: u8, vb: u8 },
    Vupklsh { vd: u8, vb: u8 },
    Vupkhpx { vd: u8, vb: u8 },
    Vupklpx { vd: u8, vb: u8 },
    
    // AltiVec Memory
    Lvx { vd: u8, ra: u8, rb: u8 },
    Stvx { vs: u8, ra: u8, rb: u8 },
    Lvxl { vd: u8, ra: u8, rb: u8 },
    Stvxl { vs: u8, ra: u8, rb: u8 },
    Lvebx { vd: u8, ra: u8, rb: u8 },
    Lvehx { vd: u8, ra: u8, rb: u8 },
    Lvewx { vd: u8, ra: u8, rb: u8 },
    Stvebx { vs: u8, ra: u8, rb: u8 },
    Stvehx { vs: u8, ra: u8, rb: u8 },
    Stvewx { vs: u8, ra: u8, rb: u8 },
    Lvsl { vd: u8, ra: u8, rb: u8 },
    Lvsr { vd: u8, ra: u8, rb: u8 },
    
    // Special
    Nop,
    Unknown { opcode: u32 },
}

/// Decode a PowerPC instruction
pub fn decode_instruction(instr: u32) -> Result<Instruction> {
    let opcode = (instr >> 26) & 0x3F;
    
    match opcode {
        // AltiVec instructions (opcode 4)
        4 => decode_altivec(instr),
        
        // Integer operations with immediate
        7 => decode_mulli(instr),
        8 => decode_subfic(instr),
        10 => decode_cmpli(instr),
        11 => decode_cmpi(instr),
        12 => decode_addic(instr),
        13 => decode_addic_dot(instr),
        14 => decode_addi(instr),
        15 => decode_addis(instr),
        
        // Branch instructions
        16 => decode_bc(instr),
        17 => decode_sc(instr),
        18 => decode_b(instr),
        
        // Extended opcodes (opcode 19)
        19 => decode_extended_19(instr),
        
        // Rotate and mask
        20 => decode_rlwimi(instr),
        21 => decode_rlwinm(instr),
        23 => decode_rlwnm(instr),
        
        // Logical immediate
        24 => decode_ori(instr),
        25 => decode_oris(instr),
        26 => decode_xori(instr),
        27 => decode_xoris(instr),
        28 => decode_andi(instr),
        29 => decode_andis(instr),
        
        // Extended opcodes (opcode 31)
        31 => decode_extended_31(instr),
        
        // Load/Store
        32 => decode_lwz(instr),
        33 => decode_lwzu(instr),
        34 => decode_lbz(instr),
        35 => decode_lbzu(instr),
        36 => decode_stw(instr),
        37 => decode_stwu(instr),
        38 => decode_stb(instr),
        39 => decode_stbu(instr),
        40 => decode_lhz(instr),
        41 => decode_lhzu(instr),
        42 => decode_lha(instr),
        43 => decode_lhau(instr),
        44 => decode_sth(instr),
        45 => decode_sthu(instr),
        46 => decode_lmw(instr),
        47 => decode_stmw(instr),
        
        // Floating point load/store
        48 => decode_lfs(instr),
        49 => decode_lfsu(instr),
        50 => decode_lfd(instr),
        51 => decode_lfdu(instr),
        52 => decode_stfs(instr),
        53 => decode_stfsu(instr),
        54 => decode_stfd(instr),
        55 => decode_stfdu(instr),
        
        // Floating point (opcode 59 - single precision)
        59 => decode_extended_59(instr),
        
        // Floating point (opcode 63 - double precision)
        63 => decode_extended_63(instr),
        
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
fn field_rc(instr: u32) -> u8 { ((instr >> 6) & 0x1F) as u8 }
#[inline]
fn field_simm(instr: u32) -> i16 { (instr & 0xFFFF) as i16 }
#[inline]
fn field_uimm(instr: u32) -> u16 { (instr & 0xFFFF) as u16 }
#[inline]
fn field_rc_bit(instr: u32) -> bool { (instr & 1) != 0 }
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
#[inline]
fn field_sh(instr: u32) -> u8 { ((instr >> 11) & 0x1F) as u8 }
#[inline]
fn field_mb(instr: u32) -> u8 { ((instr >> 6) & 0x1F) as u8 }
#[inline]
fn field_me(instr: u32) -> u8 { ((instr >> 1) & 0x1F) as u8 }
#[inline]
fn field_crfd(instr: u32) -> u8 { ((instr >> 23) & 0x7) as u8 }
#[inline]
fn field_crfs(instr: u32) -> u8 { ((instr >> 18) & 0x7) as u8 }
#[inline]
fn field_bt(instr: u32) -> u8 { ((instr >> 21) & 0x1F) as u8 }
#[inline]
fn field_ba(instr: u32) -> u8 { ((instr >> 16) & 0x1F) as u8 }
#[inline]
fn field_bb(instr: u32) -> u8 { ((instr >> 11) & 0x1F) as u8 }
#[inline]
fn field_to(instr: u32) -> u8 { ((instr >> 21) & 0x1F) as u8 }
#[inline]
fn field_l(instr: u32) -> bool { ((instr >> 21) & 1) != 0 }
#[inline]
fn field_nb(instr: u32) -> u8 { ((instr >> 11) & 0x1F) as u8 }
#[inline]
fn field_fxm(instr: u32) -> u8 { ((instr >> 12) & 0xFF) as u8 }

// SPR field needs special decoding (split across two 5-bit fields)
#[inline]
fn field_spr(instr: u32) -> u16 {
    let spr_low = ((instr >> 16) & 0x1F) as u16;
    let spr_high = ((instr >> 11) & 0x1F) as u16;
    (spr_high << 5) | spr_low
}

// Floating point register fields
#[inline]
fn field_frt(instr: u32) -> u8 { field_rt(instr) }
#[inline]
fn field_frs(instr: u32) -> u8 { field_rs(instr) }
#[inline]
fn field_fra(instr: u32) -> u8 { field_ra(instr) }
#[inline]
fn field_frb(instr: u32) -> u8 { field_rb(instr) }
#[inline]
fn field_frc(instr: u32) -> u8 { field_rc(instr) }

// Integer arithmetic immediates
fn decode_mulli(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Mulli {
        rt: field_rt(instr),
        ra: field_ra(instr),
        simm: field_simm(instr),
    })
}

fn decode_subfic(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Subfic {
        rt: field_rt(instr),
        ra: field_ra(instr),
        simm: field_simm(instr),
    })
}

fn decode_cmpli(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Cmpli {
        crfd: field_crfd(instr),
        l: field_l(instr),
        ra: field_ra(instr),
        uimm: field_uimm(instr),
    })
}

fn decode_cmpi(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Cmpi {
        crfd: field_crfd(instr),
        l: field_l(instr),
        ra: field_ra(instr),
        simm: field_simm(instr),
    })
}

fn decode_addic(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Addic {
        rt: field_rt(instr),
        ra: field_ra(instr),
        simm: field_simm(instr),
    })
}

fn decode_addic_dot(instr: u32) -> Result<Instruction> {
    Ok(Instruction::AddicDot {
        rt: field_rt(instr),
        ra: field_ra(instr),
        simm: field_simm(instr),
    })
}

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

// Branch instructions
fn decode_b(instr: u32) -> Result<Instruction> {
    let li = ((instr & 0x03FFFFFC) as i32) << 6 >> 6; // Sign extend 26-bit
    Ok(Instruction::B {
        li,
        aa: field_aa(instr),
        lk: field_lk(instr),
    })
}

fn decode_bc(instr: u32) -> Result<Instruction> {
    let bd = (((instr & 0xFFFC) as i16) << 2) >> 2; // Sign extend 16-bit
    Ok(Instruction::Bc {
        bo: field_bo(instr),
        bi: field_bi(instr),
        bd,
        aa: field_aa(instr),
        lk: field_lk(instr),
    })
}

fn decode_sc(_instr: u32) -> Result<Instruction> {
    Ok(Instruction::Sc)
}

// Rotate and mask
fn decode_rlwimi(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Rlwimi {
        ra: field_ra(instr),
        rs: field_rs(instr),
        sh: field_sh(instr),
        mb: field_mb(instr),
        me: field_me(instr),
        rc: field_rc_bit(instr),
    })
}

fn decode_rlwinm(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Rlwinm {
        ra: field_ra(instr),
        rs: field_rs(instr),
        sh: field_sh(instr),
        mb: field_mb(instr),
        me: field_me(instr),
        rc: field_rc_bit(instr),
    })
}

fn decode_rlwnm(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Rlwnm {
        ra: field_ra(instr),
        rs: field_rs(instr),
        rb: field_rb(instr),
        mb: field_mb(instr),
        me: field_me(instr),
        rc: field_rc_bit(instr),
    })
}

// Logical immediate operations
fn decode_ori(instr: u32) -> Result<Instruction> {
    // Special case: ori 0,0,0 is nop
    if instr == 0x60000000 {
        return Ok(Instruction::Nop);
    }
    Ok(Instruction::Ori {
        ra: field_ra(instr),
        rs: field_rs(instr),
        uimm: field_uimm(instr),
    })
}

fn decode_oris(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Oris {
        ra: field_ra(instr),
        rs: field_rs(instr),
        uimm: field_uimm(instr),
    })
}

fn decode_xori(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Xori {
        ra: field_ra(instr),
        rs: field_rs(instr),
        uimm: field_uimm(instr),
    })
}

fn decode_xoris(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Xoris {
        ra: field_ra(instr),
        rs: field_rs(instr),
        uimm: field_uimm(instr),
    })
}

fn decode_andi(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Andi {
        ra: field_ra(instr),
        rs: field_rs(instr),
        uimm: field_uimm(instr),
    })
}

fn decode_andis(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Andis {
        ra: field_ra(instr),
        rs: field_rs(instr),
        uimm: field_uimm(instr),
    })
}

// Load/Store instructions
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

fn decode_lbz(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Lbz {
        rt: field_rt(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_lbzu(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Lbzu {
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

fn decode_stb(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Stb {
        rs: field_rs(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_stbu(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Stbu {
        rs: field_rs(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_lhz(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Lhz {
        rt: field_rt(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_lhzu(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Lhzu {
        rt: field_rt(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_lha(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Lha {
        rt: field_rt(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_lhau(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Lhau {
        rt: field_rt(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_sth(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Sth {
        rs: field_rs(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_sthu(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Sthu {
        rs: field_rs(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_lmw(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Lmw {
        rt: field_rt(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_stmw(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Stmw {
        rs: field_rs(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

// Floating point load/store
fn decode_lfs(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Lfs {
        frt: field_frt(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_lfsu(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Lfsu {
        frt: field_frt(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_lfd(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Lfd {
        frt: field_frt(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_lfdu(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Lfdu {
        frt: field_frt(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_stfs(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Stfs {
        frs: field_frs(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_stfsu(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Stfsu {
        frs: field_frs(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_stfd(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Stfd {
        frs: field_frs(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

fn decode_stfdu(instr: u32) -> Result<Instruction> {
    Ok(Instruction::Stfdu {
        frs: field_frs(instr),
        ra: field_ra(instr),
        d: field_simm(instr),
    })
}

// Extended opcode 19 (CR ops and branches)
fn decode_extended_19(instr: u32) -> Result<Instruction> {
    let xo = (instr >> 1) & 0x3FF;
    
    match xo {
        0 => Ok(Instruction::Mcrf {
            crfd: field_crfd(instr),
            crfs: field_crfs(instr),
        }),
        16 => Ok(Instruction::Bclr {
            bo: field_bo(instr),
            bi: field_bi(instr),
            lk: field_lk(instr),
        }),
        33 => Ok(Instruction::Crnor {
            bt: field_bt(instr),
            ba: field_ba(instr),
            bb: field_bb(instr),
        }),
        50 => Ok(Instruction::Rfi),
        129 => Ok(Instruction::Crandc {
            bt: field_bt(instr),
            ba: field_ba(instr),
            bb: field_bb(instr),
        }),
        150 => Ok(Instruction::Isync),
        193 => Ok(Instruction::Crxor {
            bt: field_bt(instr),
            ba: field_ba(instr),
            bb: field_bb(instr),
        }),
        225 => Ok(Instruction::Crnand {
            bt: field_bt(instr),
            ba: field_ba(instr),
            bb: field_bb(instr),
        }),
        257 => Ok(Instruction::Crand {
            bt: field_bt(instr),
            ba: field_ba(instr),
            bb: field_bb(instr),
        }),
        289 => Ok(Instruction::Creqv {
            bt: field_bt(instr),
            ba: field_ba(instr),
            bb: field_bb(instr),
        }),
        417 => Ok(Instruction::Crorc {
            bt: field_bt(instr),
            ba: field_ba(instr),
            bb: field_bb(instr),
        }),
        449 => Ok(Instruction::Cror {
            bt: field_bt(instr),
            ba: field_ba(instr),
            bb: field_bb(instr),
        }),
        528 => Ok(Instruction::Bcctr {
            bo: field_bo(instr),
            bi: field_bi(instr),
            lk: field_lk(instr),
        }),
        _ => Ok(Instruction::Unknown { opcode: instr }),
    }
}

// Extended opcode 31 (ALU and misc)
fn decode_extended_31(instr: u32) -> Result<Instruction> {
    let xo = (instr >> 1) & 0x3FF;
    let rc = field_rc_bit(instr);
    let oe = field_oe(instr);
    
    match xo {
        0 => Ok(Instruction::Cmp {
            crfd: field_crfd(instr),
            l: field_l(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        4 => Ok(Instruction::Tw {
            to: field_to(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        8 => Ok(Instruction::Subfc {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
            oe,
            rc,
        }),
        10 => Ok(Instruction::Addc {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
            oe,
            rc,
        }),
        11 => Ok(Instruction::Mulhwu {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
            rc,
        }),
        19 => Ok(Instruction::Mfcr {
            rt: field_rt(instr),
        }),
        20 => Ok(Instruction::Lwarx {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        23 => Ok(Instruction::Lwzx {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        24 => Ok(Instruction::Slw {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rb: field_rb(instr),
            rc,
        }),
        26 => Ok(Instruction::Cntlzw {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rc,
        }),
        28 => Ok(Instruction::And {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rb: field_rb(instr),
            rc,
        }),
        32 => Ok(Instruction::Cmpl {
            crfd: field_crfd(instr),
            l: field_l(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        40 => Ok(Instruction::Subf {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
            oe,
            rc,
        }),
        54 => Ok(Instruction::Dcbst {
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        55 => Ok(Instruction::Lwzux {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        60 => Ok(Instruction::Andc {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rb: field_rb(instr),
            rc,
        }),
        75 => Ok(Instruction::Mulhw {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
            rc,
        }),
        83 => Ok(Instruction::Mfmsr {
            rt: field_rt(instr),
        }),
        86 => Ok(Instruction::Dcbf {
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        87 => Ok(Instruction::Lbzx {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        104 => Ok(Instruction::Neg {
            rt: field_rt(instr),
            ra: field_ra(instr),
            oe,
            rc,
        }),
        119 => Ok(Instruction::Lbzux {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        124 => Ok(Instruction::Nor {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rb: field_rb(instr),
            rc,
        }),
        136 => Ok(Instruction::Subfe {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
            oe,
            rc,
        }),
        138 => Ok(Instruction::Adde {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
            oe,
            rc,
        }),
        144 => Ok(Instruction::Mtcrf {
            fxm: field_fxm(instr),
            rs: field_rs(instr),
        }),
        146 => Ok(Instruction::Mtmsr {
            rs: field_rs(instr),
        }),
        150 => Ok(Instruction::Stwcx {
            rs: field_rs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        151 => Ok(Instruction::Stwx {
            rs: field_rs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        183 => Ok(Instruction::Stwux {
            rs: field_rs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        200 => Ok(Instruction::Subfze {
            rt: field_rt(instr),
            ra: field_ra(instr),
            oe,
            rc,
        }),
        202 => Ok(Instruction::Addze {
            rt: field_rt(instr),
            ra: field_ra(instr),
            oe,
            rc,
        }),
        210 => Ok(Instruction::Mtsr {
            sr: field_ra(instr),
            rs: field_rs(instr),
        }),
        215 => Ok(Instruction::Stbx {
            rs: field_rs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        232 => Ok(Instruction::Subfme {
            rt: field_rt(instr),
            ra: field_ra(instr),
            oe,
            rc,
        }),
        234 => Ok(Instruction::Addme {
            rt: field_rt(instr),
            ra: field_ra(instr),
            oe,
            rc,
        }),
        235 => Ok(Instruction::Mullw {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
            oe,
            rc,
        }),
        242 => Ok(Instruction::Mtsrin {
            rs: field_rs(instr),
            rb: field_rb(instr),
        }),
        246 => Ok(Instruction::Dcbtst {
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        247 => Ok(Instruction::Stbux {
            rs: field_rs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        266 => Ok(Instruction::Add {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
            oe,
            rc,
        }),
        278 => Ok(Instruction::Dcbt {
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        279 => Ok(Instruction::Lhzx {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        284 => Ok(Instruction::Eqv {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rb: field_rb(instr),
            rc,
        }),
        306 => Ok(Instruction::Tlbie {
            rb: field_rb(instr),
        }),
        310 => Ok(Instruction::Eciwx {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        311 => Ok(Instruction::Lhzux {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        316 => Ok(Instruction::Xor {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rb: field_rb(instr),
            rc,
        }),
        339 => Ok(Instruction::Mfspr {
            rt: field_rt(instr),
            spr: field_spr(instr),
        }),
        343 => Ok(Instruction::Lhax {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        370 => Ok(Instruction::Tlbia),
        371 => Ok(Instruction::Mftb {
            rt: field_rt(instr),
            tbr: field_spr(instr),
        }),
        375 => Ok(Instruction::Lhaux {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        407 => Ok(Instruction::Sthx {
            rs: field_rs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        412 => Ok(Instruction::Orc {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rb: field_rb(instr),
            rc,
        }),
        438 => Ok(Instruction::Ecowx {
            rs: field_rs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        439 => Ok(Instruction::Sthux {
            rs: field_rs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        444 => Ok(Instruction::Or {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rb: field_rb(instr),
            rc,
        }),
        459 => Ok(Instruction::Divwu {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
            oe,
            rc,
        }),
        467 => Ok(Instruction::Mtspr {
            spr: field_spr(instr),
            rs: field_rs(instr),
        }),
        470 => Ok(Instruction::Dcbi {
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        476 => Ok(Instruction::Nand {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rb: field_rb(instr),
            rc,
        }),
        491 => Ok(Instruction::Divw {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
            oe,
            rc,
        }),
        
        // AltiVec memory operations
        7 => Ok(Instruction::Lvebx {
            vd: field_vd(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        39 => Ok(Instruction::Lvehx {
            vd: field_vd(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        71 => Ok(Instruction::Lvewx {
            vd: field_vd(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        103 => Ok(Instruction::Lvx {
            vd: field_vd(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        359 => Ok(Instruction::Lvxl {
            vd: field_vd(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        6 => Ok(Instruction::Lvsl {
            vd: field_vd(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        38 => Ok(Instruction::Lvsr {
            vd: field_vd(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        135 => Ok(Instruction::Stvebx {
            vs: field_vs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        167 => Ok(Instruction::Stvehx {
            vs: field_vs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        199 => Ok(Instruction::Stvewx {
            vs: field_vs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        231 => Ok(Instruction::Stvx {
            vs: field_vs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        487 => Ok(Instruction::Stvxl {
            vs: field_vs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        
        512 => Ok(Instruction::Mcrxr {
            crfd: field_crfd(instr),
        }),
        533 => Ok(Instruction::Lswx {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        534 => Ok(Instruction::Lwbrx {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        536 => Ok(Instruction::Srw {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rb: field_rb(instr),
            rc,
        }),
        566 => Ok(Instruction::Tlbsync),
        595 => Ok(Instruction::Mfsr {
            rt: field_rt(instr),
            sr: field_ra(instr),
        }),
        597 => Ok(Instruction::Lswi {
            rt: field_rt(instr),
            ra: field_ra(instr),
            nb: field_nb(instr),
        }),
        598 => Ok(Instruction::Sync),
        659 => Ok(Instruction::Mfsrin {
            rt: field_rt(instr),
            rb: field_rb(instr),
        }),
        661 => Ok(Instruction::Stswx {
            rs: field_rs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        662 => Ok(Instruction::Stwbrx {
            rs: field_rs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        725 => Ok(Instruction::Stswi {
            rs: field_rs(instr),
            ra: field_ra(instr),
            nb: field_nb(instr),
        }),
        790 => Ok(Instruction::Lhbrx {
            rt: field_rt(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        792 => Ok(Instruction::Sraw {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rb: field_rb(instr),
            rc,
        }),
        824 => Ok(Instruction::Srawi {
            ra: field_ra(instr),
            rs: field_rs(instr),
            sh: field_sh(instr),
            rc,
        }),
        854 => Ok(Instruction::Eieio),
        918 => Ok(Instruction::Sthbrx {
            rs: field_rs(instr),
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        922 => Ok(Instruction::Extsh {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rc,
        }),
        954 => Ok(Instruction::Extsb {
            ra: field_ra(instr),
            rs: field_rs(instr),
            rc,
        }),
        982 => Ok(Instruction::Icbi {
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        1014 => Ok(Instruction::Dcbz {
            ra: field_ra(instr),
            rb: field_rb(instr),
        }),
        _ => Ok(Instruction::Unknown { opcode: instr }),
    }
}

// Floating point single precision (opcode 59)
fn decode_extended_59(instr: u32) -> Result<Instruction> {
    let xo = (instr >> 1) & 0x1F;
    let rc = field_rc_bit(instr);
    
    match xo {
        18 => Ok(Instruction::Fdivs {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frb: field_frb(instr),
            rc,
        }),
        20 => Ok(Instruction::Fsubs {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frb: field_frb(instr),
            rc,
        }),
        21 => Ok(Instruction::Fadds {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frb: field_frb(instr),
            rc,
        }),
        22 => Ok(Instruction::Fsqrts {
            frt: field_frt(instr),
            frb: field_frb(instr),
            rc,
        }),
        25 => Ok(Instruction::Fmuls {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frc: field_frc(instr),
            rc,
        }),
        28 => Ok(Instruction::Fmsubs {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frc: field_frc(instr),
            frb: field_frb(instr),
            rc,
        }),
        29 => Ok(Instruction::Fmadds {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frc: field_frc(instr),
            frb: field_frb(instr),
            rc,
        }),
        30 => Ok(Instruction::Fnmsubs {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frc: field_frc(instr),
            frb: field_frb(instr),
            rc,
        }),
        31 => Ok(Instruction::Fnmadds {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frc: field_frc(instr),
            frb: field_frb(instr),
            rc,
        }),
        _ => Ok(Instruction::Unknown { opcode: instr }),
    }
}

// Floating point double precision (opcode 63)
fn decode_extended_63(instr: u32) -> Result<Instruction> {
    let xo = (instr >> 1) & 0x3FF;
    let rc = field_rc_bit(instr);
    
    match xo {
        0 => Ok(Instruction::Fcmpu {
            crfd: field_crfd(instr),
            fra: field_fra(instr),
            frb: field_frb(instr),
        }),
        12 => Ok(Instruction::Frsp {
            frt: field_frt(instr),
            frb: field_frb(instr),
            rc,
        }),
        14 => Ok(Instruction::Fctiwz {
            frt: field_frt(instr),
            frb: field_frb(instr),
            rc,
        }),
        18 => Ok(Instruction::Fdiv {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frb: field_frb(instr),
            rc,
        }),
        20 => Ok(Instruction::Fsub {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frb: field_frb(instr),
            rc,
        }),
        21 => Ok(Instruction::Fadd {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frb: field_frb(instr),
            rc,
        }),
        22 => Ok(Instruction::Fsqrt {
            frt: field_frt(instr),
            frb: field_frb(instr),
            rc,
        }),
        25 => Ok(Instruction::Fmul {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frc: field_frc(instr),
            rc,
        }),
        28 => Ok(Instruction::Fmsub {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frc: field_frc(instr),
            frb: field_frb(instr),
            rc,
        }),
        29 => Ok(Instruction::Fmadd {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frc: field_frc(instr),
            frb: field_frb(instr),
            rc,
        }),
        30 => Ok(Instruction::Fnmsub {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frc: field_frc(instr),
            frb: field_frb(instr),
            rc,
        }),
        31 => Ok(Instruction::Fnmadd {
            frt: field_frt(instr),
            fra: field_fra(instr),
            frc: field_frc(instr),
            frb: field_frb(instr),
            rc,
        }),
        32 => Ok(Instruction::Fcmpo {
            crfd: field_crfd(instr),
            fra: field_fra(instr),
            frb: field_frb(instr),
        }),
        40 => Ok(Instruction::Fneg {
            frt: field_frt(instr),
            frb: field_frb(instr),
            rc,
        }),
        72 => Ok(Instruction::Fmr {
            frt: field_frt(instr),
            frb: field_frb(instr),
            rc,
        }),
        264 => Ok(Instruction::Fabs {
            frt: field_frt(instr),
            frb: field_frb(instr),
            rc,
        }),
        _ => Ok(Instruction::Unknown { opcode: instr }),
    }
}


// AltiVec field extractors
#[inline]
fn field_vd(instr: u32) -> u8 {
    ((instr >> 21) & 0x1F) as u8
}

#[inline]
fn field_va(instr: u32) -> u8 {
    ((instr >> 16) & 0x1F) as u8
}

#[inline]
fn field_vb(instr: u32) -> u8 {
    ((instr >> 11) & 0x1F) as u8
}

#[inline]
fn field_vc(instr: u32) -> u8 {
    ((instr >> 6) & 0x1F) as u8
}

#[inline]
fn field_vs(instr: u32) -> u8 {
    field_vd(instr)
}

#[inline]
fn field_vuimm(instr: u32) -> u8 {
    ((instr >> 16) & 0x1F) as u8
}

#[inline]
fn field_vsimm(instr: u32) -> i8 {
    let val = (instr >> 16) & 0x1F;
    // Sign-extend 5-bit value
    if (val & 0x10) != 0 {
        (val | 0xFFFF_FFE0) as i8
    } else {
        val as i8
    }
}

/// Decode AltiVec instructions (opcode 4)
fn decode_altivec(instr: u32) -> Result<Instruction> {
    let xo = instr & 0x7FF;  // 11-bit extended opcode
    let vd = field_vd(instr);
    let va = field_va(instr);
    let vb = field_vb(instr);
    let vc = field_vc(instr);
    let rc = (instr & 0x400) != 0;  // Bit 10 for record bit in some instructions
    
    match xo {
        // Arithmetic - Float
        10 => Ok(Instruction::Vaddfp { vd, va, vb }),
        74 => Ok(Instruction::Vsubfp { vd, va, vb }),
        46 => Ok(Instruction::Vmaddfp { vd, va, vb, vc }),
        47 => Ok(Instruction::Vnmsubfp { vd, va, vb, vc }),
        
        // Arithmetic - Integer add/subtract modulo
        0 => Ok(Instruction::Vaddubm { vd, va, vb }),
        64 => Ok(Instruction::Vadduhm { vd, va, vb }),
        128 => Ok(Instruction::Vadduwm { vd, va, vb }),
        1024 => Ok(Instruction::Vsububm { vd, va, vb }),
        1088 => Ok(Instruction::Vsubuhm { vd, va, vb }),
        1152 => Ok(Instruction::Vsubuwm { vd, va, vb }),
        
        // Arithmetic - Integer add/subtract saturate
        512 => Ok(Instruction::Vaddubs { vd, va, vb }),
        576 => Ok(Instruction::Vadduhs { vd, va, vb }),
        640 => Ok(Instruction::Vadduws { vd, va, vb }),
        768 => Ok(Instruction::Vaddsbs { vd, va, vb }),
        832 => Ok(Instruction::Vaddshs { vd, va, vb }),
        896 => Ok(Instruction::Vaddsws { vd, va, vb }),
        
        // Arithmetic - Min/Max
        258 => Ok(Instruction::Vmaxsb { vd, va, vb }),
        322 => Ok(Instruction::Vmaxsh { vd, va, vb }),
        386 => Ok(Instruction::Vmaxsw { vd, va, vb }),
        2 => Ok(Instruction::Vmaxub { vd, va, vb }),
        66 => Ok(Instruction::Vmaxuh { vd, va, vb }),
        130 => Ok(Instruction::Vmaxuw { vd, va, vb }),
        1034 => Ok(Instruction::Vmaxfp { vd, va, vb }),
        770 => Ok(Instruction::Vminsb { vd, va, vb }),
        834 => Ok(Instruction::Vminsh { vd, va, vb }),
        898 => Ok(Instruction::Vminsw { vd, va, vb }),
        514 => Ok(Instruction::Vminub { vd, va, vb }),
        578 => Ok(Instruction::Vminuh { vd, va, vb }),
        642 => Ok(Instruction::Vminuw { vd, va, vb }),
        1098 => Ok(Instruction::Vminfp { vd, va, vb }),
        
        // Arithmetic - Average
        1026 => Ok(Instruction::Vavgub { vd, va, vb }),
        1090 => Ok(Instruction::Vavguh { vd, va, vb }),
        1154 => Ok(Instruction::Vavguw { vd, va, vb }),
        1282 => Ok(Instruction::Vavgsb { vd, va, vb }),
        1346 => Ok(Instruction::Vavgsh { vd, va, vb }),
        1410 => Ok(Instruction::Vavgsw { vd, va, vb }),
        
        // Arithmetic - Multiply
        520 => Ok(Instruction::Vmuleub { vd, va, vb }),
        8 => Ok(Instruction::Vmuloub { vd, va, vb }),
        776 => Ok(Instruction::Vmulesb { vd, va, vb }),
        264 => Ok(Instruction::Vmulosb { vd, va, vb }),
        584 => Ok(Instruction::Vmuleuh { vd, va, vb }),
        72 => Ok(Instruction::Vmulouh { vd, va, vb }),
        840 => Ok(Instruction::Vmulesh { vd, va, vb }),
        328 => Ok(Instruction::Vmulosh { vd, va, vb }),
        
        // Logical
        1028 => Ok(Instruction::Vand { vd, va, vb }),
        1092 => Ok(Instruction::Vandc { vd, va, vb }),
        1156 => Ok(Instruction::Vor { vd, va, vb }),
        1284 => Ok(Instruction::Vnor { vd, va, vb }),
        1220 => Ok(Instruction::Vxor { vd, va, vb }),
        
        // Comparison (with Rc variants)
        6 if !rc => Ok(Instruction::Vcmpequb { vd, va, vb, rc: false }),
        6 if rc => Ok(Instruction::Vcmpequb { vd, va, vb, rc: true }),
        70 if !rc => Ok(Instruction::Vcmpequh { vd, va, vb, rc: false }),
        70 if rc => Ok(Instruction::Vcmpequh { vd, va, vb, rc: true }),
        134 if !rc => Ok(Instruction::Vcmpequw { vd, va, vb, rc: false }),
        134 if rc => Ok(Instruction::Vcmpequw { vd, va, vb, rc: true }),
        198 if !rc => Ok(Instruction::Vcmpeqfp { vd, va, vb, rc: false }),
        198 if rc => Ok(Instruction::Vcmpeqfp { vd, va, vb, rc: true }),
        774 if !rc => Ok(Instruction::Vcmpgtsb { vd, va, vb, rc: false }),
        774 if rc => Ok(Instruction::Vcmpgtsb { vd, va, vb, rc: true }),
        838 if !rc => Ok(Instruction::Vcmpgtsh { vd, va, vb, rc: false }),
        838 if rc => Ok(Instruction::Vcmpgtsh { vd, va, vb, rc: true }),
        902 if !rc => Ok(Instruction::Vcmpgtsw { vd, va, vb, rc: false }),
        902 if rc => Ok(Instruction::Vcmpgtsw { vd, va, vb, rc: true }),
        518 if !rc => Ok(Instruction::Vcmpgtub { vd, va, vb, rc: false }),
        518 if rc => Ok(Instruction::Vcmpgtub { vd, va, vb, rc: true }),
        582 if !rc => Ok(Instruction::Vcmpgtuh { vd, va, vb, rc: false }),
        582 if rc => Ok(Instruction::Vcmpgtuh { vd, va, vb, rc: true }),
        646 if !rc => Ok(Instruction::Vcmpgtuw { vd, va, vb, rc: false }),
        646 if rc => Ok(Instruction::Vcmpgtuw { vd, va, vb, rc: true }),
        710 if !rc => Ok(Instruction::Vcmpgtfp { vd, va, vb, rc: false }),
        710 if rc => Ok(Instruction::Vcmpgtfp { vd, va, vb, rc: true }),
        454 if !rc => Ok(Instruction::Vcmpgefp { vd, va, vb, rc: false }),
        454 if rc => Ok(Instruction::Vcmpgefp { vd, va, vb, rc: true }),
        
        // Shift/Rotate
        260 => Ok(Instruction::Vslb { vd, va, vb }),
        324 => Ok(Instruction::Vslh { vd, va, vb }),
        388 => Ok(Instruction::Vslw { vd, va, vb }),
        516 => Ok(Instruction::Vsrb { vd, va, vb }),
        580 => Ok(Instruction::Vsrh { vd, va, vb }),
        644 => Ok(Instruction::Vsrw { vd, va, vb }),
        772 => Ok(Instruction::Vsrab { vd, va, vb }),
        836 => Ok(Instruction::Vsrah { vd, va, vb }),
        900 => Ok(Instruction::Vsraw { vd, va, vb }),
        4 => Ok(Instruction::Vrlb { vd, va, vb }),
        68 => Ok(Instruction::Vrlh { vd, va, vb }),
        132 => Ok(Instruction::Vrlw { vd, va, vb }),
        452 => Ok(Instruction::Vsl { vd, va, vb }),
        708 => Ok(Instruction::Vsr { vd, va, vb }),
        
        // Permute/Merge
        43 => Ok(Instruction::Vperm { vd, va, vb, vc }),
        42 => Ok(Instruction::Vsel { vd, va, vb, vc }),
        524 => Ok(Instruction::Vspltb { vd, vb, uimm: field_vuimm(instr) }),
        588 => Ok(Instruction::Vsplth { vd, vb, uimm: field_vuimm(instr) }),
        652 => Ok(Instruction::Vspltw { vd, vb, uimm: field_vuimm(instr) }),
        780 => Ok(Instruction::Vspltisb { vd, simm: field_vsimm(instr) }),
        844 => Ok(Instruction::Vspltish { vd, simm: field_vsimm(instr) }),
        908 => Ok(Instruction::Vspltisw { vd, simm: field_vsimm(instr) }),
        12 => Ok(Instruction::Vmrghb { vd, va, vb }),
        76 => Ok(Instruction::Vmrghh { vd, va, vb }),
        140 => Ok(Instruction::Vmrghw { vd, va, vb }),
        268 => Ok(Instruction::Vmrglb { vd, va, vb }),
        332 => Ok(Instruction::Vmrglh { vd, va, vb }),
        396 => Ok(Instruction::Vmrglw { vd, va, vb }),
        
        // Pack/Unpack
        14 => Ok(Instruction::Vpkuhus { vd, va, vb }),
        78 => Ok(Instruction::Vpkuwus { vd, va, vb }),
        270 => Ok(Instruction::Vpkshss { vd, va, vb }),
        142 => Ok(Instruction::Vpkshus { vd, va, vb }),
        334 => Ok(Instruction::Vpkswss { vd, va, vb }),
        206 => Ok(Instruction::Vpkswus { vd, va, vb }),
        782 => Ok(Instruction::Vpkpx { vd, va, vb }),
        526 => Ok(Instruction::Vupkhsb { vd, vb }),
        590 => Ok(Instruction::Vupklsb { vd, vb }),
        654 => Ok(Instruction::Vupkhsh { vd, vb }),
        718 => Ok(Instruction::Vupklsh { vd, vb }),
        846 => Ok(Instruction::Vupkhpx { vd, vb }),
        974 => Ok(Instruction::Vupklpx { vd, vb }),
        
        _ => Ok(Instruction::Unknown { opcode: instr }),
    }
}
