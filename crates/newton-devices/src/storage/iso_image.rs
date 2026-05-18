// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! ISO9660 CD-ROM image support

use super::block_device::{BlockDevice, DeviceInfo, DeviceType};
use newton_utils::{Error, Result};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// ISO9660 CD-ROM image
pub struct IsoImage {
    /// Path to the ISO file
    path: PathBuf,
    /// File handle
    file: Arc<RwLock<File>>,
    /// Device information
    info: DeviceInfo,
}

impl IsoImage {
    /// Standard CD-ROM sector size
    pub const SECTOR_SIZE: u32 = 2048;
    
    /// Create a new ISO image from a file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)
            .map_err(|e| Error::Io(e))?;
        
        // Get file size
        let size = file.metadata()
            .map_err(|e| Error::Io(e))?
            .len();
        
        tracing::info!("Opened ISO image: {} ({} bytes, {} sectors)", 
                      path.display(), size, size / Self::SECTOR_SIZE as u64);
        
        let file = Arc::new(RwLock::new(file));
        
        // Read volume descriptor to get label
        let label = Self::read_volume_label(&file)?;
        
        let info = DeviceInfo {
            device_type: DeviceType::CdRom,
            model: format!("Virtual CD-ROM ({})", label),
            serial: "VCD000001".to_string(),
            firmware: "1.0".to_string(),
            size,
            block_size: Self::SECTOR_SIZE,
            read_only: true,
            removable: true,
            media_present: true,
        };
        
        Ok(Self {
            path,
            file,
            info,
        })
    }
    
    /// Read the volume label from the ISO9660 primary volume descriptor
    fn read_volume_label(file: &Arc<RwLock<File>>) -> Result<String> {
        let mut file = file.write()
            .map_err(|_| Error::Other("Failed to lock file".to_string()))?;
        
        // Primary Volume Descriptor is at sector 16
        file.seek(SeekFrom::Start(16 * 2048))
            .map_err(|e| Error::Io(e))?;
        
        let mut sector = vec![0u8; 2048];
        file.read_exact(&mut sector)
            .map_err(|e| Error::Io(e))?;
        
        // Check for CD001 signature at offset 1
        if &sector[1..6] != b"CD001" {
            return Ok("Unknown".to_string());
        }
        
        // Volume identifier is at offset 40, 32 bytes
        let volume_id = &sector[40..72];
        let volume_id = String::from_utf8_lossy(volume_id)
            .trim()
            .to_string();
        
        if volume_id.is_empty() {
            Ok("Untitled".to_string())
        } else {
            Ok(volume_id)
        }
    }
    
    /// Get the path to the ISO file
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl BlockDevice for IsoImage {
    fn info(&self) -> &DeviceInfo {
        &self.info
    }
    
    fn read_blocks(&self, lba: u64, count: u32, buffer: &mut [u8]) -> Result<usize> {
        let block_size = self.info.block_size as u64;
        let required_size = (count as usize) * (block_size as usize);
        
        if buffer.len() < required_size {
            return Err(Error::Other(format!(
                "Buffer too small: need {}, got {}", 
                required_size, buffer.len()
            )));
        }
        
        let offset = lba * block_size;
        
        // Check if read is within bounds
        if offset + (count as u64 * block_size) > self.info.size {
            return Err(Error::Other(format!(
                "Read beyond end of device: offset={}, size={}", 
                offset, self.info.size
            )));
        }
        
        let mut file = self.file.write()
            .map_err(|_| Error::Other("Failed to lock file".to_string()))?;
        
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| Error::Io(e))?;
        
        let bytes_to_read = required_size;
        file.read_exact(&mut buffer[..bytes_to_read])
            .map_err(|e| Error::Io(e))?;
        
        tracing::trace!("ISO read: LBA {} count {} = {} bytes", lba, count, bytes_to_read);
        
        Ok(bytes_to_read)
    }
    
    fn write_blocks(&mut self, _lba: u64, _count: u32, _buffer: &[u8]) -> Result<usize> {
        Err(Error::Other("ISO images are read-only".to_string()))
    }
    
    fn flush(&mut self) -> Result<()> {
        // Read-only, nothing to flush
        Ok(())
    }
    
    fn eject(&mut self) -> Result<()> {
        tracing::info!("Ejecting CD: {}", self.path.display());
        // In a real implementation, we'd close the file and mark media as not present
        // For now, just log it
        Ok(())
    }
}
