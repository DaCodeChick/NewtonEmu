// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! AltiVec load/store operations
//!
//! AltiVec loads and stores are always 16-byte aligned (128 bits).
//! The effective address is computed as (rA|0) + rB, then the low 4 bits are cleared.

use crate::altivec::Vector128;
use newton_utils::Result;

/// Compute aligned effective address for AltiVec memory operations
/// EA = ((rA|0) + rB) & 0xFFFFFFF0
#[inline]
pub fn compute_aligned_ea(ra: u32, rb: u32) -> u32 {
    ra.wrapping_add(rb) & 0xFFFF_FFF0
}

/// Vector Load Indexed (lvx)
/// Load 16 bytes from memory at aligned EA
pub fn lvx<M: crate::MemoryInterface>(memory: &M, ra: u32, rb: u32) -> Result<Vector128> {
    let ea = compute_aligned_ea(ra, rb);
    
    // Load 4 words (16 bytes) from memory
    let word0 = memory.read_u32(ea)?;
    let word1 = memory.read_u32(ea.wrapping_add(4))?;
    let word2 = memory.read_u32(ea.wrapping_add(8))?;
    let word3 = memory.read_u32(ea.wrapping_add(12))?;
    
    Ok(Vector128::from_words([word0, word1, word2, word3]))
}

/// Vector Store Indexed (stvx)
/// Store 16 bytes to memory at aligned EA
pub fn stvx<M: crate::MemoryInterface>(memory: &M, vs: Vector128, ra: u32, rb: u32) -> Result<()> {
    let ea = compute_aligned_ea(ra, rb);
    let words = vs.as_words();
    
    // Store 4 words (16 bytes) to memory
    memory.write_u32(ea, words[0])?;
    memory.write_u32(ea.wrapping_add(4), words[1])?;
    memory.write_u32(ea.wrapping_add(8), words[2])?;
    memory.write_u32(ea.wrapping_add(12), words[3])?;
    
    Ok(())
}

/// Vector Load Indexed Last (lvxl)
/// Same as lvx but with hint that this is the last use (for cache optimization)
pub fn lvxl<M: crate::MemoryInterface>(memory: &M, ra: u32, rb: u32) -> Result<Vector128> {
    // For now, identical to lvx (cache hints would be implemented in hardware)
    lvx(memory, ra, rb)
}

/// Vector Store Indexed Last (stvxl)
/// Same as stvx but with hint that this is the last use
pub fn stvxl<M: crate::MemoryInterface>(memory: &M, vs: Vector128, ra: u32, rb: u32) -> Result<()> {
    // For now, identical to stvx
    stvx(memory, vs, ra, rb)
}

/// Vector Load Element Byte Indexed (lvebx)
/// Load a single byte from memory into the appropriate byte position
pub fn lvebx<M: crate::MemoryInterface>(memory: &M, ra: u32, rb: u32) -> Result<Vector128> {
    let ea = ra.wrapping_add(rb);
    let byte = memory.read_u8(ea)?;
    
    // Position in vector is determined by EA[28:31] (low 4 bits)
    let index = (ea & 0x0F) as usize;
    let mut bytes = [0u8; 16];
    bytes[index] = byte;
    
    Ok(Vector128::from_bytes(bytes))
}

/// Vector Store Element Byte Indexed (stvebx)
/// Store a single byte from vector to memory
pub fn stvebx<M: crate::MemoryInterface>(memory: &M, vs: Vector128, ra: u32, rb: u32) -> Result<()> {
    let ea = ra.wrapping_add(rb);
    let bytes = vs.as_bytes();
    
    // Position in vector is determined by EA[28:31]
    let index = (ea & 0x0F) as usize;
    memory.write_u8(ea, bytes[index])?;
    
    Ok(())
}

/// Vector Load Element Halfword Indexed (lvehx)
/// Load a single halfword from memory (must be halfword-aligned)
pub fn lvehx<M: crate::MemoryInterface>(memory: &M, ra: u32, rb: u32) -> Result<Vector128> {
    let ea = (ra.wrapping_add(rb)) & 0xFFFF_FFFE; // Align to halfword
    let hword = memory.read_u16(ea)?;
    
    // Position in vector is determined by EA[28:30] (bits for halfword index)
    let index = ((ea >> 1) & 0x07) as usize;
    let mut hwords = [0u16; 8];
    hwords[index] = hword;
    
    Ok(Vector128::from_halfwords(hwords))
}

/// Vector Store Element Halfword Indexed (stvehx)
/// Store a single halfword from vector to memory
pub fn stvehx<M: crate::MemoryInterface>(memory: &M, vs: Vector128, ra: u32, rb: u32) -> Result<()> {
    let ea = (ra.wrapping_add(rb)) & 0xFFFF_FFFE;
    let hwords = vs.as_halfwords();
    
    let index = ((ea >> 1) & 0x07) as usize;
    memory.write_u16(ea, hwords[index])?;
    
    Ok(())
}

/// Vector Load Element Word Indexed (lvewx)
/// Load a single word from memory (must be word-aligned)
pub fn lvewx<M: crate::MemoryInterface>(memory: &M, ra: u32, rb: u32) -> Result<Vector128> {
    let ea = (ra.wrapping_add(rb)) & 0xFFFF_FFFC; // Align to word
    let word = memory.read_u32(ea)?;
    
    // Position in vector is determined by EA[28:29] (bits for word index)
    let index = ((ea >> 2) & 0x03) as usize;
    let mut words = [0u32; 4];
    words[index] = word;
    
    Ok(Vector128::from_words(words))
}

/// Vector Store Element Word Indexed (stvewx)
/// Store a single word from vector to memory
pub fn stvewx<M: crate::MemoryInterface>(memory: &M, vs: Vector128, ra: u32, rb: u32) -> Result<()> {
    let ea = (ra.wrapping_add(rb)) & 0xFFFF_FFFC;
    let words = vs.as_words();
    
    let index = ((ea >> 2) & 0x03) as usize;
    memory.write_u32(ea, words[index])?;
    
    Ok(())
}

/// Vector Load for Shift Left (lvsl)
/// Generates a permute control vector for left shift of unaligned data
pub fn lvsl(ra: u32, rb: u32) -> Vector128 {
    let ea = ra.wrapping_add(rb);
    let shift = (ea & 0x0F) as u8;
    
    // Generate control vector for vperm to shift left
    let mut bytes = [0u8; 16];
    for i in 0..16 {
        bytes[i] = (shift.wrapping_add(i as u8)) & 0x1F;
    }
    
    Vector128::from_bytes(bytes)
}

/// Vector Load for Shift Right (lvsr)
/// Generates a permute control vector for right shift of unaligned data
pub fn lvsr(ra: u32, rb: u32) -> Vector128 {
    let ea = ra.wrapping_add(rb);
    let shift = (ea & 0x0F) as u8;
    
    // Generate control vector for vperm to shift right
    let mut bytes = [0u8; 16];
    for i in 0..16 {
        bytes[i] = (16u8.wrapping_sub(shift).wrapping_add(i as u8)) & 0x1F;
    }
    
    Vector128::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    
    // Simple test memory implementation
    struct TestMemory {
        data: RefCell<HashMap<u32, u8>>,
    }
    
    impl TestMemory {
        fn new() -> Self {
            Self {
                data: RefCell::new(HashMap::new()),
            }
        }
        
        fn write_bytes(&self, addr: u32, bytes: &[u8]) {
            let mut data = self.data.borrow_mut();
            for (i, &byte) in bytes.iter().enumerate() {
                data.insert(addr + i as u32, byte);
            }
        }
    }
    
    impl crate::MemoryInterface for TestMemory {
        fn read_u8(&self, addr: u32) -> Result<u8> {
            Ok(*self.data.borrow().get(&addr).unwrap_or(&0))
        }
        
        fn read_u16(&self, addr: u32) -> Result<u16> {
            let hi = self.read_u8(addr)?;
            let lo = self.read_u8(addr + 1)?;
            Ok((hi as u16) << 8 | lo as u16)
        }
        
        fn read_u32(&self, addr: u32) -> Result<u32> {
            let b0 = self.read_u8(addr)?;
            let b1 = self.read_u8(addr + 1)?;
            let b2 = self.read_u8(addr + 2)?;
            let b3 = self.read_u8(addr + 3)?;
            Ok((b0 as u32) << 24 | (b1 as u32) << 16 | (b2 as u32) << 8 | b3 as u32)
        }
        
        fn read_u64(&self, addr: u32) -> Result<u64> {
            let hi = self.read_u32(addr)?;
            let lo = self.read_u32(addr + 4)?;
            Ok((hi as u64) << 32 | lo as u64)
        }
        
        fn write_u8(&self, addr: u32, value: u8) -> Result<()> {
            self.data.borrow_mut().insert(addr, value);
            Ok(())
        }
        
        fn write_u16(&self, addr: u32, value: u16) -> Result<()> {
            self.write_u8(addr, (value >> 8) as u8)?;
            self.write_u8(addr + 1, value as u8)?;
            Ok(())
        }
        
        fn write_u32(&self, addr: u32, value: u32) -> Result<()> {
            self.write_u8(addr, (value >> 24) as u8)?;
            self.write_u8(addr + 1, (value >> 16) as u8)?;
            self.write_u8(addr + 2, (value >> 8) as u8)?;
            self.write_u8(addr + 3, value as u8)?;
            Ok(())
        }
        
        fn write_u64(&self, addr: u32, value: u64) -> Result<()> {
            self.write_u32(addr, (value >> 32) as u32)?;
            self.write_u32(addr + 4, value as u32)?;
            Ok(())
        }
    }
    
    #[test]
    fn test_compute_aligned_ea() {
        assert_eq!(compute_aligned_ea(0x1000, 0x08), 0x1000);  // 0x1008 -> 0x1000
        assert_eq!(compute_aligned_ea(0x1001, 0x07), 0x1000);  // 0x1008 -> 0x1000
        assert_eq!(compute_aligned_ea(0x100F, 0x01), 0x1010);  // 0x1010 -> 0x1010
        assert_eq!(compute_aligned_ea(0x1010, 0x00), 0x1010);  // 0x1010 -> 0x1010
        assert_eq!(compute_aligned_ea(0x1000, 0x0F), 0x1000);  // 0x100F -> 0x1000
    }
    
    #[test]
    fn test_lvx_stvx() {
        let memory = TestMemory::new();
        let vector = Vector128::from_words([0x11223344, 0x55667788, 0x99AABBCC, 0xDDEEFF00]);
        
        // Store vector at aligned address
        stvx(&memory, vector, 0x1000, 0x00).unwrap();
        
        // Load it back
        let loaded = lvx(&memory, 0x1000, 0x00).unwrap();
        assert_eq!(loaded.as_words(), vector.as_words());
        
        // Test alignment - storing at 0x1004 should still store at 0x1000
        let vector2 = Vector128::from_words([0xAAAAAAAA, 0xBBBBBBBB, 0xCCCCCCCC, 0xDDDDDDDD]);
        stvx(&memory, vector2, 0x1004, 0x00).unwrap();
        let loaded2 = lvx(&memory, 0x1000, 0x00).unwrap();
        assert_eq!(loaded2.as_words(), vector2.as_words());
    }
    
    #[test]
    fn test_lvebx_stvebx() {
        let memory = TestMemory::new();
        let vector = Vector128::from_bytes([
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
        ]);
        
        // Store byte 5 (value 5) to address 0x1005
        stvebx(&memory, vector, 0x1000, 0x05).unwrap();
        
        // Load byte from 0x1005
        let loaded = lvebx(&memory, 0x1000, 0x05).unwrap();
        let bytes = loaded.as_bytes();
        
        // Should have loaded value 5 into position 5
        assert_eq!(bytes[5], 5);
        // Other positions should be 0
        assert_eq!(bytes[0], 0);
        assert_eq!(bytes[15], 0);
    }
    
    #[test]
    fn test_lvsl_lvsr() {
        // Test lvsl with EA = 0x1005 (shift = 5)
        let lvsl_result = lvsl(0x1000, 0x05);
        let bytes = lvsl_result.as_bytes();
        
        // Should generate: 5, 6, 7, ..., 20 (& 0x1F)
        assert_eq!(bytes[0], 5);
        assert_eq!(bytes[1], 6);
        assert_eq!(bytes[10], 15);
        assert_eq!(bytes[11], 16);
        
        // Test lvsr with same EA
        let lvsr_result = lvsr(0x1000, 0x05);
        let bytes = lvsr_result.as_bytes();
        
        // Should generate: 11, 12, 13, ..., 26 (& 0x1F)
        assert_eq!(bytes[0], 11);
        assert_eq!(bytes[1], 12);
    }
}
