// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Block device trait and common types

use newton_utils::Result;
use std::fmt;

/// Block device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// Hard disk drive
    HardDisk,
    /// CD-ROM drive
    CdRom,
    /// DVD-ROM drive
    DvdRom,
    /// Floppy disk
    Floppy,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::HardDisk => write!(f, "Hard Disk"),
            DeviceType::CdRom => write!(f, "CD-ROM"),
            DeviceType::DvdRom => write!(f, "DVD-ROM"),
            DeviceType::Floppy => write!(f, "Floppy"),
        }
    }
}

/// Block device capabilities
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Device type
    pub device_type: DeviceType,
    /// Device model name
    pub model: String,
    /// Device serial number
    pub serial: String,
    /// Firmware version
    pub firmware: String,
    /// Total size in bytes
    pub size: u64,
    /// Block size in bytes (typically 512 or 2048)
    pub block_size: u32,
    /// Whether the device is read-only
    pub read_only: bool,
    /// Whether the device is removable
    pub removable: bool,
    /// Whether media is currently present
    pub media_present: bool,
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            device_type: DeviceType::HardDisk,
            model: "Generic Device".to_string(),
            serial: "0000000000".to_string(),
            firmware: "1.0".to_string(),
            size: 0,
            block_size: 512,
            read_only: false,
            removable: false,
            media_present: true,
        }
    }
}

/// Block device trait
///
/// Abstraction for all storage devices (hard disks, CD/DVD, floppies)
pub trait BlockDevice: Send + Sync {
    /// Get device information
    fn info(&self) -> &DeviceInfo;
    
    /// Read blocks from the device
    ///
    /// # Arguments
    /// * `lba` - Logical block address (starting block number)
    /// * `count` - Number of blocks to read
    /// * `buffer` - Buffer to read into (must be at least count * block_size)
    ///
    /// # Returns
    /// Number of bytes actually read
    fn read_blocks(&self, lba: u64, count: u32, buffer: &mut [u8]) -> Result<usize>;
    
    /// Write blocks to the device
    ///
    /// # Arguments
    /// * `lba` - Logical block address (starting block number)
    /// * `count` - Number of blocks to write
    /// * `buffer` - Buffer to write from
    ///
    /// # Returns
    /// Number of bytes actually written
    fn write_blocks(&mut self, lba: u64, count: u32, buffer: &[u8]) -> Result<usize>;
    
    /// Flush any cached writes to persistent storage
    fn flush(&mut self) -> Result<()>;
    
    /// Check if media is present (for removable devices)
    fn media_present(&self) -> bool {
        self.info().media_present
    }
    
    /// Eject media (for removable devices)
    fn eject(&mut self) -> Result<()> {
        Ok(())
    }
}
