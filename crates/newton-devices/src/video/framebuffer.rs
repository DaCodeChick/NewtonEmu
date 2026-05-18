// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Framebuffer device for video output

use crate::MmioDevice;
use newton_utils::Result;
use parking_lot::RwLock;
use std::sync::Arc;

/// Color depth modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorDepth {
    /// 8-bit indexed color (256 colors)
    Indexed8,
    /// 16-bit RGB (5-6-5)
    Rgb16,
    /// 32-bit RGBA (8-8-8-8)
    Rgba32,
}

impl ColorDepth {
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            ColorDepth::Indexed8 => 1,
            ColorDepth::Rgb16 => 2,
            ColorDepth::Rgba32 => 4,
        }
    }
}

/// Framebuffer device
pub struct Framebuffer {
    width: u32,
    height: u32,
    depth: ColorDepth,
    buffer: Arc<RwLock<Vec<u8>>>,
    palette: [u32; 256],  // For indexed color mode
}

impl Framebuffer {
    /// Create a new framebuffer
    pub fn new(width: u32, height: u32, depth: ColorDepth) -> Self {
        let size = (width * height) as usize * depth.bytes_per_pixel();
        Self {
            width,
            height,
            depth,
            buffer: Arc::new(RwLock::new(vec![0; size])),
            palette: [0; 256],
        }
    }

    /// Get framebuffer dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get color depth
    pub fn color_depth(&self) -> ColorDepth {
        self.depth
    }

    /// Get shared buffer reference for rendering
    pub fn buffer(&self) -> Arc<RwLock<Vec<u8>>> {
        Arc::clone(&self.buffer)
    }

    /// Set palette entry (for indexed color mode)
    pub fn set_palette(&mut self, index: u8, color: u32) {
        self.palette[index as usize] = color;
    }

    /// Get palette entry
    pub fn get_palette(&self, index: u8) -> u32 {
        self.palette[index as usize]
    }

    /// Convert indexed buffer to RGBA for rendering
    pub fn to_rgba(&self) -> Vec<u8> {
        let buffer = self.buffer.read();
        match self.depth {
            ColorDepth::Indexed8 => {
                let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);
                for &idx in buffer.iter() {
                    let color = self.palette[idx as usize];
                    rgba.push(((color >> 16) & 0xFF) as u8);  // R
                    rgba.push(((color >> 8) & 0xFF) as u8);   // G
                    rgba.push((color & 0xFF) as u8);          // B
                    rgba.push(255);                            // A
                }
                rgba
            }
            ColorDepth::Rgb16 => {
                let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);
                for chunk in buffer.chunks(2) {
                    let pixel = u16::from_be_bytes([chunk[0], chunk[1]]);
                    let r = ((pixel >> 11) & 0x1F) as u8;
                    let g = ((pixel >> 5) & 0x3F) as u8;
                    let b = (pixel & 0x1F) as u8;
                    rgba.push((r << 3) | (r >> 2));
                    rgba.push((g << 2) | (g >> 4));
                    rgba.push((b << 3) | (b >> 2));
                    rgba.push(255);
                }
                rgba
            }
            ColorDepth::Rgba32 => buffer.clone(),
        }
    }
}

impl MmioDevice for Framebuffer {
    fn read(&self, offset: u32, size: u8) -> Result<u32> {
        let buffer = self.buffer.read();
        let offset = offset as usize;
        
        match size {
            1 => Ok(buffer.get(offset).copied().unwrap_or(0) as u32),
            2 => {
                let bytes = buffer.get(offset..offset + 2).unwrap_or(&[0, 0]);
                Ok(u16::from_be_bytes([bytes[0], bytes[1]]) as u32)
            }
            4 => {
                let bytes = buffer.get(offset..offset + 4).unwrap_or(&[0, 0, 0, 0]);
                Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            }
            _ => Ok(0),
        }
    }

    fn write(&self, offset: u32, size: u8, value: u32) -> Result<()> {
        let mut buffer = self.buffer.write();
        let offset = offset as usize;
        
        match size {
            1 => {
                if let Some(byte) = buffer.get_mut(offset) {
                    *byte = value as u8;
                }
            }
            2 => {
                let bytes = (value as u16).to_be_bytes();
                if let Some(slice) = buffer.get_mut(offset..offset + 2) {
                    slice.copy_from_slice(&bytes);
                }
            }
            4 => {
                let bytes = value.to_be_bytes();
                if let Some(slice) = buffer.get_mut(offset..offset + 4) {
                    slice.copy_from_slice(&bytes);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Framebuffer"
    }
}
