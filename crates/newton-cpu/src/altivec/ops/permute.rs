// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! AltiVec permute and merge operations

use crate::altivec::Vector128;

/// Vector Permute (vperm)
///
/// For each byte i in vd, select byte from va or vb based on control vector vc.
/// If vc[i] bit 4 is 0, select from va; if 1, select from vb.
/// The low 4 bits of vc[i] specify which byte to select (0-15).
pub fn vperm(va: Vector128, vb: Vector128, vc: Vector128) -> Vector128 {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let c_bytes = vc.as_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        let selector = c_bytes[i];
        let src_idx = (selector & 0x0F) as usize;
        
        result[i] = if (selector & 0x10) == 0 {
            a_bytes[src_idx % 16]
        } else {
            b_bytes[src_idx % 16]
        };
    }
    
    Vector128::from_bytes(result)
}

/// Vector Splat Byte (vspltb)
///
/// Replicate byte index from vb across all bytes of result
pub fn vspltb(vb: Vector128, uimm: u8) -> Vector128 {
    let b_bytes = vb.as_bytes();
    let idx = (uimm & 0x0F) as usize;
    let splat_byte = b_bytes[idx];
    let result = [splat_byte; 16];
    Vector128::from_bytes(result)
}

/// Vector Splat Halfword (vsplth)
///
/// Replicate halfword index from vb across all halfwords of result
pub fn vsplth(vb: Vector128, uimm: u8) -> Vector128 {
    let b_hwords = vb.as_halfwords();
    let idx = (uimm & 0x07) as usize;
    let splat_hword = b_hwords[idx];
    let result = [splat_hword; 8];
    Vector128::from_halfwords(result)
}

/// Vector Splat Word (vspltw)
///
/// Replicate word index from vb across all words of result
pub fn vspltw(vb: Vector128, uimm: u8) -> Vector128 {
    let b_words = vb.as_words();
    let idx = (uimm & 0x03) as usize;
    let splat_word = b_words[idx];
    let result = [splat_word; 4];
    Vector128::from_words(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vperm() {
        let va = Vector128::from_bytes([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        let vb = Vector128::from_bytes([16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]);
        
        // Select all from va (bits 4 clear)
        let vc = Vector128::from_bytes([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        let result = vperm(va, vb, vc);
        assert_eq!(result.as_bytes(), [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        
        // Select all from vb (bits 4 set)
        let vc = Vector128::from_bytes([16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]);
        let result = vperm(va, vb, vc);
        assert_eq!(result.as_bytes(), [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]);
    }
    
    #[test]
    fn test_vspltb() {
        let vb = Vector128::from_bytes([10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160]);
        let result = vspltb(vb, 2); // Splat byte index 2 (value 30)
        assert_eq!(result.as_bytes(), [30; 16]);
    }
    
    #[test]
    fn test_vspltw() {
        let vb = Vector128::from_words([0x11111111, 0x22222222, 0x33333333, 0x44444444]);
        let result = vspltw(vb, 2); // Splat word index 2
        assert_eq!(result.as_words(), [0x33333333; 4]);
    }
}
