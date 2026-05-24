// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Simple text console for rendering OpenFirmware output to the framebuffer

use super::Framebuffer;
use parking_lot::RwLock;
use std::sync::Arc;

/// 8x8 monospace font data (ASCII 32-126)
/// Simple bitmap font where each character is 8x8 pixels
const FONT_DATA: &[u8] = include_bytes!("font8x8.bin");

const CHAR_WIDTH: usize = 8;
const CHAR_HEIGHT: usize = 8;
const SCALE_FACTOR: usize = 2; // 2x scaling for better readability

/// Text console that renders to a framebuffer
pub struct TextConsole {
    framebuffer: Arc<RwLock<Framebuffer>>,
    cursor_x: usize,
    cursor_y: usize,
    cols: usize,
    rows: usize,
    fg_color: u32, // ARGB8888
    bg_color: u32, // ARGB8888
}

impl TextConsole {
    /// Create a new text console
    pub fn new(framebuffer: Arc<RwLock<Framebuffer>>) -> Self {
        let (width, height) = {
            let fb = framebuffer.read();
            fb.dimensions()
        };
        
        let cols = width as usize / (CHAR_WIDTH * SCALE_FACTOR);
        let rows = height as usize / (CHAR_HEIGHT * SCALE_FACTOR);

        let mut console = Self {
            framebuffer,
            cursor_x: 0,
            cursor_y: 0,
            cols,
            rows,
            fg_color: 0xFFFFFFFF, // White
            bg_color: 0xFF000000, // Black
        };
        
        // Clear screen on initialization
        console.clear();
        tracing::info!("TextConsole initialized: {}x{} chars ({}x{} pixels)", cols, rows, width, height);
        
        console
    }

    /// Write a string to the console
    pub fn write_str(&mut self, s: &str) {
        tracing::info!("TextConsole::write_str: {:?} (len={})", s, s.len());
        for ch in s.chars() {
            self.write_char(ch);
        }
    }

    /// Write a single character
    pub fn write_char(&mut self, ch: char) {
        match ch {
            '\n' => self.newline(),
            '\r' => self.cursor_x = 0,
            '\t' => {
                let spaces = 4 - (self.cursor_x % 4);
                for _ in 0..spaces {
                    self.write_char(' ');
                }
            }
            _ if ch >= ' ' && ch <= '~' => {
                if self.cursor_x >= self.cols {
                    self.newline();
                }
                self.draw_char(ch, self.cursor_x, self.cursor_y);
                self.cursor_x += 1;
            }
            _ => {} // Ignore other control characters
        }
    }

    /// Move to the next line
    fn newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y >= self.rows {
            self.scroll();
            self.cursor_y = self.rows - 1;
        }
    }

    /// Scroll the display up by one line
    fn scroll(&mut self) {
        let fb = self.framebuffer.read();
        let (width, height) = fb.dimensions();
        let buffer = fb.buffer();
        let mut buf = buffer.write();
        
        let width = width as usize;
        let height = height as usize;
        let bpp = 4; // RGBA32
        let scaled_char_height = CHAR_HEIGHT * SCALE_FACTOR;
        
        // Copy each line up by scaled_char_height pixels
        for y in 0..(height - scaled_char_height) {
            for x in 0..width {
                let src_idx = ((y + scaled_char_height) * width + x) * bpp;
                let dst_idx = (y * width + x) * bpp;
                for i in 0..bpp {
                    buf[dst_idx + i] = buf[src_idx + i];
                }
            }
        }
        
        // Clear the last line
        for y in (height - scaled_char_height)..height {
            for x in 0..width {
                let idx = (y * width + x) * bpp;
                let color_bytes = self.bg_color.to_be_bytes();
                buf[idx] = color_bytes[1];     // R
                buf[idx + 1] = color_bytes[2]; // G
                buf[idx + 2] = color_bytes[3]; // B
                buf[idx + 3] = color_bytes[0]; // A
            }
        }
    }

    /// Draw a character at the given column and row
    fn draw_char(&mut self, ch: char, col: usize, row: usize) {
        let char_index = (ch as usize).saturating_sub(32);
        if char_index >= 95 {
            return; // Out of range
        }
        
        tracing::debug!("Drawing '{}' at ({}, {})", ch, col, row);

        let fb = self.framebuffer.read();
        let (width, height) = fb.dimensions();
        let buffer = fb.buffer();
        let mut buf = buffer.write();
        
        let width = width as usize;
        let height = height as usize;
        let x_start = col * CHAR_WIDTH * SCALE_FACTOR;
        let y_start = row * CHAR_HEIGHT * SCALE_FACTOR;
        let bpp = 4; // RGBA32

        // Each character is 8 bytes (8x8 pixels)
        let font_offset = char_index * 8;

        for y in 0..CHAR_HEIGHT {
            let byte = FONT_DATA[font_offset + y];
            for x in 0..CHAR_WIDTH {
                let pixel_set = (byte & (1 << (7 - x))) != 0;
                let color = if pixel_set { self.fg_color } else { self.bg_color };
                
                // Scale up each pixel by SCALE_FACTOR x SCALE_FACTOR
                for sy in 0..SCALE_FACTOR {
                    for sx in 0..SCALE_FACTOR {
                        let screen_x = x_start + x * SCALE_FACTOR + sx;
                        let screen_y = y_start + y * SCALE_FACTOR + sy;
                        if screen_x < width && screen_y < height {
                            let idx = (screen_y * width + screen_x) * bpp;
                            let color_bytes = color.to_be_bytes();
                            buf[idx] = color_bytes[1];     // R
                            buf[idx + 1] = color_bytes[2]; // G
                            buf[idx + 2] = color_bytes[3]; // B
                            buf[idx + 3] = color_bytes[0]; // A
                        }
                    }
                }
            }
        }
    }

    /// Clear the console
    pub fn clear(&mut self) {
        let fb = self.framebuffer.read();
        let buffer = fb.buffer();
        let mut buf = buffer.write();
        
        tracing::info!("Clearing console: buffer len={}, bg_color=0x{:08X}", buf.len(), self.bg_color);
        
        let color_bytes = self.bg_color.to_be_bytes();
        for i in (0..buf.len()).step_by(4) {
            buf[i] = color_bytes[1];     // R
            buf[i + 1] = color_bytes[2]; // G
            buf[i + 2] = color_bytes[3]; // B
            buf[i + 3] = color_bytes[0]; // A
        }
        
        // Verify first few pixels
        tracing::info!("First pixel after clear: [{}, {}, {}, {}]", buf[0], buf[1], buf[2], buf[3]);
        
        self.cursor_x = 0;
        self.cursor_y = 0;
    }
}
