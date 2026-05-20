// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! SCSI controller MMIO device
//!
//! Emulates a MESH (Macintosh Enhanced SCSI Hardware) controller
//! used in PowerPC Macs

use crate::MmioDevice;
use super::{ScsiDevice, StorageBus};
use newton_utils::Result;
use parking_lot::RwLock;
use std::sync::Arc;

/// MESH SCSI controller registers
const MESH_REG_COUNT: u32 = 0x10;
const MESH_REG_TRANSFER_COUNT: u32 = 0x00;
const MESH_REG_FIFO: u32 = 0x01;
const MESH_REG_SEQUENCE: u32 = 0x02;
const MESH_REG_BUS_STATUS0: u32 = 0x03;
const MESH_REG_BUS_STATUS1: u32 = 0x04;
const MESH_REG_FIFO_COUNT: u32 = 0x05;
const MESH_REG_EXCEPTION: u32 = 0x06;
const MESH_REG_ERROR: u32 = 0x07;
const MESH_REG_INTERRUPT_MASK: u32 = 0x08;
const MESH_REG_INTERRUPT: u32 = 0x09;
const MESH_REG_SOURCE_ID: u32 = 0x0A;
const MESH_REG_DEST_ID: u32 = 0x0B;
const MESH_REG_SYNC_PARAMS: u32 = 0x0C;
const MESH_REG_MESH_ID: u32 = 0x0D;

/// MESH SCSI controller MMIO device
pub struct MeshController {
    /// Internal state (using RwLock for interior mutability from &self methods)
    state: RwLock<MeshState>,
    
    /// Reference to the storage bus (for accessing SCSI devices)
    storage_bus: Option<Arc<RwLock<StorageBus>>>,
}

/// Internal mutable state
struct MeshState {
    /// Register file
    registers: [u8; MESH_REG_COUNT as usize],
    
    /// FIFO buffer (16 bytes)
    fifo: Vec<u8>,
    
    /// Currently selected target ID
    target_id: u8,
    
    /// Data buffer for transfers
    data_buffer: Vec<u8>,
}

impl MeshController {
    /// Create a new MESH controller
    pub fn new() -> Self {
        let mut registers = [0u8; MESH_REG_COUNT as usize];
        
        // Set MESH ID register (chip identification)
        registers[MESH_REG_MESH_ID as usize] = 0xE2; // MESH II
        
        Self {
            state: RwLock::new(MeshState {
                registers,
                fifo: Vec::new(),
                target_id: 0,
                data_buffer: Vec::new(),
            }),
            storage_bus: None,
        }
    }
    
    /// Set the storage bus reference
    /// This allows the MESH controller to access SCSI devices
    pub fn set_storage_bus(&mut self, storage_bus: Arc<RwLock<StorageBus>>) {
        self.storage_bus = Some(storage_bus);
        tracing::info!("MESH: Connected to storage bus");
    }
}

impl MeshState {
    /// Handle a register write
    fn write_register(&mut self, offset: u32, value: u8, storage_bus: &Option<Arc<RwLock<StorageBus>>>) {
        tracing::trace!("MESH: write register 0x{:02X} <- 0x{:02X}", offset, value);
        
        match offset {
            MESH_REG_FIFO => {
                // Write to FIFO
                if self.fifo.len() < 16 {
                    self.fifo.push(value);
                } else {
                    tracing::warn!("MESH: FIFO overflow");
                }
            }
            MESH_REG_SEQUENCE => {
                // Sequence register - triggers commands
                self.handle_sequence_command(value, storage_bus);
            }
            MESH_REG_DEST_ID => {
                self.target_id = value & 0x07; // Only 3 bits for SCSI ID
                tracing::debug!("MESH: Target ID set to {}", self.target_id);
            }
            MESH_REG_INTERRUPT => {
                // Clear interrupt by writing
                self.registers[offset as usize] = 0;
            }
            _ => {
                // Generic register write
                self.registers[offset as usize] = value;
            }
        }
    }
    
    /// Handle a register read
    fn read_register(&self, offset: u32) -> u8 {
        let value = match offset {
            MESH_REG_FIFO => {
                // Would need to pop from FIFO, but read is immutable
                // In real hardware, reading FIFO has side effects
                tracing::warn!("MESH: FIFO read without proper handling");
                0
            }
            MESH_REG_FIFO_COUNT => {
                self.fifo.len().min(0xFF) as u8
            }
            _ => {
                self.registers[offset as usize]
            }
        };
        
        tracing::trace!("MESH: read register 0x{:02X} -> 0x{:02X}", offset, value);
        value
    }
    
    /// Handle sequence command
    fn handle_sequence_command(&mut self, cmd: u8, storage_bus: &Option<Arc<RwLock<StorageBus>>>) {
        tracing::debug!("MESH: Sequence command 0x{:02X}", cmd);
        
        // Simplified command handling
        match cmd {
            0x01 => { // Arbitrate
                tracing::debug!("MESH: Arbitrate");
                // Set bus status to indicate arbitration won
                self.registers[MESH_REG_BUS_STATUS0 as usize] = 0x01;
            }
            0x02 => { // Select
                tracing::debug!("MESH: Select target {}", self.target_id);
                // Check if device exists via storage bus
                let device_exists = if let Some(bus) = storage_bus {
                    bus.read().scsi_device(self.target_id).is_some()
                } else {
                    false
                };
                
                if device_exists {
                    self.registers[MESH_REG_BUS_STATUS0 as usize] = 0x02;
                } else {
                    tracing::warn!("MESH: No device at ID {}", self.target_id);
                    self.registers[MESH_REG_EXCEPTION as usize] = 0x01; // Selection timeout
                }
            }
            0x03 => { // Command
                tracing::debug!("MESH: Send command");
                // Parse command from FIFO
                self.execute_scsi_command(storage_bus);
            }
            0x04 => { // Status
                tracing::debug!("MESH: Get status");
                // Return status byte in FIFO
                self.fifo.clear();
                self.fifo.push(0x00); // GOOD status
            }
            0x05 => { // Data in
                tracing::debug!("MESH: Data in (read from device)");
                // Transfer data from device to FIFO
                if !self.data_buffer.is_empty() {
                    let chunk_size = self.data_buffer.len().min(16);
                    self.fifo = self.data_buffer.drain(..chunk_size).collect();
                }
            }
            0x06 => { // Data out
                tracing::debug!("MESH: Data out (write to device)");
                // Transfer data from FIFO to device buffer
                self.data_buffer.extend_from_slice(&self.fifo);
                self.fifo.clear();
            }
            0x10 => { // Message in
                tracing::debug!("MESH: Message in");
                self.fifo.clear();
                self.fifo.push(0x00); // COMMAND COMPLETE message
            }
            _ => {
                tracing::warn!("MESH: Unknown sequence command 0x{:02X}", cmd);
            }
        }
        
        // Set interrupt to signal completion
        self.registers[MESH_REG_INTERRUPT as usize] = 0x01;
    }
    
    /// Execute SCSI command from FIFO
    fn execute_scsi_command(&mut self, storage_bus: &Option<Arc<RwLock<StorageBus>>>) {
        if self.fifo.is_empty() {
            tracing::warn!("MESH: No command in FIFO");
            return;
        }
        
        let opcode = self.fifo[0];
        tracing::debug!("MESH: Executing SCSI command 0x{:02X}", opcode);
        
        // Parse common SCSI commands
        match opcode {
            0x00 => { // TEST UNIT READY
                tracing::debug!("MESH: TEST UNIT READY");
                // Just succeed
                self.data_buffer.clear();
            }
            0x12 => { // INQUIRY
                tracing::debug!("MESH: INQUIRY");
                self.execute_inquiry(storage_bus);
            }
            0x25 => { // READ CAPACITY
                tracing::debug!("MESH: READ CAPACITY");
                self.execute_read_capacity(storage_bus);
            }
            0x28 => { // READ(10)
                tracing::debug!("MESH: READ(10)");
                self.execute_read10(storage_bus);
            }
            0x2A => { // WRITE(10)
                tracing::debug!("MESH: WRITE(10)");
                self.execute_write10();
            }
            _ => {
                tracing::warn!("MESH: Unsupported SCSI command 0x{:02X}", opcode);
                self.data_buffer.clear();
            }
        }
        
        self.fifo.clear();
    }
    
    /// Execute INQUIRY command
    fn execute_inquiry(&mut self, storage_bus: &Option<Arc<RwLock<StorageBus>>>) {
        // Get device model name from storage bus if available
        let device_model = if let Some(bus) = storage_bus {
            bus.read().scsi_device(self.target_id).map(|dev| {
                let block_device = dev.block_device();
                let bd = block_device.read();
                bd.info().model.clone()
            })
        } else {
            None
        };
        
        // Standard INQUIRY response (36 bytes minimum)
        let mut response = vec![
            0x05, // Peripheral device type: CD-ROM
            0x80, // Removable
            0x00, // ANSI version
            0x02, // Response data format
            0x1F, // Additional length (31 more bytes)
            0x00, 0x00, 0x00, // Reserved
        ];
        
        // Vendor ID (8 bytes): "APPLE   "
        response.extend_from_slice(b"APPLE   ");
        
        // Product ID (16 bytes): Use device model if available
        if let Some(model) = device_model {
            let formatted = format!("{:<16}", model);
            response.extend_from_slice(formatted.as_bytes().get(..16).unwrap_or(b"Virtual Device  "));
        } else {
            response.extend_from_slice(b"Virtual Device  ");
        }
        
        // Revision (4 bytes): "1.0 "
        response.extend_from_slice(b"1.0 ");
        
        self.data_buffer = response;
    }
    
    /// Execute READ CAPACITY command
    fn execute_read_capacity(&mut self, storage_bus: &Option<Arc<RwLock<StorageBus>>>) {
        if let Some(bus) = storage_bus {
            if let Some(device) = bus.read().scsi_device(self.target_id) {
                let block_device = device.block_device();
                let bd = block_device.read();
                let info = bd.info();
                let num_blocks = (info.size / info.block_size as u64) as u32;
                let block_size = info.block_size;
                
                // Return last LBA (num_blocks - 1) and block size
                let mut response = Vec::new();
                response.extend_from_slice(&(num_blocks - 1).to_be_bytes());
                response.extend_from_slice(&block_size.to_be_bytes());
                
                self.data_buffer = response;
                tracing::debug!("MESH: READ CAPACITY -> {} blocks of {} bytes", num_blocks, block_size);
            }
        }
    }
    
    /// Execute READ(10) command
    fn execute_read10(&mut self, storage_bus: &Option<Arc<RwLock<StorageBus>>>) {
        if self.fifo.len() < 10 {
            tracing::warn!("MESH: READ(10) incomplete command");
            return;
        }
        
        // Parse LBA and transfer length from command
        let lba = u32::from_be_bytes([self.fifo[2], self.fifo[3], self.fifo[4], self.fifo[5]]);
        let transfer_length = u16::from_be_bytes([self.fifo[7], self.fifo[8]]) as u32;
        
        tracing::debug!("MESH: READ(10) LBA={} length={}", lba, transfer_length);
        
        if let Some(bus) = storage_bus {
            if let Some(device) = bus.read().scsi_device(self.target_id) {
                let block_device = device.block_device();
                let bd = block_device.read();
                let block_size = bd.info().block_size;
                let total_bytes = (transfer_length * block_size) as usize;
                
                // Allocate buffer
                self.data_buffer = vec![0u8; total_bytes];
                
                // Read blocks
                if let Err(e) = bd.read_blocks(lba as u64, transfer_length, &mut self.data_buffer) {
                    tracing::error!("MESH: READ(10) failed: {}", e);
                    self.data_buffer.clear();
                }
            }
        }
    }
    
    /// Execute WRITE(10) command
    fn execute_write10(&mut self) {
        if self.fifo.len() < 10 {
            tracing::warn!("MESH: WRITE(10) incomplete command");
            return;
        }
        
        // Parse LBA and transfer length from command
        let lba = u32::from_be_bytes([self.fifo[2], self.fifo[3], self.fifo[4], self.fifo[5]]);
        let transfer_length = u16::from_be_bytes([self.fifo[7], self.fifo[8]]) as u32;
        
        tracing::debug!("MESH: WRITE(10) LBA={} length={}", lba, transfer_length);
        
        // Data will be written in subsequent data out phases
        // For now, just acknowledge the command
    }
}

impl MmioDevice for MeshController {
    fn name(&self) -> &str {
        "MESH SCSI Controller"
    }
    
    fn read(&self, offset: u32, size: u8) -> Result<u32> {
        if size != 1 {
            tracing::warn!("MESH: Non-byte read (size={})", size);
        }
        
        if offset < MESH_REG_COUNT {
            // Special handling for FIFO reads (which have side effects)
            if offset == MESH_REG_FIFO {
                let mut state = self.state.write();
                if state.fifo.is_empty() {
                    Ok(0)
                } else {
                    Ok(state.fifo.remove(0) as u32)
                }
            } else {
                Ok(self.state.read().read_register(offset) as u32)
            }
        } else {
            tracing::warn!("MESH: Read from invalid offset 0x{:X}", offset);
            Ok(0)
        }
    }
    
    fn write(&self, offset: u32, size: u8, value: u32) -> Result<()> {
        if size != 1 {
            tracing::warn!("MESH: Non-byte write (size={})", size);
        }
        
        if offset < MESH_REG_COUNT {
            self.state.write().write_register(offset, value as u8, &self.storage_bus);
            Ok(())
        } else {
            tracing::warn!("MESH: Write to invalid offset 0x{:X}", offset);
            Ok(())
        }
    }
}
