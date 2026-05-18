// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! AltiVec pack and unpack operations
//!
//! Pack operations convert wider elements to narrower elements with saturation.
//! Unpack operations convert narrower elements to wider elements.

use crate::altivec::{Vector128, saturate_u8, saturate_i8, saturate_u16, saturate_i16};

/// Vector Pack Unsigned Halfword Unsigned Saturate (vpkuhus)
/// Converts 8 halfwords from va and 8 from vb to 16 bytes with unsigned saturation
pub fn vpkuhus(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_halfwords();
    let b_hwords = vb.as_halfwords();
    let mut result = [0u8; 16];
    
    // Pack va halfwords into first 8 bytes
    for i in 0..8 {
        result[i] = saturate_u8(a_hwords[i] as i16);
    }
    
    // Pack vb halfwords into last 8 bytes
    for i in 0..8 {
        result[i + 8] = saturate_u8(b_hwords[i] as i16);
    }
    
    Vector128::from_bytes(result)
}

/// Vector Pack Unsigned Halfword Signed Saturate (vpkuhss)
pub fn vpkuhss(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_signed_halfwords();
    let b_hwords = vb.as_signed_halfwords();
    let mut result = [0u8; 16];
    
    for i in 0..8 {
        result[i] = saturate_i8(a_hwords[i] as i16) as u8;
    }
    
    for i in 0..8 {
        result[i + 8] = saturate_i8(b_hwords[i] as i16) as u8;
    }
    
    Vector128::from_bytes(result)
}

/// Vector Pack Unsigned Word Unsigned Saturate (vpkuwus)
/// Converts 4 words from va and 4 from vb to 8 halfwords with unsigned saturation
pub fn vpkuwus(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_words();
    let b_words = vb.as_words();
    let mut result = [0u16; 8];
    
    // Pack va words into first 4 halfwords
    for i in 0..4 {
        result[i] = saturate_u16(a_words[i] as i32);
    }
    
    // Pack vb words into last 4 halfwords
    for i in 0..4 {
        result[i + 4] = saturate_u16(b_words[i] as i32);
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Pack Unsigned Word Signed Saturate (vpkuwss)
pub fn vpkuwss(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_signed_words();
    let b_words = vb.as_signed_words();
    let mut result = [0u16; 8];
    
    for i in 0..4 {
        result[i] = saturate_i16(a_words[i]) as u16;
    }
    
    for i in 0..4 {
        result[i + 4] = saturate_i16(b_words[i]) as u16;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Pack Signed Halfword Signed Saturate (vpkshss)
pub fn vpkshss(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_signed_halfwords();
    let b_hwords = vb.as_signed_halfwords();
    let mut result = [0u8; 16];
    
    for i in 0..8 {
        result[i] = saturate_i8(a_hwords[i] as i16) as u8;
    }
    
    for i in 0..8 {
        result[i + 8] = saturate_i8(b_hwords[i] as i16) as u8;
    }
    
    Vector128::from_bytes(result)
}

/// Vector Pack Signed Halfword Unsigned Saturate (vpkshus)
pub fn vpkshus(va: Vector128, vb: Vector128) -> Vector128 {
    let a_hwords = va.as_signed_halfwords();
    let b_hwords = vb.as_signed_halfwords();
    let mut result = [0u8; 16];
    
    for i in 0..8 {
        result[i] = saturate_u8(a_hwords[i] as i16);
    }
    
    for i in 0..8 {
        result[i + 8] = saturate_u8(b_hwords[i] as i16);
    }
    
    Vector128::from_bytes(result)
}

/// Vector Pack Signed Word Signed Saturate (vpkswss)
pub fn vpkswss(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_signed_words();
    let b_words = vb.as_signed_words();
    let mut result = [0u16; 8];
    
    for i in 0..4 {
        result[i] = saturate_i16(a_words[i]) as u16;
    }
    
    for i in 0..4 {
        result[i + 4] = saturate_i16(b_words[i]) as u16;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Pack Signed Word Unsigned Saturate (vpkswus)
pub fn vpkswus(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_signed_words();
    let b_words = vb.as_signed_words();
    let mut result = [0u16; 8];
    
    for i in 0..4 {
        result[i] = saturate_u16(a_words[i]);
    }
    
    for i in 0..4 {
        result[i + 4] = saturate_u16(b_words[i]);
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Pack Pixel (vpkpx)
/// Packs 8 pixels from 32-bit format to 16-bit 1/5/5/5 format
pub fn vpkpx(va: Vector128, vb: Vector128) -> Vector128 {
    let a_words = va.as_words();
    let b_words = vb.as_words();
    let mut result = [0u16; 8];
    
    // Pack 4 words from va into first 4 halfwords
    for i in 0..4 {
        let word = a_words[i];
        // Extract components: A(1 bit), R(5 bits), G(5 bits), B(5 bits)
        let a = ((word >> 24) & 0x80) >> 7;
        let r = ((word >> 16) & 0xF8) >> 3;
        let g = ((word >> 8) & 0xF8) >> 3;
        let b = (word & 0xF8) >> 3;
        result[i] = ((a << 15) | (r << 10) | (g << 5) | b) as u16;
    }
    
    // Pack 4 words from vb into last 4 halfwords
    for i in 0..4 {
        let word = b_words[i];
        let a = ((word >> 24) & 0x80) >> 7;
        let r = ((word >> 16) & 0xF8) >> 3;
        let g = ((word >> 8) & 0xF8) >> 3;
        let b = (word & 0xF8) >> 3;
        result[i + 4] = ((a << 15) | (r << 10) | (g << 5) | b) as u16;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Unpack High Signed Byte (vupkhsb)
/// Sign-extends the high 8 bytes to 8 halfwords
pub fn vupkhsb(vb: Vector128) -> Vector128 {
    let bytes = vb.as_signed_bytes();
    let mut result = [0u16; 8];
    
    // Unpack high 8 bytes (indices 0-7)
    for i in 0..8 {
        result[i] = bytes[i] as i16 as u16;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Unpack Low Signed Byte (vupklsb)
/// Sign-extends the low 8 bytes to 8 halfwords
pub fn vupklsb(vb: Vector128) -> Vector128 {
    let bytes = vb.as_signed_bytes();
    let mut result = [0u16; 8];
    
    // Unpack low 8 bytes (indices 8-15)
    for i in 0..8 {
        result[i] = bytes[i + 8] as i16 as u16;
    }
    
    Vector128::from_halfwords(result)
}

/// Vector Unpack High Signed Halfword (vupkhsh)
/// Sign-extends the high 4 halfwords to 4 words
pub fn vupkhsh(vb: Vector128) -> Vector128 {
    let hwords = vb.as_signed_halfwords();
    let mut result = [0u32; 4];
    
    // Unpack high 4 halfwords (indices 0-3)
    for i in 0..4 {
        result[i] = hwords[i] as i32 as u32;
    }
    
    Vector128::from_words(result)
}

/// Vector Unpack Low Signed Halfword (vupklsh)
/// Sign-extends the low 4 halfwords to 4 words
pub fn vupklsh(vb: Vector128) -> Vector128 {
    let hwords = vb.as_signed_halfwords();
    let mut result = [0u32; 4];
    
    // Unpack low 4 halfwords (indices 4-7)
    for i in 0..4 {
        result[i] = hwords[i + 4] as i32 as u32;
    }
    
    Vector128::from_words(result)
}

/// Vector Unpack High Pixel (vupkhpx)
/// Unpacks high 4 pixels from 16-bit 1/5/5/5 format to 32-bit format
pub fn vupkhpx(vb: Vector128) -> Vector128 {
    let hwords = vb.as_halfwords();
    let mut result = [0u32; 4];
    
    // Unpack high 4 halfwords (indices 0-3)
    for i in 0..4 {
        let pixel = hwords[i];
        let a = if (pixel & 0x8000) != 0 { 0xFF } else { 0x00 };
        let r = ((pixel >> 10) & 0x1F) << 3;
        let g = ((pixel >> 5) & 0x1F) << 3;
        let b = (pixel & 0x1F) << 3;
        result[i] = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    }
    
    Vector128::from_words(result)
}

/// Vector Unpack Low Pixel (vupklpx)
/// Unpacks low 4 pixels from 16-bit 1/5/5/5 format to 32-bit format
pub fn vupklpx(vb: Vector128) -> Vector128 {
    let hwords = vb.as_halfwords();
    let mut result = [0u32; 4];
    
    // Unpack low 4 halfwords (indices 4-7)
    for i in 0..4 {
        let pixel = hwords[i + 4];
        let a = if (pixel & 0x8000) != 0 { 0xFF } else { 0x00 };
        let r = ((pixel >> 10) & 0x1F) << 3;
        let g = ((pixel >> 5) & 0x1F) << 3;
        let b = (pixel & 0x1F) << 3;
        result[i] = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    }
    
    Vector128::from_words(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vpkuhus() {
        let va = Vector128::from_halfwords([10, 20, 300, 40, 50, 60, 70, 80]);
        let vb = Vector128::from_halfwords([90, 100, 110, 120, 130, 140, 150, 400]);
        let result = vpkuhus(va, vb);
        let bytes = result.as_bytes();
        
        assert_eq!(bytes[0], 10);
        assert_eq!(bytes[1], 20);
        assert_eq!(bytes[2], 255);  // 300 saturated to 255
        assert_eq!(bytes[3], 40);
        assert_eq!(bytes[15], 255); // 400 saturated to 255
    }
    
    #[test]
    fn test_vpkshss() {
        let va = Vector128::from_halfwords([
            10u16, 
            (-20i16) as u16, 
            300u16, 
            (-300i16) as u16,
            0, 0, 0, 0
        ]);
        let vb = Vector128::from_halfwords([0, 0, 0, 0, 0, 0, 0, 0]);
        let result = vpkshss(va, vb);
        let bytes = result.as_signed_bytes();
        
        assert_eq!(bytes[0], 10);
        assert_eq!(bytes[1], -20);
        assert_eq!(bytes[2], 127);   // 300 saturated to 127
        assert_eq!(bytes[3], -128);  // -300 saturated to -128
    }
    
    #[test]
    fn test_vupkhsb() {
        let vb = Vector128::from_bytes([
            10, 
            (-20i8) as u8, 
            127, 
            (-128i8) as u8,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        let result = vupkhsb(vb);
        let hwords = result.as_signed_halfwords();
        
        assert_eq!(hwords[0], 10);
        assert_eq!(hwords[1], -20);
        assert_eq!(hwords[2], 127);
        assert_eq!(hwords[3], -128);
    }
    
    #[test]
    fn test_vupklsb() {
        let vb = Vector128::from_bytes([
            0, 0, 0, 0, 0, 0, 0, 0,
            10, 
            (-20i8) as u8, 
            127, 
            (-128i8) as u8,
            0, 0, 0, 0
        ]);
        let result = vupklsb(vb);
        let hwords = result.as_signed_halfwords();
        
        assert_eq!(hwords[0], 10);
        assert_eq!(hwords[1], -20);
        assert_eq!(hwords[2], 127);
        assert_eq!(hwords[3], -128);
    }
    
    #[test]
    fn test_vupkhsh() {
        let vb = Vector128::from_halfwords([
            100, 
            (-200i16) as u16, 
            30000, 
            (-30000i16) as u16,
            0, 0, 0, 0
        ]);
        let result = vupkhsh(vb);
        let words = result.as_signed_words();
        
        assert_eq!(words[0], 100);
        assert_eq!(words[1], -200);
        assert_eq!(words[2], 30000);
        assert_eq!(words[3], -30000);
    }
    
    #[test]
    fn test_vpkpx_vupkhpx_roundtrip() {
        // Create 4 32-bit pixels with 1/5/5/5 components
        let va = Vector128::from_words([
            0xFF_F8_F8_F8,  // White with alpha
            0x00_00_00_00,  // Black no alpha
            0xFF_F8_00_00,  // Red with alpha
            0x00_00_F8_00,  // Green no alpha
        ]);
        let vb = Vector128::from_words([0, 0, 0, 0]);
        
        // Pack to 16-bit
        let packed = vpkpx(va, vb);
        
        // Unpack high 4 pixels back to 32-bit
        let unpacked = vupkhpx(packed);
        let words = unpacked.as_words();
        
        // Check that we get back the original (with some precision loss due to 5-bit components)
        assert_eq!(words[0], 0xFF_F8_F8_F8);
        assert_eq!(words[1], 0x00_00_00_00);
        assert_eq!(words[2], 0xFF_F8_00_00);
        assert_eq!(words[3], 0x00_00_F8_00);
    }
}
