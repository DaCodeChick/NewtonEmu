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
}
