// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! LZSS decompression for Mac ROM images
//!
//! Mac OS ROM images use LZSS (Lempel-Ziv-Storer-Szymanski) compression.
//! The compressed data is wrapped in a 'prcl' (parcel) container format.

use newton_utils::Result;

/// LZSS ring buffer size (typically 4096 for classic LZSS)
const RING_SIZE: usize = 4096;

/// Lookahead buffer size
const LOOKAHEAD_SIZE: usize = 18;

/// Threshold for match length
const THRESHOLD: usize = 2;

/// Decompress LZSS data
///
/// This implements the classic LZSS algorithm used in Mac ROMs.
/// The algorithm uses a sliding window dictionary with back-references.
pub fn decompress_lzss(compressed: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut ring_buffer = [0u8; RING_SIZE];
    let mut ring_pos = RING_SIZE - LOOKAHEAD_SIZE;
    
    let mut input_pos = 0;
    
    while input_pos < compressed.len() {
        // Read flags byte (8 bits for next 8 operations)
        if input_pos >= compressed.len() {
            break;
        }
        let flags = compressed[input_pos];
        input_pos += 1;
        
        for i in 0..8 {
            if input_pos >= compressed.len() {
                break;
            }
            
            if (flags & (1 << i)) != 0 {
                // Literal byte
                let byte = compressed[input_pos];
                input_pos += 1;
                
                output.push(byte);
                ring_buffer[ring_pos] = byte;
                ring_pos = (ring_pos + 1) % RING_SIZE;
            } else {
                // Back-reference (offset + length)
                if input_pos + 1 >= compressed.len() {
                    break;
                }
                
                let byte1 = compressed[input_pos] as usize;
                let byte2 = compressed[input_pos + 1] as usize;
                input_pos += 2;
                
                // Decode offset and length
                // Format: [offset_high:4][offset_low:8][length:4]
                let offset = ((byte1 & 0xF0) << 4) | byte2;
                let length = (byte1 & 0x0F) + THRESHOLD + 1;
                
                // Copy from ring buffer
                for _ in 0..length {
                    let byte = ring_buffer[offset % RING_SIZE];
                    output.push(byte);
                    ring_buffer[ring_pos] = byte;
                    ring_pos = (ring_pos + 1) % RING_SIZE;
                }
            }
        }
    }
    
    Ok(output)
}

/// Parse and decompress a 'prcl' parcel
///
/// Mac ROM uses a container format with 'prcl' magic.
/// Format:
///   [0x00] magic: 'prcl' (4 bytes)
///   [0x04] version? (4 bytes)
///   [0x08] unknown (4 bytes)
///   [0x0C] unknown (4 bytes)
///   [0x10] unknown (4 bytes)
///   [0x14] chunk_size (4 bytes)
///   [0x18] name (4 bytes)
///   [0x1C...] data
pub fn decompress_prcl(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 32 {
        return Err(newton_utils::Error::Other("Data too small for prcl header".to_string()));
    }
    
    // Check magic
    if &data[0..4] != b"prcl" {
        return Err(newton_utils::Error::Other("Not a prcl container".to_string()));
    }
    
    // Parse header (big-endian)
    let chunk_size = u32::from_be_bytes([data[0x14], data[0x15], data[0x16], data[0x17]]) as usize;
    
    tracing::debug!("prcl container: chunk_size={} (0x{:X})", chunk_size, chunk_size);
    
    // The compressed data starts after the header
    // But the format is more complex - it's not directly LZSS
    // For now, just return the raw data minus the header
    if data.len() < 32 + chunk_size {
        return Err(newton_utils::Error::Other("Data too small for chunk_size".to_string()));
    }
    
    // The 'prcl' format is actually a container for structured data
    // The actual Mac Toolbox image is not directly LZSS compressed,
    // but uses a custom format that the ELF bootloader understands.
    // For now, return the payload as-is
    Ok(data[32..32 + chunk_size].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lzss_literal() {
        // Simple literal-only LZSS data
        let compressed = vec![
            0xFF, // All literals
            b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o',
        ];
        
        let result = decompress_lzss(&compressed).unwrap();
        assert_eq!(&result[..8], b"Hello Wo");
    }
    
    #[test]
    fn test_prcl_magic() {
        let mut data = vec![0u8; 64];
        data[0..4].copy_from_slice(b"prcl");
        data[0x14] = 0;
        data[0x15] = 0;
        data[0x16] = 0;
        data[0x17] = 16; // chunk_size = 16
        
        let result = decompress_prcl(&data);
        assert!(result.is_ok());
    }
}
