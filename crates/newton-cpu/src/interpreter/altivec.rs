// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! AltiVec instruction interpreter

use crate::registers::Registers;
use crate::altivec::ops::*;
use crate::altivec::types::Vector128;
use crate::MemoryInterface;
use newton_utils::Result;

/// Convert vector register to Vector128
#[inline]
fn get_vector(regs: &Registers, vr: u8) -> Vector128 {
    let idx = vr as usize;
    Vector128::from_words(regs.vr[idx])
}

/// Set vector register from Vector128
#[inline]
fn set_vector(regs: &mut Registers, vr: u8, v: Vector128) {
    let idx = vr as usize;
    regs.vr[idx] = v.as_words();
}

// Arithmetic operations
pub fn vaddfp(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vaddfp(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vsubfp(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vsubfp(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vmaddfp(regs: &mut Registers, vd: u8, va: u8, vb: u8, vc: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let c = get_vector(regs, vc);
    let result = arithmetic::vmaddfp(a, b, c);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vnmsubfp(regs: &mut Registers, vd: u8, va: u8, vb: u8, vc: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let c = get_vector(regs, vc);
    let result = arithmetic::vnmsubfp(a, b, c);
    set_vector(regs, vd, result);
    Ok(())
}

// Integer arithmetic - modulo
pub fn vaddubm(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vaddubm(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vadduhm(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vadduhm(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vadduwm(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vadduwm(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vsububm(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vsububm(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vsubuhm(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vsubuhm(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vsubuwm(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vsubuwm(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

// Integer arithmetic - saturate
pub fn vaddubs(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let (result, _sat) = arithmetic::vaddubs(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vadduhs(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let (result, _sat) = arithmetic::vadduhs(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vadduws(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let (result, _sat) = arithmetic::vadduws(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vaddsbs(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let (result, _sat) = arithmetic::vaddsbs(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vaddshs(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let (result, _sat) = arithmetic::vaddshs(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vaddsws(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let (result, _sat) = arithmetic::vaddsws(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

// Min/Max operations
pub fn vmaxsb(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vmaxsb(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vmaxsh(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vmaxsh(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vmaxsw(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vmaxsw(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vmaxub(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vmaxub(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vmaxuh(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vmaxuh(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vmaxuw(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vmaxuw(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vmaxfp(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vmaxfp(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vminsb(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vminsb(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vminsh(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vminsh(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vminsw(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vminsw(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vminub(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vminub(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vminuh(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vminuh(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vminuw(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vminuw(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vminfp(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = arithmetic::vminfp(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

// Logical operations
pub fn vand(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = logical::vand(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vandc(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = logical::vandc(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vor(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = logical::vor(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vnor(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = logical::vnor(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vxor(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = logical::vxor(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

// Comparison operations
pub fn vcmpequb(regs: &mut Registers, vd: u8, va: u8, vb: u8, _rc: bool) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = compare::vcmpequb(a, b);
    set_vector(regs, vd, result);
    // TODO: Handle CR6 update when rc=true
    Ok(())
}

pub fn vcmpequh(regs: &mut Registers, vd: u8, va: u8, vb: u8, _rc: bool) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = compare::vcmpequh(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vcmpequw(regs: &mut Registers, vd: u8, va: u8, vb: u8, _rc: bool) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = compare::vcmpequw(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vcmpeqfp(regs: &mut Registers, vd: u8, va: u8, vb: u8, _rc: bool) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = compare::vcmpeqfp(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vcmpgtsb(regs: &mut Registers, vd: u8, va: u8, vb: u8, _rc: bool) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = compare::vcmpgtsb(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vcmpgtsh(regs: &mut Registers, vd: u8, va: u8, vb: u8, _rc: bool) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = compare::vcmpgtsh(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vcmpgtsw(regs: &mut Registers, vd: u8, va: u8, vb: u8, _rc: bool) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = compare::vcmpgtsw(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vcmpgtub(regs: &mut Registers, vd: u8, va: u8, vb: u8, _rc: bool) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = compare::vcmpgtub(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vcmpgtuh(regs: &mut Registers, vd: u8, va: u8, vb: u8, _rc: bool) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = compare::vcmpgtuh(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vcmpgtuw(regs: &mut Registers, vd: u8, va: u8, vb: u8, _rc: bool) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = compare::vcmpgtuw(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vcmpgtfp(regs: &mut Registers, vd: u8, va: u8, vb: u8, _rc: bool) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = compare::vcmpgtfp(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vcmpgefp(regs: &mut Registers, vd: u8, va: u8, vb: u8, _rc: bool) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = compare::vcmpgefp(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

// Shift/Rotate operations
pub fn vslb(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = shift::vslb(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vslh(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = shift::vslh(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vslw(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = shift::vslw(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vsrb(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = shift::vsrb(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vsrh(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = shift::vsrh(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vsrw(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = shift::vsrw(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vsrab(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = shift::vsrab(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vsrah(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = shift::vsrah(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vsraw(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = shift::vsraw(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vrlb(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = shift::vrlb(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vrlh(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = shift::vrlh(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vrlw(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = shift::vrlw(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vsl(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = shift::vsl(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vsr(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = shift::vsr(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

// Permute/Merge operations
pub fn vperm(regs: &mut Registers, vd: u8, va: u8, vb: u8, vc: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let c = get_vector(regs, vc);
    let result = permute::vperm(a, b, c);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vsel(regs: &mut Registers, vd: u8, va: u8, vb: u8, vc: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let c = get_vector(regs, vc);
    let result = permute::vsel(a, b, c);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vspltb(regs: &mut Registers, vd: u8, vb: u8, uimm: u8) -> Result<()> {
    let b = get_vector(regs, vb);
    let result = permute::vspltb(b, uimm);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vsplth(regs: &mut Registers, vd: u8, vb: u8, uimm: u8) -> Result<()> {
    let b = get_vector(regs, vb);
    let result = permute::vsplth(b, uimm);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vspltw(regs: &mut Registers, vd: u8, vb: u8, uimm: u8) -> Result<()> {
    let b = get_vector(regs, vb);
    let result = permute::vspltw(b, uimm);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vspltisb(regs: &mut Registers, vd: u8, simm: i8) -> Result<()> {
    let result = permute::vspltisb(simm);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vspltish(regs: &mut Registers, vd: u8, simm: i8) -> Result<()> {
    let result = permute::vspltish(simm as i16);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vspltisw(regs: &mut Registers, vd: u8, simm: i8) -> Result<()> {
    let result = permute::vspltisw(simm as i32);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vmrghb(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = permute::vmrghb(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vmrghh(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = permute::vmrghh(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vmrghw(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = permute::vmrghw(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vmrglb(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = permute::vmrglb(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vmrglh(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = permute::vmrglh(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vmrglw(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = permute::vmrglw(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

// Pack/Unpack operations
pub fn vpkuhus(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = pack::vpkuhus(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vpkuwus(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = pack::vpkuwus(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vpkshss(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = pack::vpkshss(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vpkshus(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = pack::vpkshus(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vpkswss(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = pack::vpkswss(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vpkswus(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = pack::vpkswus(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vpkpx(regs: &mut Registers, vd: u8, va: u8, vb: u8) -> Result<()> {
    let a = get_vector(regs, va);
    let b = get_vector(regs, vb);
    let result = pack::vpkpx(a, b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vupkhsb(regs: &mut Registers, vd: u8, vb: u8) -> Result<()> {
    let b = get_vector(regs, vb);
    let result = pack::vupkhsb(b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vupklsb(regs: &mut Registers, vd: u8, vb: u8) -> Result<()> {
    let b = get_vector(regs, vb);
    let result = pack::vupklsb(b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vupkhsh(regs: &mut Registers, vd: u8, vb: u8) -> Result<()> {
    let b = get_vector(regs, vb);
    let result = pack::vupkhsh(b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vupklsh(regs: &mut Registers, vd: u8, vb: u8) -> Result<()> {
    let b = get_vector(regs, vb);
    let result = pack::vupklsh(b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vupkhpx(regs: &mut Registers, vd: u8, vb: u8) -> Result<()> {
    let b = get_vector(regs, vb);
    let result = pack::vupkhpx(b);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn vupklpx(regs: &mut Registers, vd: u8, vb: u8) -> Result<()> {
    let b = get_vector(regs, vb);
    let result = pack::vupklpx(b);
    set_vector(regs, vd, result);
    Ok(())
}

// Memory operations
pub fn lvx(regs: &mut Registers, memory: &dyn MemoryInterface, vd: u8, ra: u8, rb: u8) -> Result<()> {
    let ra_val = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let rb_val = regs.gpr[rb as usize];
    let result = memory::lvx(memory, ra_val, rb_val)?;
    set_vector(regs, vd, result);
    Ok(())
}

pub fn stvx(regs: &mut Registers, memory: &dyn MemoryInterface, vs: u8, ra: u8, rb: u8) -> Result<()> {
    let ra_val = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let rb_val = regs.gpr[rb as usize];
    let vec = get_vector(regs, vs);
    memory::stvx(memory, vec, ra_val, rb_val)?;
    Ok(())
}

pub fn lvxl(regs: &mut Registers, memory: &dyn MemoryInterface, vd: u8, ra: u8, rb: u8) -> Result<()> {
    let ra_val = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let rb_val = regs.gpr[rb as usize];
    let result = memory::lvxl(memory, ra_val, rb_val)?;
    set_vector(regs, vd, result);
    Ok(())
}

pub fn stvxl(regs: &mut Registers, memory: &dyn MemoryInterface, vs: u8, ra: u8, rb: u8) -> Result<()> {
    let ra_val = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let rb_val = regs.gpr[rb as usize];
    let vec = get_vector(regs, vs);
    memory::stvxl(memory, vec, ra_val, rb_val)?;
    Ok(())
}

pub fn lvebx(regs: &mut Registers, memory: &dyn MemoryInterface, vd: u8, ra: u8, rb: u8) -> Result<()> {
    let ra_val = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let rb_val = regs.gpr[rb as usize];
    let result = memory::lvebx(memory, ra_val, rb_val)?;
    set_vector(regs, vd, result);
    Ok(())
}

pub fn lvehx(regs: &mut Registers, memory: &dyn MemoryInterface, vd: u8, ra: u8, rb: u8) -> Result<()> {
    let ra_val = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let rb_val = regs.gpr[rb as usize];
    let result = memory::lvehx(memory, ra_val, rb_val)?;
    set_vector(regs, vd, result);
    Ok(())
}

pub fn lvewx(regs: &mut Registers, memory: &dyn MemoryInterface, vd: u8, ra: u8, rb: u8) -> Result<()> {
    let ra_val = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let rb_val = regs.gpr[rb as usize];
    let result = memory::lvewx(memory, ra_val, rb_val)?;
    set_vector(regs, vd, result);
    Ok(())
}

pub fn stvebx(regs: &mut Registers, memory: &dyn MemoryInterface, vs: u8, ra: u8, rb: u8) -> Result<()> {
    let ra_val = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let rb_val = regs.gpr[rb as usize];
    let vec = get_vector(regs, vs);
    memory::stvebx(memory, vec, ra_val, rb_val)?;
    Ok(())
}

pub fn stvehx(regs: &mut Registers, memory: &dyn MemoryInterface, vs: u8, ra: u8, rb: u8) -> Result<()> {
    let ra_val = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let rb_val = regs.gpr[rb as usize];
    let vec = get_vector(regs, vs);
    memory::stvehx(memory, vec, ra_val, rb_val)?;
    Ok(())
}

pub fn stvewx(regs: &mut Registers, memory: &dyn MemoryInterface, vs: u8, ra: u8, rb: u8) -> Result<()> {
    let ra_val = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let rb_val = regs.gpr[rb as usize];
    let vec = get_vector(regs, vs);
    memory::stvewx(memory, vec, ra_val, rb_val)?;
    Ok(())
}

pub fn lvsl(regs: &mut Registers, _memory: &dyn MemoryInterface, vd: u8, ra: u8, rb: u8) -> Result<()> {
    let ra_val = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let rb_val = regs.gpr[rb as usize];
    let result = memory::lvsl(ra_val, rb_val);
    set_vector(regs, vd, result);
    Ok(())
}

pub fn lvsr(regs: &mut Registers, _memory: &dyn MemoryInterface, vd: u8, ra: u8, rb: u8) -> Result<()> {
    let ra_val = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
    let rb_val = regs.gpr[rb as usize];
    let result = memory::lvsr(ra_val, rb_val);
    set_vector(regs, vd, result);
    Ok(())
}
