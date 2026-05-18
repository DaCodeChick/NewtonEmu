// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Memory management system

use crate::rom::Rom;
use newton_devices::MmioDevice;
use newton_utils::{Error, Result};
use byteorder::{BigEndian, ByteOrder};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

// Re-export the memory interface trait
pub use newton_cpu::MemoryInterface;

/// Memory address space
///
/// Thread-safe memory system using interior mutability.
/// ROM is immutable and requires no locking.
/// RAM uses RwLock for concurrent reads and exclusive writes.
pub struct Memory {
    /// System RAM (thread-safe with interior mutability)
    ram: Arc<RwLock<Vec<u8>>>,
    
    /// Boot ROM (immutable, no lock needed)
    rom: Option<Rom>,
    
    /// Memory-mapped I/O devices
    mmio_devices: HashMap<u32, Arc<RwLock<Box<dyn MmioDevice>>>>,
}

impl Memory {
    /// Create new memory with specified RAM size (in bytes)
    pub fn new(ram_size: usize) -> Self {
        tracing::info!("Initializing {} MB of RAM", ram_size / (1024 * 1024));
        Self {
            ram: Arc::new(RwLock::new(vec![0; ram_size])),
            rom: None,
            mmio_devices: HashMap::new(),
        }
    }

    /// Load ROM into memory
    pub fn load_rom(&mut self, rom: Rom) {
        tracing::info!("Loading ROM at 0x{:08X}, size {} bytes", rom.base_address(), rom.size());
        self.rom = Some(rom);
    }

    /// Register a memory-mapped I/O device
    pub fn register_mmio(&mut self, base_address: u32, device: Box<dyn MmioDevice>) {
        tracing::info!("Registering MMIO device '{}' at 0x{:08X}", device.name(), base_address);
        self.mmio_devices.insert(base_address, Arc::new(RwLock::new(device)));
    }

    /// Get RAM size
    pub fn ram_size(&self) -> usize {
        self.ram.read().len()
    }
}

// Implement MemoryInterface for CPU access
impl MemoryInterface for Memory {
    /// Read a byte from memory
    fn read_u8(&self, addr: u32) -> Result<u8> {
        // Check ROM range first (typically 0xFFF00000-0xFFFFFFFF)
        // ROM is immutable, so no lock needed
        if let Some(rom) = &self.rom {
            if addr >= rom.base_address() {
                let offset = (addr - rom.base_address()) as usize;
                if offset < rom.size() {
                    return Ok(rom.read_u8(offset));
                }
            }
        }

        // Check MMIO devices
        for (&base, device) in &self.mmio_devices {
            if addr >= base && addr < base.wrapping_add(0x10000) {
                let offset = addr - base;
                return device.read().read(offset, 1).map(|v| v as u8);
            }
        }

        // RAM access - use read lock for shared access
        let ram = self.ram.read();
        if (addr as usize) < ram.len() {
            Ok(ram[addr as usize])
        } else {
            Err(Error::Memory(format!("Invalid read at 0x{:08X}", addr)))
        }
    }

    /// Read a 16-bit word from memory (big-endian)
    fn read_u16(&self, addr: u32) -> Result<u16> {
        let b0 = self.read_u8(addr)?;
        let b1 = self.read_u8(addr + 1)?;
        Ok(BigEndian::read_u16(&[b0, b1]))
    }

    /// Read a 32-bit word from memory (big-endian)
    fn read_u32(&self, addr: u32) -> Result<u32> {
        // Check ROM (immutable, no lock)
        if let Some(rom) = &self.rom {
            if addr >= rom.base_address() {
                let offset = (addr - rom.base_address()) as usize;
                if offset + 4 <= rom.size() {
                    return Ok(rom.read_u32(offset));
                }
            }
        }

        // Check MMIO
        for (&base, device) in &self.mmio_devices {
            if addr >= base && addr < base.wrapping_add(0x10000) {
                let offset = addr - base;
                return device.read().read(offset, 4);
            }
        }

        // RAM - use read lock for shared access
        let ram = self.ram.read();
        if (addr as usize) + 4 <= ram.len() {
            Ok(BigEndian::read_u32(&ram[addr as usize..]))
        } else {
            Err(Error::Memory(format!("Invalid read at 0x{:08X}", addr)))
        }
    }

    /// Write a byte to memory
    fn write_u8(&self, addr: u32, value: u8) -> Result<()> {
        // Check MMIO devices
        for (&base, device) in &self.mmio_devices {
            if addr >= base && addr < base.wrapping_add(0x10000) {
                let offset = addr - base;
                return device.write().write(offset, 1, value as u32);
            }
        }

        // RAM access (ROM is read-only) - use write lock for exclusive access
        let mut ram = self.ram.write();
        if (addr as usize) < ram.len() {
            ram[addr as usize] = value;
            Ok(())
        } else {
            Err(Error::Memory(format!("Invalid write at 0x{:08X}", addr)))
        }
    }

    /// Write a 16-bit word to memory (big-endian)
    fn write_u16(&self, addr: u32, value: u16) -> Result<()> {
        let bytes = value.to_be_bytes();
        self.write_u8(addr, bytes[0])?;
        self.write_u8(addr + 1, bytes[1])?;
        Ok(())
    }

    /// Write a 32-bit word to memory (big-endian)
    fn write_u32(&self, addr: u32, value: u32) -> Result<()> {
        // Check MMIO
        for (&base, device) in &self.mmio_devices {
            if addr >= base && addr < base.wrapping_add(0x10000) {
                let offset = addr - base;
                return device.write().write(offset, 4, value);
            }
        }

        // RAM - use write lock for exclusive access
        let mut ram = self.ram.write();
        if (addr as usize) + 4 <= ram.len() {
            BigEndian::write_u32(&mut ram[addr as usize..], value);
            Ok(())
        } else {
            Err(Error::Memory(format!("Invalid write at 0x{:08X}", addr)))
        }
    }
}
