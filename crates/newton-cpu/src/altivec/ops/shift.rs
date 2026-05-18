// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! AltiVec shift and rotate operations

use crate::altivec::Vector128;

/// Vector Shift Left Byte (vslb)
pub fn vslb(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        let shift = b_bytes[i] & 0x07; // Only lower 3 bits for byte shift
        result[i] = a_bytes[i] << shift;
    }
    
    Vector128::from_bytes(result)
}

/// Vector Shift Left Halfword (vslh)
pub fn vslh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        let shift = b_hwords[i] & 0x0F; // Only lower 4 bits for halfword shift
        result[i] = a_hwords[i] << shift;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Shift Left Word (vslw)
pub fn vslw(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_words();
    let b_words = vb.as_words();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        let shift = b_words[i] & 0x1F; // Only lower 5 bits for word shift
        result[i] = a_words[i] << shift;
    }
    
    Vector128::from_words(result)
}

/// Vector Shift Right Byte (vsrb)
pub fn vsrb(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        let shift = b_bytes[i] & 0x07;
        result[i] = a_bytes[i] >> shift;
    }
    
    Vector128::from_bytes(result)
}

/// Vector Shift Right Halfword (vsrh)
pub fn vsrh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        let shift = b_hwords[i] & 0x0F;
        result[i] = a_hwords[i] >> shift;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Shift Right Word (vsrw)
pub fn vsrw(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_words();
    let b_words = vb.as_words();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        let shift = b_words[i] & 0x1F;
        result[i] = a_words[i] >> shift;
    }
    
    Vector128::from_words(result)
}

/// Vector Shift Right Algebraic Byte (vsrab)
pub fn vsrab(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_signed_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        let shift = b_bytes[i] & 0x07;
        result[i] = (a_bytes[i] >> shift) as u8;
    }
    
    Vector128::from_bytes(result)
}

/// Vector Shift Right Algebraic Halfword (vsrah)
pub fn vsrah(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_signed_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        let shift = b_hwords[i] & 0x0F;
        result[i] = (a_hwords[i] >> shift) as u16;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Shift Right Algebraic Word (vsraw)
pub fn vsraw(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_signed_words();
    let b_words = vb.as_words();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        let shift = b_words[i] & 0x1F;
        result[i] = (a_words[i] >> shift) as u32;
    }
    
    Vector128::from_words(result)
}

/// Vector Rotate Left Byte (vrlb)
pub fn vrlb(va: Vector128, vb: Vector128) -> Vector128 {
    let a_bytes = va.as_bytes();
    let b_bytes = vb.as_bytes();
    let mut result = [0u8; 16];
    
    for i in 0..16 {
        let rotate = b_bytes[i] & 0x07;
        result[i] = a_bytes[i].rotate_left(rotate as u32);
    }
    
    Vector128::from_bytes(result)
}

/// Vector Rotate Left Halfword (vrlh)
pub fn vrlh(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u16; 8];
    
    for i in 0..8 {
        let rotate = b_hwords[i] & 0x0F;
        result[i] = a_hwords[i].rotate_left(rotate as u32);
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Rotate Left Word (vrlw)
pub fn vrlw(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_words();
    let b_words = vb.as_words();
    let mut result = [0u32; 4];
    
    for i in 0..4 {
        let rotate = b_words[i] & 0x1F;
        result[i] = a_words[i].rotate_left(rotate);
    }
    
    Vector128::from_words(result)
}

/// Vector Shift Left (vsl)
/// Shifts the entire 128-bit vector left by the number of bits specified in vb[124:127]
pub fn vsl(va: Vector128, vb: Vector128) -> Vector128 {
    let bytes_a = va.as_bytes();
    let bytes_b = vb.as_bytes();
    let shift_bits = (bytes_b[15] & 0x07) as usize; // Bits to shift (0-7)
    
    let mut result = [0u8; 16];
    
    if shift_bits == 0 {
        return va;
    }
    
    // Shift entire vector left by shift_bits
    for i in 0..16 {
        let mut byte = bytes_a[i] << shift_bits;
        if i < 15 {
            byte |= bytes_a[i + 1] >> (8 - shift_bits);
        }
        result[i] = byte;
    }
    
    Vector128::from_bytes(result)
}

/// Vector Shift Right (vsr)
/// Shifts the entire 128-bit vector right by the number of bits specified in vb[124:127]
pub fn vsr(va: Vector128, vb: Vector128) -> Vector128 {
    let bytes_a = va.as_bytes();
    let bytes_b = vb.as_bytes();
    let shift_bits = (bytes_b[15] & 0x07) as usize; // Bits to shift (0-7)
    
    let mut result = [0u8; 16];
    
    if shift_bits == 0 {
        return va;
    }
    
    // Shift entire vector right by shift_bits
    for i in (0..16).rev() {
        let mut byte = bytes_a[i] >> shift_bits;
        if i > 0 {
            byte |= bytes_a[i - 1] << (8 - shift_bits);
        }
        result[i] = byte;
    }
    
    Vector128::from_bytes(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vslb() {
        let va = Vector128::from_bytes([0x01, 0x02, 0x04, 0x08, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let vb = Vector128::from_bytes([1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let result = vslb(va, vb);
        let bytes = result.as_bytes();
        
        assert_eq!(bytes[0], 0x02); // 0x01 << 1
        assert_eq!(bytes[1], 0x04); // 0x02 << 1
        assert_eq!(bytes[2], 0x08); // 0x04 << 1
        assert_eq!(bytes[3], 0x10); // 0x08 << 1
    }
    
    #[test]
    fn test_vsrb() {
        let va = Vector128::from_bytes([0x80, 0x40, 0x20, 0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let vb = Vector128::from_bytes([1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let result = vsrb(va, vb);
        let bytes = result.as_bytes();
        
        assert_eq!(bytes[0], 0x40); // 0x80 >> 1
        assert_eq!(bytes[1], 0x20); // 0x40 >> 1
        assert_eq!(bytes[2], 0x10); // 0x20 >> 1
        assert_eq!(bytes[3], 0x08); // 0x10 >> 1
    }
    
    #[test]
    fn test_vsrab() {
        let va = Vector128::from_bytes([0x80, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // -128, 64 as i8
        let vb = Vector128::from_bytes([1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let result = vsrab(va, vb);
        let bytes = result.as_bytes();
        
        assert_eq!(bytes[0], 0xC0); // -128 >> 1 = -64 (arithmetic, sign extend)
        assert_eq!(bytes[1], 0x20); // 64 >> 1 = 32
    }
    
    #[test]
    fn test_vrlb() {
        let va = Vector128::from_bytes([0b1010_0101, 0b1100_0011, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let vb = Vector128::from_bytes([2, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let result = vrlb(va, vb);
        let bytes = result.as_bytes();
        
        assert_eq!(bytes[0], 0b1001_0110); // rotate left by 2
        assert_eq!(bytes[1], 0b0011_1100); // rotate left by 4
    }
    
    #[test]
    fn test_vslw() {
        let va = Vector128::from_words([0x0000_0001, 0x0000_0002, 0x0000_0004, 0x0000_0008]);
        let vb = Vector128::from_words([4, 4, 4, 4]);
        let result = vslw(va, vb);
        let words = result.as_words();
        
        assert_eq!(words[0], 0x0000_0010);
        assert_eq!(words[1], 0x0000_0020);
        assert_eq!(words[2], 0x0000_0040);
        assert_eq!(words[3], 0x0000_0080);
    }
    
    #[test]
    fn test_vsl() {
        let va = Vector128::from_bytes([0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0, 0, 0, 0, 0, 0, 0, 0]);
        let vb = Vector128::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4]); // Shift by 4 bits
        let result = vsl(va, vb);
        let bytes = result.as_bytes();
        
        // Entire vector shifts left by 4 bits
        assert_eq!(bytes[0], 0x12);
        assert_eq!(bytes[1], 0x34);
    }
}
