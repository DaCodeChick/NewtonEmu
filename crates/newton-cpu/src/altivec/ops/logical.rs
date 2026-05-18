// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! AltiVec logical operations

use crate::altivec::Vector128;

/// Vector Logical AND (vand)
pub fn vand(va: Vector128, vb: Vector128) -> Vector128 {
    let words_a = va.as_words();
    let words_b = vb.as_words();
    let result = [
        words_a[0] & words_b[0],
        words_a[1] & words_b[1],
        words_a[2] & words_b[2],
        words_a[3] & words_b[3],
    ];
    Vector128::from_words(result)
}

/// Vector Logical AND with Complement (vandc)
pub fn vandc(va: Vector128, vb: Vector128) -> Vector128 {
    let words_a = va.as_words();
    let words_b = vb.as_words();
    let result = [
        words_a[0] & !words_b[0],
        words_a[1] & !words_b[1],
        words_a[2] & !words_b[2],
        words_a[3] & !words_b[3],
    ];
    Vector128::from_words(result)
}

/// Vector Logical OR (vor)
pub fn vor(va: Vector128, vb: Vector128) -> Vector128 {
    let words_a = va.as_words();
    let words_b = vb.as_words();
    let result = [
        words_a[0] | words_b[0],
        words_a[1] | words_b[1],
        words_a[2] | words_b[2],
        words_a[3] | words_b[3],
    ];
    Vector128::from_words(result)
}

/// Vector Logical NOR (vnor)
pub fn vnor(va: Vector128, vb: Vector128) -> Vector128 {
    let words_a = va.as_words();
    let words_b = vb.as_words();
    let result = [
        !(words_a[0] | words_b[0]),
        !(words_a[1] | words_b[1]),
        !(words_a[2] | words_b[2]),
        !(words_a[3] | words_b[3]),
    ];
    Vector128::from_words(result)
}

/// Vector Logical XOR (vxor)
pub fn vxor(va: Vector128, vb: Vector128) -> Vector128 {
    let words_a = va.as_words();
    let words_b = vb.as_words();
    let result = [
        words_a[0] ^ words_b[0],
        words_a[1] ^ words_b[1],
        words_a[2] ^ words_b[2],
        words_a[3] ^ words_b[3],
    ];
    Vector128::from_words(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vand() {
        let va = Vector128::from_words([0xFFFF_FFFF, 0xFFFF_0000, 0x0000_FFFF, 0x0000_0000]);
        let vb = Vector128::from_words([0xFFFF_0000, 0xFFFF_FFFF, 0x0000_0000, 0x0000_FFFF]);
        let result = vand(va, vb);
        assert_eq!(result.as_words(), [0xFFFF_0000, 0xFFFF_0000, 0x0000_0000, 0x0000_0000]);
    }
    
    #[test]
    fn test_vor() {
        let va = Vector128::from_words([0xFFFF_0000, 0x0000_0000, 0x0000_FFFF, 0x0000_0000]);
        let vb = Vector128::from_words([0x0000_FFFF, 0x0000_0000, 0x0000_0000, 0xFFFF_FFFF]);
        let result = vor(va, vb);
        assert_eq!(result.as_words(), [0xFFFF_FFFF, 0x0000_0000, 0x0000_FFFF, 0xFFFF_FFFF]);
    }
    
    #[test]
    fn test_vxor() {
        let va = Vector128::from_words([0xFFFF_FFFF, 0xAAAA_AAAA, 0x0000_0000, 0x1234_5678]);
        let vb = Vector128::from_words([0x0000_0000, 0x5555_5555, 0xFFFF_FFFF, 0x1234_5678]);
        let result = vxor(va, vb);
        assert_eq!(result.as_words(), [0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF, 0x0000_0000]);
    }
}
