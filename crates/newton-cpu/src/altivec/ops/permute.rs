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

/// Vector Merge High Byte (vmrghb)
/// Interleaves high bytes from va and vb
pub fn vmrghb(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u8; 16];
    
    // Interleave high 8 bytes (indices 0-7)
    for i in 0..8 {
        result[i * 2] = a_bytes[i];
        result[i * 2 + 1] = b_bytes[i];
    }
    
    Vector128::from_bytes(result)
}

/// Vector Merge Low Byte (vmrglb)
/// Interleaves low bytes from va and vb
pub fn vmrglb(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u8; 16];
    
    // Interleave low 8 bytes (indices 8-15)
    for i in 0..8 {
        result[i * 2] = a_bytes[i + 8];
        result[i * 2 + 1] = b_bytes[i + 8];
    }
    
    Vector128::from_bytes(result)
}

/// Vector Merge High Halfword (vmrghh)
/// Interleaves high halfwords from va and vb
pub fn vmrghh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u16; 8];
    
    // Interleave high 4 halfwords (indices 0-3)
    for i in 0..4 {
        result[i * 2] = a_hwords[i];
        result[i * 2 + 1] = b_hwords[i];
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Merge Low Halfword (vmrglh)
/// Interleaves low halfwords from va and vb
pub fn vmrglh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u16; 8];
    
    // Interleave low 4 halfwords (indices 4-7)
    for i in 0..4 {
        result[i * 2] = a_hwords[i + 4];
        result[i * 2 + 1] = b_hwords[i + 4];
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Merge High Word (vmrghw)
/// Interleaves high words from va and vb
pub fn vmrghw(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_words();
    let b_words = vb.as_words();
    let result = [
        a_words[0],
        b_words[0],
        a_words[1],
        b_words[1],
    ];
    
    Vector128::from_words(result)
}

/// Vector Merge Low Word (vmrglw)
/// Interleaves low words from va and vb
pub fn vmrglw(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_words();
    let b_words = vb.as_words();
    let result = [
        a_words[2],
        b_words[2],
        a_words[3],
        b_words[3],
    ];
    
    Vector128::from_words(result)
}

/// Vector Splat Immediate Signed Byte (vspltisb)
/// Replicates a signed immediate value across all bytes
pub fn vspltisb(simm: i8) -> Vector128 {
    let byte = simm as u8;
    Vector128::from_bytes([byte; 16])
}

/// Vector Splat Immediate Signed Halfword (vspltish)
/// Replicates a signed immediate value across all halfwords
pub fn vspltish(simm: i16) -> Vector128 {
    let hword = simm as u16;
    Vector128::from_halfwords([hword; 8])
}

/// Vector Splat Immediate Signed Word (vspltisw)
/// Replicates a signed immediate value across all words
pub fn vspltisw(simm: i32) -> Vector128 {
    let word = simm as u32;
    Vector128::from_words([word; 4])
}

/// Vector Select (vsel)
/// For each bit position: result = (vb & vc) | (va & ~vc)
/// This is a bitwise multiplexer: vc acts as a mask
pub fn vsel(va: Vector128, vb: Vector128, vc: Vector128) -> Vector128 {
    let a_words = va.as_words();
    let b_words = vb.as_words();
    let c_words = vc.as_words();
    let result = [
        (b_words[0] & c_words[0]) | (a_words[0] & !c_words[0]),
        (b_words[1] & c_words[1]) | (a_words[1] & !c_words[1]),
        (b_words[2] & c_words[2]) | (a_words[2] & !c_words[2]),
        (b_words[3] & c_words[3]) | (a_words[3] & !c_words[3]),
    ];
    
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
    
    #[test]
    fn test_vmrghb() {
        let va = Vector128::from_bytes([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        let vb = Vector128::from_bytes([100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115]);
        let result = vmrghb(va, vb);
        let bytes = result.as_bytes();
        
        // Should interleave: a[0], b[0], a[1], b[1], ...
        assert_eq!(bytes[0], 0);
        assert_eq!(bytes[1], 100);
        assert_eq!(bytes[2], 1);
        assert_eq!(bytes[3], 101);
        assert_eq!(bytes[4], 2);
        assert_eq!(bytes[5], 102);
    }
    
    #[test]
    fn test_vmrglb() {
        let va = Vector128::from_bytes([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        let vb = Vector128::from_bytes([100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115]);
        let result = vmrglb(va, vb);
        let bytes = result.as_bytes();
        
        // Should interleave low 8 bytes: a[8], b[8], a[9], b[9], ...
        assert_eq!(bytes[0], 8);
        assert_eq!(bytes[1], 108);
        assert_eq!(bytes[2], 9);
        assert_eq!(bytes[3], 109);
    }
    
    #[test]
    fn test_vmrghw() {
        let va = Vector128::from_words([0x11111111, 0x22222222, 0x33333333, 0x44444444]);
        let vb = Vector128::from_words([0xAAAAAAAA, 0xBBBBBBBB, 0xCCCCCCCC, 0xDDDDDDDD]);
        let result = vmrghw(va, vb);
        let words = result.as_words();
        
        // Should interleave high 2 words: a[0], b[0], a[1], b[1]
        assert_eq!(words[0], 0x11111111);
        assert_eq!(words[1], 0xAAAAAAAA);
        assert_eq!(words[2], 0x22222222);
        assert_eq!(words[3], 0xBBBBBBBB);
    }
    
    #[test]
    fn test_vspltisb() {
        let result = vspltisb(-5);
        let bytes = result.as_signed_bytes();
        assert_eq!(bytes, [-5; 16]);
    }
    
    #[test]
    fn test_vspltisw() {
        let result = vspltisw(-1);
        let words = result.as_signed_words();
        assert_eq!(words, [-1; 4]);
    }
    
    #[test]
    fn test_vsel() {
        let va = Vector128::from_words([0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF]);
        let vb = Vector128::from_words([0x0000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0000]);
        let vc = Vector128::from_words([0xFFFF_0000, 0x0000_FFFF, 0xF0F0_F0F0, 0x0F0F_0F0F]);
        let result = vsel(va, vb, vc);
        let words = result.as_words();
        
        // Where vc is 1, select from vb (0x00000000)
        // Where vc is 0, select from va (0xFFFFFFFF)
        assert_eq!(words[0], 0x0000_FFFF);
        assert_eq!(words[1], 0xFFFF_0000);
        assert_eq!(words[2], 0x0F0F_0F0F);
        assert_eq!(words[3], 0xF0F0_F0F0);
    }
}
