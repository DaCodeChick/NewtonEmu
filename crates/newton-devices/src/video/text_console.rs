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
        let fb = framebuffer.read();
        let cols = fb.width as usize / CHAR_WIDTH;
        let rows = fb.height as usize / CHAR_HEIGHT;
        drop(fb);

        Self {
            framebuffer,
            cursor_x: 0,
            cursor_y: 0,
            cols,
            rows,
            fg_color: 0xFFFFFFFF, // White
            bg_color: 0xFF000000, // Black
        }
    }

    /// Write a string to the console
    pub fn write_str(&mut self, s: &str) {
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
        let mut fb = self.framebuffer.write();
        let width = fb.width as usize;
        let height = fb.height as usize;
        
        // Copy each line up by CHAR_HEIGHT pixels
        for y in 0..(height - CHAR_HEIGHT) {
            for x in 0..width {
                let src_idx = (y + CHAR_HEIGHT) * width + x;
                let dst_idx = y * width + x;
                fb.data[dst_idx] = fb.data[src_idx];
            }
        }
        
        // Clear the last line
        for y in (height - CHAR_HEIGHT)..height {
            for x in 0..width {
                let idx = y * width + x;
                fb.data[idx] = self.bg_color;
            }
        }
    }

    /// Draw a character at the given column and row
    fn draw_char(&mut self, ch: char, col: usize, row: usize) {
        let char_index = (ch as usize).saturating_sub(32);
        if char_index >= 95 {
            return; // Out of range
        }

        let mut fb = self.framebuffer.write();
        let width = fb.width as usize;
        let x_start = col * CHAR_WIDTH;
        let y_start = row * CHAR_HEIGHT;

        // Each character is 8 bytes (8x8 pixels)
        let font_offset = char_index * 8;

        for y in 0..CHAR_HEIGHT {
            let byte = FONT_DATA[font_offset + y];
            for x in 0..CHAR_WIDTH {
                let pixel_set = (byte & (1 << (7 - x))) != 0;
                let color = if pixel_set { self.fg_color } else { self.bg_color };
                
                let screen_x = x_start + x;
                let screen_y = y_start + y;
                if screen_x < width && screen_y < (fb.height as usize) {
                    let idx = screen_y * width + screen_x;
                    fb.data[idx] = color;
                }
            }
        }
    }

    /// Clear the console
    pub fn clear(&mut self) {
        let mut fb = self.framebuffer.write();
        for pixel in fb.data.iter_mut() {
            *pixel = self.bg_color;
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }
}
