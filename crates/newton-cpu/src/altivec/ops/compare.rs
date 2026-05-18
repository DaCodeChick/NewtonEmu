// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! AltiVec comparison operations
//!
//! Comparison operations produce all-ones (0xFF) or all-zeros (0x00) per element.

use crate::altivec::Vector128;

/// Vector Compare Equal Unsigned Byte (vcmpequb)
pub fn vcmpequb(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        result[i] = if a_bytes[i] == b_bytes[i] { 0xFF } else { 0x00 };
    }
    
    Vector128::from_bytes(result)
}

/// Vector Compare Equal Unsigned Halfword (vcmpequh)
pub fn vcmpequh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        result[i] = if a_hwords[i] == b_hwords[i] { 0xFFFF } else { 0x0000 };
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Compare Equal Unsigned Word (vcmpequw)
pub fn vcmpequw(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_words();
    let b_words = vb.as_words();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        result[i] = if a_words[i] == b_words[i] { 0xFFFF_FFFF } else { 0x0000_0000 };
    }
    
    Vector128::from_words(result)
}

/// Vector Compare Equal Single-Precision (vcmpeqfp)
pub fn vcmpeqfp(va: Vector128, vb: Vector128) -> Vector128 {
    let a_floats = va.as_floats();
    let b_floats = vb.as_floats();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        result[i] = if a_floats[i] == b_floats[i] { 0xFFFF_FFFF } else { 0x0000_0000 };
    }
    
    Vector128::from_words(result)
}

/// Vector Compare Greater Than Unsigned Byte (vcmpgtub)
pub fn vcmpgtub(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        result[i] = if a_bytes[i] > b_bytes[i] { 0xFF } else { 0x00 };
    }
    
    Vector128::from_bytes(result)
}

/// Vector Compare Greater Than Unsigned Halfword (vcmpgtuh)
pub fn vcmpgtuh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        result[i] = if a_hwords[i] > b_hwords[i] { 0xFFFF } else { 0x0000 };
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Compare Greater Than Unsigned Word (vcmpgtuw)
pub fn vcmpgtuw(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_words();
    let b_words = vb.as_words();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        result[i] = if a_words[i] > b_words[i] { 0xFFFF_FFFF } else { 0x0000_0000 };
    }
    
    Vector128::from_words(result)
}

/// Vector Compare Greater Than Signed Byte (vcmpgtsb)
pub fn vcmpgtsb(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_signed_bytes();
    let b_bytes = vb.as_signed_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        result[i] = if a_bytes[i] > b_bytes[i] { 0xFF } else { 0x00 };
    }
    
    Vector128::from_bytes(result)
}

/// Vector Compare Greater Than Signed Halfword (vcmpgtsh)
pub fn vcmpgtsh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_signed_halfwords();
    let b_hwords = vb.as_signed_halfwords();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        result[i] = if a_hwords[i] > b_hwords[i] { 0xFFFF } else { 0x0000 };
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Compare Greater Than Signed Word (vcmpgtsw)
pub fn vcmpgtsw(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_signed_words();
    let b_words = vb.as_signed_words();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        result[i] = if a_words[i] > b_words[i] { 0xFFFF_FFFF } else { 0x0000_0000 };
    }
    
    Vector128::from_words(result)
}

/// Vector Compare Greater Than Single-Precision (vcmpgtfp)
pub fn vcmpgtfp(va: Vector128, vb: Vector128) -> Vector128 {
    let a_floats = va.as_floats();
    let b_floats = vb.as_floats();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        result[i] = if a_floats[i] > b_floats[i] { 0xFFFF_FFFF } else { 0x0000_0000 };
    }
    
    Vector128::from_words(result)
}

/// Vector Compare Greater Than or Equal Single-Precision (vcmpgefp)
pub fn vcmpgefp(va: Vector128, vb: Vector128) -> Vector128 {
    let a_floats = va.as_floats();
    let b_floats = vb.as_floats();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        result[i] = if a_floats[i] >= b_floats[i] { 0xFFFF_FFFF } else { 0x0000_0000 };
    }
    
    Vector128::from_words(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vcmpequb() {
        let va = Vector128::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let vb = Vector128::from_bytes([1, 0, 3, 0, 5, 0, 7, 0, 9, 0, 11, 0, 13, 0, 15, 0]);
        let result = vcmpequb(va, vb);
        let bytes = result.as_bytes();
        
        assert_eq!(bytes[0], 0xFF);  // 1 == 1
        assert_eq!(bytes[1], 0x00);  // 2 != 0
        assert_eq!(bytes[2], 0xFF);  // 3 == 3
        assert_eq!(bytes[3], 0x00);  // 4 != 0
    }
    
    #[test]
    fn test_vcmpgtub() {
        let va = Vector128::from_bytes([10, 5, 20, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let vb = Vector128::from_bytes([5, 10, 15, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let result = vcmpgtub(va, vb);
        let bytes = result.as_bytes();
        
        assert_eq!(bytes[0], 0xFF);  // 10 > 5
        assert_eq!(bytes[1], 0x00);  // 5 < 10
        assert_eq!(bytes[2], 0xFF);  // 20 > 15
        assert_eq!(bytes[3], 0x00);  // 15 < 20
    }
    
    #[test]
    fn test_vcmpgtsb() {
        let va = Vector128::from_bytes([250, 10, 200, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // -6, 10, -56, 50 as i8
        let vb = Vector128::from_bytes([10, 250, 50, 200, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // 10, -6, 50, -56 as i8
        let result = vcmpgtsb(va, vb);
        let bytes = result.as_bytes();
        
        // -6 < 10 = false
        assert_eq!(bytes[0], 0x00);
        // 10 > -6 = true
        assert_eq!(bytes[1], 0xFF);
        // -56 < 50 = false
        assert_eq!(bytes[2], 0x00);
        // 50 > -56 = true
        assert_eq!(bytes[3], 0xFF);
    }
    
    #[test]
    fn test_vcmpequw() {
        let va = Vector128::from_words([0x1111_1111, 0x2222_2222, 0x3333_3333, 0x4444_4444]);
        let vb = Vector128::from_words([0x1111_1111, 0x0000_0000, 0x3333_3333, 0x0000_0000]);
        let result = vcmpequw(va, vb);
        let words = result.as_words();
        
        assert_eq!(words[0], 0xFFFF_FFFF);  // equal
        assert_eq!(words[1], 0x0000_0000);  // not equal
        assert_eq!(words[2], 0xFFFF_FFFF);  // equal
        assert_eq!(words[3], 0x0000_0000);  // not equal
    }
    
    #[test]
    fn test_vcmpeqfp() {
        let va = Vector128::from_floats([1.0, 2.0, 3.0, 4.0]);
        let vb = Vector128::from_floats([1.0, 2.5, 3.0, 4.5]);
        let result = vcmpeqfp(va, vb);
        let words = result.as_words();
        
        assert_eq!(words[0], 0xFFFF_FFFF);  // 1.0 == 1.0
        assert_eq!(words[1], 0x0000_0000);  // 2.0 != 2.5
        assert_eq!(words[2], 0xFFFF_FFFF);  // 3.0 == 3.0
        assert_eq!(words[3], 0x0000_0000);  // 4.0 != 4.5
    }
    
    #[test]
    fn test_vcmpgtfp() {
        let va = Vector128::from_floats([2.0, 1.0, 3.0, 1.5]);
        let vb = Vector128::from_floats([1.0, 2.0, 3.0, 1.5]);
        let result = vcmpgtfp(va, vb);
        let words = result.as_words();
        
        assert_eq!(words[0], 0xFFFF_FFFF);  // 2.0 > 1.0
        assert_eq!(words[1], 0x0000_0000);  // 1.0 < 2.0
        assert_eq!(words[2], 0x0000_0000);  // 3.0 == 3.0 (not >)
        assert_eq!(words[3], 0x0000_0000);  // 1.5 == 1.5 (not >)
    }
}
