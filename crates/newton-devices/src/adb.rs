// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Apple Desktop Bus (ADB) controller
//!
//! ADB is the input system used in Mac OS 8/9 era machines.
//! It connects keyboards, mice, and other input devices.

use newton_utils::Result;

/// ADB packet structure
#[derive(Debug, Clone)]
pub struct AdbPacket {
    pub address: u8,
    pub command: u8,
    pub data: Vec<u8>,
}

/// ADB device trait
pub trait AdbDevice: Send {
    /// Get device address (1-15)
    fn address(&self) -> u8;
    
    /// Poll device for data
    fn poll(&mut self) -> Option<AdbPacket>;
    
    /// Send command to device
    fn send_command(&mut self, cmd: u8, data: &[u8]) -> Result<()>;
}

/// ADB controller
pub struct AdbController {
    devices: Vec<Box<dyn AdbDevice>>,
}

impl AdbController {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }

    pub fn add_device(&mut self, device: Box<dyn AdbDevice>) {
        self.devices.push(device);
    }

    pub fn poll_devices(&mut self) -> Vec<AdbPacket> {
        self.devices.iter_mut()
            .filter_map(|dev| dev.poll())
            .collect()
    }
}

impl Default for AdbController {
    fn default() -> Self {
        Self::new()
    }
}

/// ADB Keyboard device
pub struct AdbKeyboard {
    address: u8,
    pending_keys: Vec<u8>,
}

impl AdbKeyboard {
    pub fn new(address: u8) -> Self {
        Self {
            address,
            pending_keys: Vec::new(),
        }
    }

    pub fn key_down(&mut self, keycode: u8) {
        self.pending_keys.push(keycode);
    }

    pub fn key_up(&mut self, keycode: u8) {
        self.pending_keys.push(keycode | 0x80);
    }
}

impl AdbDevice for AdbKeyboard {
    fn address(&self) -> u8 {
        self.address
    }

    fn poll(&mut self) -> Option<AdbPacket> {
        if self.pending_keys.is_empty() {
            None
        } else {
            Some(AdbPacket {
                address: self.address,
                command: 0,
                data: self.pending_keys.drain(..).collect(),
            })
        }
    }

    fn send_command(&mut self, _cmd: u8, _data: &[u8]) -> Result<()> {
        Ok(())
    }
}

/// ADB Mouse device
pub struct AdbMouse {
    address: u8,
    x: i16,
    y: i16,
    buttons: u8,
}

impl AdbMouse {
    pub fn new(address: u8) -> Self {
        Self {
            address,
            x: 0,
            y: 0,
            buttons: 0,
        }
    }

    pub fn move_to(&mut self, x: i16, y: i16) {
        self.x = x;
        self.y = y;
    }

    pub fn set_button(&mut self, button: u8, pressed: bool) {
        if pressed {
            self.buttons |= 1 << button;
        } else {
            self.buttons &= !(1 << button);
        }
    }
}

impl AdbDevice for AdbMouse {
    fn address(&self) -> u8 {
        self.address
    }

    fn poll(&mut self) -> Option<AdbPacket> {
        // Return current mouse state
        let data = vec![
            (self.buttons & 0x80) as u8,
            (self.x >> 8) as u8,
            (self.x & 0xFF) as u8,
            (self.y >> 8) as u8,
            (self.y & 0xFF) as u8,
        ];
        
        Some(AdbPacket {
            address: self.address,
            command: 0,
            data,
        })
    }

    fn send_command(&mut self, _cmd: u8, _data: &[u8]) -> Result<()> {
        Ok(())
    }
}
