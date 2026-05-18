// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! AltiVec vector data types and utilities

/// A 128-bit vector that can be interpreted as different element types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vector128 {
    /// Raw data stored as 4 x 32-bit words (matches Registers::vr layout)
    pub words: [u32; 4],
}

impl Vector128 {
    /// Create a new zero vector
    pub const fn zero() -> Self {
        Self { words: [0; 4] }
    }
    
    /// Create from raw 32-bit words
    pub const fn from_words(words: [u32; 4]) -> Self {
        Self { words }
    }
    
    /// Create from bytes (16 x 8-bit)
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        let mut words = [0u32; 4];
        for i in 0..4 {
            words[i] = u32::from_be_bytes([
                bytes[i * 4],
                bytes[i * 4 + 1],
                bytes[i * 4 + 2],
                bytes[i * 4 + 3],
            ]);
        }
        Self { words }
    }
    
    /// Create from halfwords (8 x 16-bit)
    pub fn from_halfwords(halfwords: [u16; 8]) -> Self {
        let mut words = [0u32; 4];
        for i in 0..4 {
            words[i] = ((halfwords[i * 2] as u32) << 16) | (halfwords[i * 2 + 1] as u32);
        }
        Self { words }
    }
    
    /// Create from single-precision floats (4 x 32-bit)
    pub fn from_floats(floats: [f32; 4]) -> Self {
        let words = [
            floats[0].to_bits(),
            floats[1].to_bits(),
            floats[2].to_bits(),
            floats[3].to_bits(),
        ];
        Self { words }
    }
    
    /// Get as bytes (16 x 8-bit)
    pub fn as_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        for i in 0..4 {
            let word_bytes = self.words[i].to_be_bytes();
            bytes[i * 4..i * 4 + 4].copy_from_slice(&word_bytes);
        }
        bytes
    }
    
    /// Get as signed bytes (16 x i8)
    pub fn as_signed_bytes(&self) -> [i8; 16] {
        let bytes = self.as_bytes();
        unsafe { std::mem::transmute(bytes) }
    }
    
    /// Get as halfwords (8 x 16-bit)
    pub fn as_halfwords(&self) -> [u16; 8] {
        let mut halfwords = [0u16; 8];
        for i in 0..4 {
            halfwords[i * 2] = (self.words[i] >> 16) as u16;
            halfwords[i * 2 + 1] = self.words[i] as u16;
        }
        halfwords
    }
    
    /// Get as signed halfwords (8 x i16)
    pub fn as_signed_halfwords(&self) -> [i16; 8] {
        let halfwords = self.as_halfwords();
        unsafe { std::mem::transmute(halfwords) }
    }
    
    /// Get as words (4 x 32-bit)
    pub const fn as_words(&self) -> [u32; 4] {
        self.words
    }
    
    /// Get as signed words (4 x i32)
    pub fn as_signed_words(&self) -> [i32; 4] {
        unsafe { std::mem::transmute(self.words) }
    }
    
    /// Get as single-precision floats (4 x f32)
    pub fn as_floats(&self) -> [f32; 4] {
        [
            f32::from_bits(self.words[0]),
            f32::from_bits(self.words[1]),
            f32::from_bits(self.words[2]),
            f32::from_bits(self.words[3]),
        ]
    }
}

impl From<[u32; 4]> for Vector128 {
    fn from(words: [u32; 4]) -> Self {
        Self { words }
    }
}

impl From<Vector128> for [u32; 4] {
    fn from(vec: Vector128) -> [u32; 4] {
        vec.words
    }
}

/// Saturate a signed value to i8 range
pub fn saturate_i8(val: i16) -> i8 {
    if val > i8::MAX as i16 {
        i8::MAX
    } else if val < i8::MIN as i16 {
        i8::MIN
    } else {
        val as i8
    }
}

/// Saturate an unsigned value to u8 range
pub fn saturate_u8(val: i16) -> u8 {
    if val > u8::MAX as i16 {
        u8::MAX
    } else if val < 0 {
        0
    } else {
        val as u8
    }
}

/// Saturate a signed value to i16 range
pub fn saturate_i16(val: i32) -> i16 {
    if val > i16::MAX as i32 {
        i16::MAX
    } else if val < i16::MIN as i32 {
        i16::MIN
    } else {
        val as i16
    }
}

/// Saturate an unsigned value to u16 range
pub fn saturate_u16(val: i32) -> u16 {
    if val > u16::MAX as i32 {
        u16::MAX
    } else if val < 0 {
        0
    } else {
        val as u16
    }
}

/// Saturate a signed value to i32 range (from i64)
pub fn saturate_i32(val: i64) -> i32 {
    if val > i32::MAX as i64 {
        i32::MAX
    } else if val < i32::MIN as i64 {
        i32::MIN
    } else {
        val as i32
    }
}

/// Saturate an unsigned value to u32 range (from i64)
pub fn saturate_u32(val: i64) -> u32 {
    if val > u32::MAX as i64 {
        u32::MAX
    } else if val < 0 {
        0
    } else {
        val as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vector128_bytes() {
        let bytes = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let vec = Vector128::from_bytes(bytes);
        assert_eq!(vec.as_bytes(), bytes);
    }
    
    #[test]
    fn test_vector128_words() {
        let words = [0x01020304, 0x05060708, 0x090A0B0C, 0x0D0E0F10];
        let vec = Vector128::from_words(words);
        assert_eq!(vec.as_words(), words);
    }
    
    #[test]
    fn test_saturate_i8() {
        assert_eq!(saturate_i8(127), 127);
        assert_eq!(saturate_i8(128), 127);
        assert_eq!(saturate_i8(-128), -128);
        assert_eq!(saturate_i8(-129), -128);
    }
    
    #[test]
    fn test_saturate_u8() {
        assert_eq!(saturate_u8(255), 255);
        assert_eq!(saturate_u8(256), 255);
        assert_eq!(saturate_u8(-1), 0);
        assert_eq!(saturate_u8(0), 0);
    }
}
