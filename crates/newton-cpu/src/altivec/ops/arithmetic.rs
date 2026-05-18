// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! AltiVec arithmetic operations

use crate::altivec::{Vector128, saturate_i8, saturate_i16, saturate_i32};

/// Vector Add Unsigned Byte Saturate (vaddubs)
pub fn vaddubs(va: Vector128, vb: Vector128) -> (Vector128, bool) {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u8; 16];
    let mut saturated = false;
    
    for i in 0..16 {
        let sum = a_bytes[i] as u16 + b_bytes[i] as u16;
        if sum > u8::MAX as u16 {
            result[i] = u8::MAX;
            saturated = true;
        } else {
            result[i] = sum as u8;
        }
    }
    
    (Vector128::from_bytes(result), saturated)
}

/// Vector Add Unsigned Halfword Saturate (vadduhs)
pub fn vadduhs(va: Vector128, vb: Vector128) -> (Vector128, bool) {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u16; 8];
    let mut saturated = false;
    
    for i in 0..8 {
        let sum = a_hwords[i] as u32 + b_hwords[i] as u32;
        if sum > u16::MAX as u32 {
            result[i] = u16::MAX;
            saturated = true;
        } else {
            result[i] = sum as u16;
        }
    }
    
    (Vector128::from_halfwords(result), saturated)
}

/// Vector Add Unsigned Word Saturate (vadduws)
pub fn vadduws(va: Vector128, vb: Vector128) -> (Vector128, bool) {
    let a_words = va.as_words();
    let b_words = vb.as_words();
    let mut result = [0u32; 4];
    let mut saturated = false;
    
    for i in 0..4 {
        let sum = a_words[i] as u64 + b_words[i] as u64;
        if sum > u32::MAX as u64 {
            result[i] = u32::MAX;
            saturated = true;
        } else {
            result[i] = sum as u32;
        }
    }
    
    (Vector128::from_words(result), saturated)
}

/// Vector Add Signed Byte Saturate (vaddsbs)
pub fn vaddsbs(va: Vector128, vb: Vector128) -> (Vector128, bool) {
    let a_bytes = va.as_signed_bytes();
    let b_bytes = vb.as_signed_bytes();
    let mut result = [0u8; 16];
    let mut saturated = false;
    
    for i in 0..16 {
        let sum = a_bytes[i] as i16 + b_bytes[i] as i16;
        let sat_result = saturate_i8(sum);
        if sat_result as i16 != sum {
            saturated = true;
        }
        result[i] = sat_result as u8;
    }
    
    (Vector128::from_bytes(result), saturated)
}

/// Vector Add Signed Halfword Saturate (vaddshs)
pub fn vaddshs(va: Vector128, vb: Vector128) -> (Vector128, bool) {
    let a_hwords = va.as_signed_halfwords();
    let b_hwords = vb.as_signed_halfwords();
    let mut result = [0u16; 8];
    let mut saturated = false;
    
    for i in 0..8 {
        let sum = a_hwords[i] as i32 + b_hwords[i] as i32;
        let sat_result = saturate_i16(sum);
        if sat_result as i32 != sum {
            saturated = true;
        }
        result[i] = sat_result as u16;
    }
    
    (Vector128::from_halfwords(result), saturated)
}

/// Vector Add Signed Word Saturate (vaddsws)
pub fn vaddsws(va: Vector128, vb: Vector128) -> (Vector128, bool) {
    let a_words = va.as_signed_words();
    let b_words = vb.as_signed_words();
    let mut result = [0u32; 4];
    let mut saturated = false;
    
    for i in 0..4 {
        let sum = a_words[i] as i64 + b_words[i] as i64;
        let sat_result = saturate_i32(sum);
        if sat_result as i64 != sum {
            saturated = true;
        }
        result[i] = sat_result as u32;
    }
    
    (Vector128::from_words(result), saturated)
}

/// Vector Add Unsigned Byte Modulo (vaddubm)
pub fn vaddubm(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        result[i] = a_bytes[i].wrapping_add(b_bytes[i]);
    }
    
    Vector128::from_bytes(result)
}

/// Vector Add Unsigned Halfword Modulo (vadduhm)
pub fn vadduhm(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        result[i] = a_hwords[i].wrapping_add(b_hwords[i]);
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Add Unsigned Word Modulo (vadduwm)
pub fn vadduwm(va: Vector128, vb: Vector128) -> Vector128 {
    let words_a = va.as_words();
    let words_b = vb.as_words();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        result[i] = words_a[i].wrapping_add(words_b[i]);
    }
    
    Vector128::from_words(result)
}

/// Vector Add Single-Precision (vaddfp)
pub fn vaddfp(va: Vector128, vb: Vector128) -> Vector128 {
    let a_floats = va.as_floats();
    let b_floats = vb.as_floats();
    let mut result = [0.0f32; 4];
    
    for i in 0..4 {
        result[i] = a_floats[i] + b_floats[i];
    }
    
    Vector128::from_floats(result)
}

/// Vector Subtract Unsigned Byte Modulo (vsububm)
pub fn vsububm(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        result[i] = a_bytes[i].wrapping_sub(b_bytes[i]);
    }
    
    Vector128::from_bytes(result)
}

/// Vector Subtract Unsigned Halfword Modulo (vsubuhm)
pub fn vsubuhm(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        result[i] = a_hwords[i].wrapping_sub(b_hwords[i]);
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Subtract Unsigned Word Modulo (vsubuwm)
pub fn vsubuwm(va: Vector128, vb: Vector128) -> Vector128 {
    let words_a = va.as_words();
    let words_b = vb.as_words();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        result[i] = words_a[i].wrapping_sub(words_b[i]);
    }
    
    Vector128::from_words(result)
}

/// Vector Subtract Single-Precision (vsubfp)
pub fn vsubfp(va: Vector128, vb: Vector128) -> Vector128 {
    let a_floats = va.as_floats();
    let b_floats = vb.as_floats();
    let mut result = [0.0f32; 4];
    
    for i in 0..4 {
        result[i] = a_floats[i] - b_floats[i];
    }
    
    Vector128::from_floats(result)
}

/// Vector Multiply Even Unsigned Byte (vmuleub)
pub fn vmuleub(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u16; 8];
    
    // Multiply even-indexed bytes (0, 2, 4, 6, 8, 10, 12, 14)
    for i in 0..8 {
        let src_idx = i * 2;
        result[i] = a_bytes[src_idx] as u16 * b_bytes[src_idx] as u16;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Multiply Odd Unsigned Byte (vmuloub)
pub fn vmuloub(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u16; 8];
    
    // Multiply odd-indexed bytes (1, 3, 5, 7, 9, 11, 13, 15)
    for i in 0..8 {
        let src_idx = i * 2 + 1;
        result[i] = a_bytes[src_idx] as u16 * b_bytes[src_idx] as u16;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Multiply Even Signed Byte (vmulesb)
pub fn vmulesb(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_signed_bytes();
    let b_bytes = vb.as_signed_bytes();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        let src_idx = i * 2;
        let product = a_bytes[src_idx] as i16 * b_bytes[src_idx] as i16;
        result[i] = product as u16;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Multiply Odd Signed Byte (vmulosb)
pub fn vmulosb(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_signed_bytes();
    let b_bytes = vb.as_signed_bytes();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        let src_idx = i * 2 + 1;
        let product = a_bytes[src_idx] as i16 * b_bytes[src_idx] as i16;
        result[i] = product as u16;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Multiply Even Unsigned Halfword (vmuleuh)
pub fn vmuleuh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u32; 4];
    
    // Multiply even-indexed halfwords (0, 2, 4, 6)
    for i in 0..4 {
        let src_idx = i * 2;
        result[i] = a_hwords[src_idx] as u32 * b_hwords[src_idx] as u32;
    }
    
    Vector128::from_words(result)
}

/// Vector Multiply Odd Unsigned Halfword (vmulouh)
pub fn vmulouh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u32; 4];
    
    // Multiply odd-indexed halfwords (1, 3, 5, 7)
    for i in 0..4 {
        let src_idx = i * 2 + 1;
        result[i] = a_hwords[src_idx] as u32 * b_hwords[src_idx] as u32;
    }
    
    Vector128::from_words(result)
}

/// Vector Multiply Even Signed Halfword (vmulesh)
pub fn vmulesh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_signed_halfwords();
    let b_hwords = vb.as_signed_halfwords();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        let src_idx = i * 2;
        let product = a_hwords[src_idx] as i32 * b_hwords[src_idx] as i32;
        result[i] = product as u32;
    }
    
    Vector128::from_words(result)
}

/// Vector Multiply Odd Signed Halfword (vmulosh)
pub fn vmulosh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_signed_halfwords();
    let b_hwords = vb.as_signed_halfwords();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        let src_idx = i * 2 + 1;
        let product = a_hwords[src_idx] as i32 * b_hwords[src_idx] as i32;
        result[i] = product as u32;
    }
    
    Vector128::from_words(result)
}

/// Vector Multiply Add Single-Precision (vmaddfp)
pub fn vmaddfp(va: Vector128, vb: Vector128, vc: Vector128) -> Vector128 {
    let a_floats = va.as_floats();
    let b_floats = vb.as_floats();
    let c_floats = vc.as_floats();
    let mut result = [0.0f32; 4];
    
    for i in 0..4 {
        result[i] = a_floats[i] * b_floats[i] + c_floats[i];
    }
    
    Vector128::from_floats(result)
}

/// Vector Negative Multiply-Subtract Single-Precision (vnmsubfp)
pub fn vnmsubfp(va: Vector128, vb: Vector128, vc: Vector128) -> Vector128 {
    let a_floats = va.as_floats();
    let b_floats = vb.as_floats();
    let c_floats = vc.as_floats();
    let mut result = [0.0f32; 4];
    
    for i in 0..4 {
        result[i] = -(a_floats[i] * b_floats[i] - c_floats[i]);
    }
    
    Vector128::from_floats(result)
}

/// Vector Multiply Single-Precision (vmulfp) - Not standard, but useful
pub fn vmulfp(va: Vector128, vb: Vector128) -> Vector128 {
    let a_floats = va.as_floats();
    let b_floats = vb.as_floats();
    let mut result = [0.0f32; 4];
    
    for i in 0..4 {
        result[i] = a_floats[i] * b_floats[i];
    }
    
    Vector128::from_floats(result)
}

/// Vector Sum Across Quarter Signed Word Saturate (vsumsws)
pub fn vsumsws(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_signed_words();
    let b_words = vb.as_signed_words();
    
    // Sum all four words from va
    let sum = a_words.iter().fold(0i64, |acc, &x| acc + x as i64);
    // Add word 3 from vb
    let total = sum + b_words[3] as i64;
    
    // Saturate to i32 range
    let saturated = saturate_i32(total);
    
    Vector128::from_words([0, 0, 0, saturated as u32])
}

/// Vector Sum Across Half Signed Word Saturate (vsum2sws)
pub fn vsum2sws(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_signed_words();
    let b_words = vb.as_signed_words();
    let mut result = [0u32; 4];
    
    // Sum pairs: (0+1) and (2+3)
    let sum0 = a_words[0] as i64 + a_words[1] as i64 + b_words[1] as i64;
    let sum1 = a_words[2] as i64 + a_words[3] as i64 + b_words[3] as i64;
    
    result[1] = saturate_i32(sum0) as u32;
    result[3] = saturate_i32(sum1) as u32;
    
    Vector128::from_words(result)
}

/// Vector Maximum Signed Byte (vmaxsb)
pub fn vmaxsb(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_signed_bytes();
    let b_bytes = vb.as_signed_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        result[i] = a_bytes[i].max(b_bytes[i]) as u8;
    }
    
    Vector128::from_bytes(result)
}

/// Vector Maximum Unsigned Byte (vmaxub)
pub fn vmaxub(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        result[i] = a_bytes[i].max(b_bytes[i]);
    }
    
    Vector128::from_bytes(result)
}

/// Vector Maximum Signed Halfword (vmaxsh)
pub fn vmaxsh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_signed_halfwords();
    let b_hwords = vb.as_signed_halfwords();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        result[i] = a_hwords[i].max(b_hwords[i]) as u16;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Maximum Unsigned Halfword (vmaxuh)
pub fn vmaxuh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        result[i] = a_hwords[i].max(b_hwords[i]);
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Maximum Signed Word (vmaxsw)
pub fn vmaxsw(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_signed_words();
    let b_words = vb.as_signed_words();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        result[i] = a_words[i].max(b_words[i]) as u32;
    }
    
    Vector128::from_words(result)
}

/// Vector Maximum Unsigned Word (vmaxuw)
pub fn vmaxuw(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_words();
    let b_words = vb.as_words();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        result[i] = a_words[i].max(b_words[i]);
    }
    
    Vector128::from_words(result)
}

/// Vector Maximum Single-Precision (vmaxfp)
pub fn vmaxfp(va: Vector128, vb: Vector128) -> Vector128 {
    let a_floats = va.as_floats();
    let b_floats = vb.as_floats();
    let mut result = [0.0f32; 4];
    
    for i in 0..4 {
        result[i] = a_floats[i].max(b_floats[i]);
    }
    
    Vector128::from_floats(result)
}

/// Vector Minimum Signed Byte (vminsb)
pub fn vminsb(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_signed_bytes();
    let b_bytes = vb.as_signed_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        result[i] = a_bytes[i].min(b_bytes[i]) as u8;
    }
    
    Vector128::from_bytes(result)
}

/// Vector Minimum Unsigned Byte (vminub)
pub fn vminub(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        result[i] = a_bytes[i].min(b_bytes[i]);
    }
    
    Vector128::from_bytes(result)
}

/// Vector Minimum Signed Halfword (vminsh)
pub fn vminsh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_signed_halfwords();
    let b_hwords = vb.as_signed_halfwords();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        result[i] = a_hwords[i].min(b_hwords[i]) as u16;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Minimum Unsigned Halfword (vminuh)
pub fn vminuh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        result[i] = a_hwords[i].min(b_hwords[i]);
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Minimum Signed Word (vminsw)
pub fn vminsw(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_signed_words();
    let b_words = vb.as_signed_words();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        result[i] = a_words[i].min(b_words[i]) as u32;
    }
    
    Vector128::from_words(result)
}

/// Vector Minimum Unsigned Word (vminuw)
pub fn vminuw(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_words();
    let b_words = vb.as_words();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        result[i] = a_words[i].min(b_words[i]);
    }
    
    Vector128::from_words(result)
}

/// Vector Minimum Single-Precision (vminfp)
pub fn vminfp(va: Vector128, vb: Vector128) -> Vector128 {
    let a_floats = va.as_floats();
    let b_floats = vb.as_floats();
    let mut result = [0.0f32; 4];
    
    for i in 0..4 {
        result[i] = a_floats[i].min(b_floats[i]);
    }
    
    Vector128::from_floats(result)
}

/// Vector Average Unsigned Byte (vavgub)
pub fn vavgub(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        result[i] = ((a_bytes[i] as u16 + b_bytes[i] as u16 + 1) / 2) as u8;
    }
    
    Vector128::from_bytes(result)
}

/// Vector Average Signed Byte (vavgsb)
pub fn vavgsb(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_signed_bytes();
    let b_bytes = vb.as_signed_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        result[i] = (((a_bytes[i] as i16 + b_bytes[i] as i16 + 1) / 2) as i8) as u8;
    }
    
    Vector128::from_bytes(result)
}

/// Vector Average Unsigned Halfword (vavguh)
pub fn vavguh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        result[i] = ((a_hwords[i] as u32 + b_hwords[i] as u32 + 1) / 2) as u16;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Average Signed Halfword (vavgsh)
pub fn vavgsh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_signed_halfwords();
    let b_hwords = vb.as_signed_halfwords();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        result[i] = (((a_hwords[i] as i32 + b_hwords[i] as i32 + 1) / 2) as i16) as u16;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Average Unsigned Word (vavguw)
pub fn vavguw(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_words();
    let b_words = vb.as_words();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        result[i] = ((a_words[i] as u64 + b_words[i] as u64 + 1) / 2) as u32;
    }
    
    Vector128::from_words(result)
}

/// Vector Average Signed Word (vavgsw)
pub fn vavgsw(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_signed_words();
    let b_words = vb.as_signed_words();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        result[i] = (((a_words[i] as i64 + b_words[i] as i64 + 1) / 2) as i32) as u32;
    }
    
    Vector128::from_words(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vaddubm() {
        let va = Vector128::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let vb = Vector128::from_bytes([16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
        let result = vaddubm(va, vb);
        let expected = [17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17];
        assert_eq!(result.as_bytes(), expected);
    }
    
    #[test]
    fn test_vaddubs_saturate() {
        let va = Vector128::from_bytes([255, 200, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let vb = Vector128::from_bytes([1, 100, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let (result, saturated) = vaddubs(va, vb);
        let bytes = result.as_bytes();
        assert_eq!(bytes[0], 255); // saturated
        assert_eq!(bytes[1], 255); // saturated
        assert_eq!(bytes[2], 150); // not saturated
        assert!(saturated);
    }
    
    #[test]
    fn test_vaddfp() {
        let va = Vector128::from_floats([1.0, 2.0, 3.0, 4.0]);
        let vb = Vector128::from_floats([0.5, 1.5, 2.5, 3.5]);
        let result = vaddfp(va, vb);
        let floats = result.as_floats();
        assert_eq!(floats, [1.5, 3.5, 5.5, 7.5]);
    }
    
    #[test]
    fn test_vmuleub() {
        let va = Vector128::from_bytes([2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]);
        let vb = Vector128::from_bytes([10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160]);
        let result = vmuleub(va, vb);
        let hwords = result.as_halfwords();
        
        // Even indices: 0, 2, 4, 6, 8, 10, 12, 14
        assert_eq!(hwords[0], 2 * 10);    // 20
        assert_eq!(hwords[1], 4 * 30);    // 120
        assert_eq!(hwords[2], 6 * 50);    // 300
        assert_eq!(hwords[3], 8 * 70);    // 560
    }
    
    #[test]
    fn test_vmuloub() {
        let va = Vector128::from_bytes([2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]);
        let vb = Vector128::from_bytes([10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160]);
        let result = vmuloub(va, vb);
        let hwords = result.as_halfwords();
        
        // Odd indices: 1, 3, 5, 7, 9, 11, 13, 15
        assert_eq!(hwords[0], 3 * 20);    // 60
        assert_eq!(hwords[1], 5 * 40);    // 200
        assert_eq!(hwords[2], 7 * 60);    // 420
        assert_eq!(hwords[3], 9 * 80);    // 720
    }
    
    #[test]
    fn test_vmaddfp() {
        let va = Vector128::from_floats([2.0, 3.0, 4.0, 5.0]);
        let vb = Vector128::from_floats([1.5, 2.5, 3.5, 4.5]);
        let vc = Vector128::from_floats([1.0, 1.0, 1.0, 1.0]);
        let result = vmaddfp(va, vb, vc);
        let floats = result.as_floats();
        
        // (2.0 * 1.5) + 1.0 = 4.0
        // (3.0 * 2.5) + 1.0 = 8.5
        // (4.0 * 3.5) + 1.0 = 15.0
        // (5.0 * 4.5) + 1.0 = 23.5
        assert_eq!(floats, [4.0, 8.5, 15.0, 23.5]);
    }
    
    #[test]
    fn test_vmaxub() {
        let va = Vector128::from_bytes([10, 20, 30, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let vb = Vector128::from_bytes([5, 25, 20, 35, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let result = vmaxub(va, vb);
        let bytes = result.as_bytes();
        
        assert_eq!(bytes[0], 10);
        assert_eq!(bytes[1], 25);
        assert_eq!(bytes[2], 30);
        assert_eq!(bytes[3], 35);
    }
    
    #[test]
    fn test_vminub() {
        let va = Vector128::from_bytes([10, 20, 30, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let vb = Vector128::from_bytes([5, 25, 20, 35, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let result = vminub(va, vb);
        let bytes = result.as_bytes();
        
        assert_eq!(bytes[0], 5);
        assert_eq!(bytes[1], 20);
        assert_eq!(bytes[2], 20);
        assert_eq!(bytes[3], 5);
    }
    
    #[test]
    fn test_vavgub() {
        let va = Vector128::from_bytes([10, 20, 30, 40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let vb = Vector128::from_bytes([20, 30, 40, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let result = vavgub(va, vb);
        let bytes = result.as_bytes();
        
        // (10+20+1)/2 = 15
        // (20+30+1)/2 = 25
        // (30+40+1)/2 = 35
        // (40+50+1)/2 = 45
        assert_eq!(bytes[0], 15);
        assert_eq!(bytes[1], 25);
        assert_eq!(bytes[2], 35);
        assert_eq!(bytes[3], 45);
    }
}
