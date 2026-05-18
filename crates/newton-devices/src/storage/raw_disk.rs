// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Raw disk image support

use super::block_device::{BlockDevice, DeviceInfo, DeviceType};
use newton_utils::{Error, Result};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Raw disk image
///
/// Simple flat file that can be read and written as a hard disk.
/// Supports both fixed-size and growable images.
pub struct RawDiskImage {
    /// Path to the disk image file
    path: PathBuf,
    /// File handle
    file: Arc<RwLock<File>>,
    /// Device information
    info: DeviceInfo,
    /// Whether writes are allowed
    writable: bool,
}

impl RawDiskImage {
    /// Standard hard disk sector size
    pub const SECTOR_SIZE: u32 = 512;
    
    /// Open an existing raw disk image
    pub fn open<P: AsRef<Path>>(path: P, writable: bool) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        let file = if writable {
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .map_err(|e| Error::Io(e))?
        } else {
            File::open(&path)
                .map_err(|e| Error::Io(e))?
        };
        
        let size = file.metadata()
            .map_err(|e| Error::Io(e))?
            .len();
        
        tracing::info!("Opened raw disk image: {} ({} bytes, {} sectors, {})", 
                      path.display(), size, size / Self::SECTOR_SIZE as u64,
                      if writable { "writable" } else { "read-only" });
        
        let file = Arc::new(RwLock::new(file));
        
        let info = DeviceInfo {
            device_type: DeviceType::HardDisk,
            model: format!("Virtual Hard Disk ({})", 
                          path.file_name().unwrap_or_default().to_string_lossy()),
            serial: "VHD000001".to_string(),
            firmware: "1.0".to_string(),
            size,
            block_size: Self::SECTOR_SIZE,
            read_only: !writable,
            removable: false,
            media_present: true,
        };
        
        Ok(Self {
            path,
            file,
            info,
            writable,
        })
    }
    
    /// Create a new raw disk image
    pub fn create<P: AsRef<Path>>(path: P, size_mb: u64) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let size_bytes = size_mb * 1024 * 1024;
        
        tracing::info!("Creating raw disk image: {} ({} MB)", path.display(), size_mb);
        
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .map_err(|e| Error::Io(e))?;
        
        // Set file size
        file.set_len(size_bytes)
            .map_err(|e| Error::Io(e))?;
        
        let file = Arc::new(RwLock::new(file));
        
        let info = DeviceInfo {
            device_type: DeviceType::HardDisk,
            model: format!("Virtual Hard Disk ({})", 
                          path.file_name().unwrap_or_default().to_string_lossy()),
            serial: "VHD000001".to_string(),
            firmware: "1.0".to_string(),
            size: size_bytes,
            block_size: Self::SECTOR_SIZE,
            read_only: false,
            removable: false,
            media_present: true,
        };
        
        Ok(Self {
            path,
            file,
            info,
            writable: true,
        })
    }
    
    /// Get the path to the disk image
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl BlockDevice for RawDiskImage {
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
        
        tracing::trace!("Disk read: LBA {} count {} = {} bytes", lba, count, bytes_to_read);
        
        Ok(bytes_to_read)
    }
    
    fn write_blocks(&mut self, lba: u64, count: u32, buffer: &[u8]) -> Result<usize> {
        if !self.writable {
            return Err(Error::Other("Disk is read-only".to_string()));
        }
        
        let block_size = self.info.block_size as u64;
        let required_size = (count as usize) * (block_size as usize);
        
        if buffer.len() < required_size {
            return Err(Error::Other(format!(
                "Buffer too small: need {}, got {}", 
                required_size, buffer.len()
            )));
        }
        
        let offset = lba * block_size;
        
        // Check if write is within bounds
        if offset + (count as u64 * block_size) > self.info.size {
            return Err(Error::Other(format!(
                "Write beyond end of device: offset={}, size={}", 
                offset, self.info.size
            )));
        }
        
        let mut file = self.file.write()
            .map_err(|_| Error::Other("Failed to lock file".to_string()))?;
        
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| Error::Io(e))?;
        
        let bytes_to_write = required_size;
        file.write_all(&buffer[..bytes_to_write])
            .map_err(|e| Error::Io(e))?;
        
        tracing::trace!("Disk write: LBA {} count {} = {} bytes", lba, count, bytes_to_write);
        
        Ok(bytes_to_write)
    }
    
    fn flush(&mut self) -> Result<()> {
        if self.writable {
            let mut file = self.file.write()
                .map_err(|_| Error::Other("Failed to lock file".to_string()))?;
            
            file.flush()
                .map_err(|e| Error::Io(e))?;
            
            tracing::debug!("Flushed disk: {}", self.path.display());
        }
        Ok(())
    }
}
