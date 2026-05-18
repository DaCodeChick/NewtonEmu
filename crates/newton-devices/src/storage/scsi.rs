// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! SCSI controller emulation

use super::block_device::BlockDevice;
use newton_utils::{Error, Result};
use std::sync::{Arc, RwLock};

/// SCSI command opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ScsiCommand {
    TestUnitReady = 0x00,
    RequestSense = 0x03,
    Inquiry = 0x12,
    ReadCapacity = 0x25,
    Read10 = 0x28,
    Write10 = 0x2A,
    ReadToc = 0x43,
}

impl ScsiCommand {
    pub fn from_u8(opcode: u8) -> Option<Self> {
        match opcode {
            0x00 => Some(ScsiCommand::TestUnitReady),
            0x03 => Some(ScsiCommand::RequestSense),
            0x12 => Some(ScsiCommand::Inquiry),
            0x25 => Some(ScsiCommand::ReadCapacity),
            0x28 => Some(ScsiCommand::Read10),
            0x2A => Some(ScsiCommand::Write10),
            0x43 => Some(ScsiCommand::ReadToc),
            _ => None,
        }
    }
}

/// SCSI device status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ScsiStatus {
    Good = 0x00,
    CheckCondition = 0x02,
    ConditionMet = 0x04,
    Busy = 0x08,
    Intermediate = 0x10,
    IntermediateConditionMet = 0x14,
    ReservationConflict = 0x18,
    CommandTerminated = 0x22,
    QueueFull = 0x28,
}

/// SCSI sense key
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SenseKey {
    NoSense = 0x00,
    RecoveredError = 0x01,
    NotReady = 0x02,
    MediumError = 0x03,
    HardwareError = 0x04,
    IllegalRequest = 0x05,
    UnitAttention = 0x06,
    DataProtect = 0x07,
    BlankCheck = 0x08,
    VendorSpecific = 0x09,
    CopyAborted = 0x0A,
    AbortedCommand = 0x0B,
    Obsolete = 0x0C,
    VolumeOverflow = 0x0D,
    Miscompare = 0x0E,
}

/// SCSI device
pub struct ScsiDevice {
    /// SCSI ID (0-7)
    id: u8,
    /// Block device
    device: Arc<RwLock<dyn BlockDevice>>,
    /// Current sense data
    sense_key: SenseKey,
    sense_code: u8,
    sense_qualifier: u8,
}

impl ScsiDevice {
    /// Create a new SCSI device
    pub fn new(id: u8, device: Arc<RwLock<dyn BlockDevice>>) -> Self {
        Self {
            id,
            device,
            sense_key: SenseKey::NoSense,
            sense_code: 0,
            sense_qualifier: 0,
        }
    }
    
    /// Get SCSI ID
    pub fn id(&self) -> u8 {
        self.id
    }
    
    /// Execute a SCSI command
    pub fn execute_command(&mut self, cdb: &[u8], data_in: &mut [u8]) -> Result<(ScsiStatus, usize)> {
        if cdb.is_empty() {
            return Err(Error::Other("Empty SCSI command".to_string()));
        }
        
        let opcode = cdb[0];
        let command = ScsiCommand::from_u8(opcode)
            .ok_or_else(|| Error::Other(format!("Unknown SCSI command: 0x{:02X}", opcode)))?;
        
        tracing::debug!("SCSI[{}]: {:?} (opcode 0x{:02X})", self.id, command, opcode);
        
        match command {
            ScsiCommand::TestUnitReady => self.cmd_test_unit_ready(),
            ScsiCommand::RequestSense => self.cmd_request_sense(data_in),
            ScsiCommand::Inquiry => self.cmd_inquiry(data_in),
            ScsiCommand::ReadCapacity => self.cmd_read_capacity(data_in),
            ScsiCommand::Read10 => self.cmd_read10(cdb, data_in),
            ScsiCommand::Write10 => self.cmd_write10(cdb, data_in),
            ScsiCommand::ReadToc => self.cmd_read_toc(cdb, data_in),
        }
    }
    
    fn cmd_test_unit_ready(&mut self) -> Result<(ScsiStatus, usize)> {
        let media_present = {
            let device = self.device.read()
                .map_err(|_| Error::Other("Failed to lock device".to_string()))?;
            device.media_present()
        };
        
        if media_present {
            Ok((ScsiStatus::Good, 0))
        } else {
            self.set_sense(SenseKey::NotReady, 0x3A, 0x00); // Medium not present
            Ok((ScsiStatus::CheckCondition, 0))
        }
    }
    
    fn cmd_request_sense(&mut self, data: &mut [u8]) -> Result<(ScsiStatus, usize)> {
        if data.len() < 18 {
            return Err(Error::Other("Buffer too small for REQUEST SENSE".to_string()));
        }
        
        // Build sense data
        data[0] = 0x70; // Response code: current errors
        data[1] = 0;
        data[2] = self.sense_key as u8;
        data[3..7].fill(0);
        data[7] = 10; // Additional sense length
        data[8..12].fill(0);
        data[12] = self.sense_code;
        data[13] = self.sense_qualifier;
        data[14..18].fill(0);
        
        // Clear sense after reading
        self.sense_key = SenseKey::NoSense;
        self.sense_code = 0;
        self.sense_qualifier = 0;
        
        Ok((ScsiStatus::Good, 18))
    }
    
    fn cmd_inquiry(&self, data: &mut [u8]) -> Result<(ScsiStatus, usize)> {
        if data.len() < 36 {
            return Err(Error::Other("Buffer too small for INQUIRY".to_string()));
        }
        
        let device = self.device.read()
            .map_err(|_| Error::Other("Failed to lock device".to_string()))?;
        
        let info = device.info();
        
        // Standard INQUIRY data
        data[0] = match info.device_type {
            super::block_device::DeviceType::HardDisk => 0x00, // Direct access
            super::block_device::DeviceType::CdRom | 
            super::block_device::DeviceType::DvdRom => 0x05, // CD-ROM
            super::block_device::DeviceType::Floppy => 0x00,
        };
        data[1] = if info.removable { 0x80 } else { 0x00 }; // Removable
        data[2] = 0x02; // SCSI-2
        data[3] = 0x02; // Response data format
        data[4] = 31; // Additional length
        data[5] = 0;
        data[6] = 0;
        data[7] = 0;
        
        // Vendor ID (8 bytes)
        let vendor = b"NewtonVM";
        data[8..16].copy_from_slice(vendor);
        
        // Product ID (16 bytes)
        let product = format!("{:<16}", info.model);
        data[16..32].copy_from_slice(&product.as_bytes()[..16]);
        
        // Revision (4 bytes)
        let revision = format!("{:<4}", info.firmware);
        data[32..36].copy_from_slice(&revision.as_bytes()[..4]);
        
        Ok((ScsiStatus::Good, 36))
    }
    
    fn cmd_read_capacity(&self, data: &mut [u8]) -> Result<(ScsiStatus, usize)> {
        if data.len() < 8 {
            return Err(Error::Other("Buffer too small for READ CAPACITY".to_string()));
        }
        
        let device = self.device.read()
            .map_err(|_| Error::Other("Failed to lock device".to_string()))?;
        
        let info = device.info();
        let total_blocks = (info.size / info.block_size as u64) - 1; // Last LBA
        
        // Last logical block address (4 bytes, big-endian)
        data[0..4].copy_from_slice(&(total_blocks as u32).to_be_bytes());
        
        // Block size in bytes (4 bytes, big-endian)
        data[4..8].copy_from_slice(&info.block_size.to_be_bytes());
        
        tracing::debug!("READ CAPACITY: {} blocks, {} bytes/block", total_blocks, info.block_size);
        
        Ok((ScsiStatus::Good, 8))
    }
    
    fn cmd_read10(&self, cdb: &[u8], data: &mut [u8]) -> Result<(ScsiStatus, usize)> {
        if cdb.len() < 10 {
            return Err(Error::Other("Invalid READ(10) CDB".to_string()));
        }
        
        // Parse CDB
        let lba = u32::from_be_bytes([cdb[2], cdb[3], cdb[4], cdb[5]]) as u64;
        let transfer_length = u16::from_be_bytes([cdb[7], cdb[8]]) as u32;
        
        if transfer_length == 0 {
            return Ok((ScsiStatus::Good, 0));
        }
        
        let device = self.device.read()
            .map_err(|_| Error::Other("Failed to lock device".to_string()))?;
        
        let bytes_read = device.read_blocks(lba, transfer_length, data)?;
        
        Ok((ScsiStatus::Good, bytes_read))
    }
    
    fn cmd_write10(&mut self, cdb: &[u8], data: &[u8]) -> Result<(ScsiStatus, usize)> {
        if cdb.len() < 10 {
            return Err(Error::Other("Invalid WRITE(10) CDB".to_string()));
        }
        
        // Parse CDB
        let lba = u32::from_be_bytes([cdb[2], cdb[3], cdb[4], cdb[5]]) as u64;
        let transfer_length = u16::from_be_bytes([cdb[7], cdb[8]]) as u32;
        
        if transfer_length == 0 {
            return Ok((ScsiStatus::Good, 0));
        }
        
        let mut device = self.device.write()
            .map_err(|_| Error::Other("Failed to lock device".to_string()))?;
        
        let bytes_written = device.write_blocks(lba, transfer_length, data)?;
        
        Ok((ScsiStatus::Good, bytes_written))
    }
    
    fn cmd_read_toc(&self, cdb: &[u8], data: &mut [u8]) -> Result<(ScsiStatus, usize)> {
        if cdb.len() < 10 {
            return Err(Error::Other("Invalid READ TOC CDB".to_string()));
        }
        
        // For now, return a minimal TOC for a single data track
        if data.len() < 12 {
            return Err(Error::Other("Buffer too small for READ TOC".to_string()));
        }
        
        // TOC data length (2 bytes)
        data[0..2].copy_from_slice(&10u16.to_be_bytes());
        data[2] = 1; // First track
        data[3] = 1; // Last track
        
        // Track descriptor for track 1
        data[4] = 0; // Reserved
        data[5] = 0x14; // ADR + Control (data track)
        data[6] = 1; // Track number
        data[7] = 0; // Reserved
        data[8..12].copy_from_slice(&0u32.to_be_bytes()); // Track start LBA
        
        Ok((ScsiStatus::Good, 12))
    }
    
    fn set_sense(&mut self, key: SenseKey, code: u8, qualifier: u8) {
        self.sense_key = key;
        self.sense_code = code;
        self.sense_qualifier = qualifier;
    }
}
