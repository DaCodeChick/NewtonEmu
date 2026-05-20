// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Framebuffer MMIO device
//! 
//! Maps framebuffer into emulated address space so ROM/OS can write pixels

use super::Framebuffer;
use crate::MmioDevice;
use newton_utils::Result;
use std::sync::Arc;
use parking_lot::RwLock;

/// Framebuffer MMIO device
/// 
/// Provides memory-mapped access to the framebuffer for emulated code
pub struct FramebufferMmio {
    /// Shared framebuffer reference
    framebuffer: Arc<RwLock<Framebuffer>>,
}

impl FramebufferMmio {
    /// Create a new framebuffer MMIO device
    pub fn new(framebuffer: Arc<RwLock<Framebuffer>>) -> Self {
        Self {
            framebuffer,
        }
    }
}

impl MmioDevice for FramebufferMmio {
    fn name(&self) -> &str {
        "Framebuffer"
    }
    
    fn read(&self, offset: u32, size: u8) -> Result<u32> {
        let fb = self.framebuffer.read();
        let buffer = fb.buffer();
        let buffer_lock = buffer.read();
        
        match size {
            1 => {
                if (offset as usize) < buffer_lock.len() {
                    Ok(buffer_lock[offset as usize] as u32)
                } else {
                    tracing::warn!("Framebuffer read out of bounds: offset=0x{:X}, size={}", offset, size);
                    Ok(0)
                }
            }
            2 => {
                if (offset as usize) + 1 < buffer_lock.len() {
                    let bytes = [buffer_lock[offset as usize], buffer_lock[offset as usize + 1]];
                    Ok(u16::from_be_bytes(bytes) as u32)
                } else {
                    tracing::warn!("Framebuffer read out of bounds: offset=0x{:X}, size={}", offset, size);
                    Ok(0)
                }
            }
            4 => {
                if (offset as usize) + 3 < buffer_lock.len() {
                    let bytes = [
                        buffer_lock[offset as usize],
                        buffer_lock[offset as usize + 1],
                        buffer_lock[offset as usize + 2],
                        buffer_lock[offset as usize + 3],
                    ];
                    Ok(u32::from_be_bytes(bytes))
                } else {
                    tracing::warn!("Framebuffer read out of bounds: offset=0x{:X}, size={}", offset, size);
                    Ok(0)
                }
            }
            _ => {
                tracing::warn!("Invalid framebuffer read size: {}", size);
                Ok(0)
            }
        }
    }
    
    fn write(&self, offset: u32, size: u8, value: u32) -> Result<()> {
        let fb = self.framebuffer.read();
        let buffer = fb.buffer();
        let mut buffer_lock = buffer.write();
        
        match size {
            1 => {
                if (offset as usize) < buffer_lock.len() {
                    buffer_lock[offset as usize] = value as u8;
                } else {
                    tracing::warn!("Framebuffer write out of bounds: offset=0x{:X}, size={}", offset, size);
                }
            }
            2 => {
                if (offset as usize) + 1 < buffer_lock.len() {
                    let bytes = (value as u16).to_be_bytes();
                    buffer_lock[offset as usize] = bytes[0];
                    buffer_lock[offset as usize + 1] = bytes[1];
                } else {
                    tracing::warn!("Framebuffer write out of bounds: offset=0x{:X}, size={}", offset, size);
                }
            }
            4 => {
                if (offset as usize) + 3 < buffer_lock.len() {
                    let bytes = value.to_be_bytes();
                    buffer_lock[offset as usize] = bytes[0];
                    buffer_lock[offset as usize + 1] = bytes[1];
                    buffer_lock[offset as usize + 2] = bytes[2];
                    buffer_lock[offset as usize + 3] = bytes[3];
                } else {
                    tracing::warn!("Framebuffer write out of bounds: offset=0x{:X}, size={}", offset, size);
                }
            }
            _ => {
                tracing::warn!("Invalid framebuffer write size: {}", size);
            }
        }
        
        Ok(())
    }
}
