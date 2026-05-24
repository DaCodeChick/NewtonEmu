// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Storage device emulation (IDE/SCSI)
//!
//! This module provides emulation for various storage devices:
//! - CD/DVD drives (ISO images via SCSI)
//! - Hard disks (raw disk images via IDE/SCSI)
//! - Future: SCSI tape, floppy drives

pub mod block_device;
pub mod iso_image;
pub mod raw_disk;
pub mod dmg_image;
pub mod toast_image;
pub mod zip_image;
pub mod scsi;
pub mod mesh_controller;
pub mod hfs;

pub use block_device::{BlockDevice, DeviceInfo, DeviceType};
pub use iso_image::IsoImage;
pub use raw_disk::RawDiskImage;
pub use dmg_image::DmgImage;
pub use toast_image::ToastImage;
pub use zip_image::ZipImage;
pub use scsi::{ScsiDevice, ScsiCommand, ScsiStatus};
pub use mesh_controller::MeshController;

use newton_utils::Result;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Storage bus manager
///
/// Manages all storage devices attached to the system
pub struct StorageBus {
    /// SCSI devices (ID 0-7)
    scsi_devices: HashMap<u8, ScsiDevice>,
    
    /// IDE devices (master/slave on primary/secondary channels)
    ide_devices: HashMap<(u8, u8), Arc<RwLock<dyn BlockDevice>>>,
}

impl StorageBus {
    /// Create a new storage bus
    pub fn new() -> Self {
        Self {
            scsi_devices: HashMap::new(),
            ide_devices: HashMap::new(),
        }
    }
    
    /// Attach a SCSI device
    pub fn attach_scsi(&mut self, id: u8, device: Arc<RwLock<dyn BlockDevice>>) -> Result<()> {
        if id > 7 {
            return Err(newton_utils::Error::Other(
                format!("Invalid SCSI ID: {} (must be 0-7)", id)
            ));
        }
        
        let scsi_device = ScsiDevice::new(id, device);
        
        if self.scsi_devices.insert(id, scsi_device).is_some() {
            tracing::warn!("Replaced existing SCSI device at ID {}", id);
        } else {
            tracing::info!("Attached SCSI device at ID {}", id);
        }
        
        Ok(())
    }
    
    /// Detach a SCSI device
    pub fn detach_scsi(&mut self, id: u8) -> Option<ScsiDevice> {
        let device = self.scsi_devices.remove(&id);
        if device.is_some() {
            tracing::info!("Detached SCSI device at ID {}", id);
        }
        device
    }
    
    /// Get a SCSI device
    pub fn scsi_device(&self, id: u8) -> Option<&ScsiDevice> {
        self.scsi_devices.get(&id)
    }
    
    /// Get a mutable SCSI device
    pub fn scsi_device_mut(&mut self, id: u8) -> Option<&mut ScsiDevice> {
        self.scsi_devices.get_mut(&id)
    }
    
    /// Attach an IDE device
    ///
    /// # Arguments
    /// * `channel` - IDE channel (0 = primary, 1 = secondary)
    /// * `device` - Device number (0 = master, 1 = slave)
    pub fn attach_ide(&mut self, channel: u8, device_num: u8, device: Arc<RwLock<dyn BlockDevice>>) -> Result<()> {
        if channel > 1 {
            return Err(newton_utils::Error::Other(
                format!("Invalid IDE channel: {} (must be 0-1)", channel)
            ));
        }
        if device_num > 1 {
            return Err(newton_utils::Error::Other(
                format!("Invalid IDE device: {} (must be 0-1)", device_num)
            ));
        }
        
        if self.ide_devices.insert((channel, device_num), device).is_some() {
            tracing::warn!("Replaced existing IDE device at channel {}, device {}", channel, device_num);
        } else {
            tracing::info!("Attached IDE device at channel {}, device {}", channel, device_num);
        }
        
        Ok(())
    }
    
    /// Detach an IDE device
    pub fn detach_ide(&mut self, channel: u8, device_num: u8) -> Option<Arc<RwLock<dyn BlockDevice>>> {
        let device = self.ide_devices.remove(&(channel, device_num));
        if device.is_some() {
            tracing::info!("Detached IDE device at channel {}, device {}", channel, device_num);
        }
        device
    }
    
    /// Get an IDE device
    pub fn ide_device(&self, channel: u8, device_num: u8) -> Option<&Arc<RwLock<dyn BlockDevice>>> {
        self.ide_devices.get(&(channel, device_num))
    }
    
    /// List all attached devices
    pub fn list_devices(&self) {
        tracing::info!("=== Storage Devices ===");
        
        if self.scsi_devices.is_empty() && self.ide_devices.is_empty() {
            tracing::info!("  No devices attached");
            return;
        }
        
        if !self.scsi_devices.is_empty() {
            tracing::info!("SCSI Devices:");
            for (id, _device) in &self.scsi_devices {
                tracing::info!("  ID {}: (device info not accessible here)", id);
            }
        }
        
        if !self.ide_devices.is_empty() {
            tracing::info!("IDE Devices:");
            for ((channel, device_num), device) in &self.ide_devices {
                let dev = device.read();
                let name = dev.info().model.clone();
                tracing::info!("  Channel {}, Device {}: {}", channel, device_num, name);
            }
        }
    }
}

impl Default for StorageBus {
    fn default() -> Self {
        Self::new()
    }
}
