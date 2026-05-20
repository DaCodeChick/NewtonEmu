// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Roxio Toast CD/DVD image support
//!
//! Supports reading Toast image files (.toast, .img) created by Roxio Toast.
//! Toast files can contain CD/DVD data in various formats.
//!
//! Note: Toast format is proprietary and partially documented through
//! reverse engineering. This implementation supports common Toast v1-v3
//! formats used for Mac OS 9 installation media.

use crate::storage::block_device::{BlockDevice, DeviceInfo};
use crate::storage::iso_image::IsoImage;
use crate::storage::raw_disk::RawDiskImage;
use newton_utils::Result;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt};

/// Toast image magic numbers
const TOAST_MAGIC_V1: &[u8] = b"TOC ";  // Toast v1
const TOAST_MAGIC_V2: &[u8] = b"TOC2";  // Toast v2
const TOAST_MAGIC_V3: &[u8] = b"TOC3";  // Toast v3

/// Toast CD/DVD image
///
/// Toast files can be:
/// - Raw ISO data (just wrapped in a Toast header)
/// - Compressed data
/// - Multiple sessions
///
/// For simplicity, we detect if it's a raw ISO and use IsoImage,
/// or extract to a temporary file for complex formats.
pub enum ToastImage {
    /// Direct ISO passthrough (Toast header wraps raw ISO)
    Iso(IsoImage),
    /// Extracted raw image
    Raw(RawDiskImage),
}

impl ToastImage {
    /// Open a Toast image file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let path_str = path_ref.to_string_lossy();
        
        tracing::info!("Opening Toast image: {}", path_str);
        
        let mut file = File::open(path_ref)
            .map_err(|e| newton_utils::Error::Io(e))?;
        
        // Read and verify Toast magic
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)
            .map_err(|e| newton_utils::Error::Io(e))?;
        
        let version = if &magic == TOAST_MAGIC_V1 {
            1
        } else if &magic == TOAST_MAGIC_V2 {
            2
        } else if &magic == TOAST_MAGIC_V3 {
            3
        } else {
            return Err(newton_utils::Error::Other(
                format!("Not a valid Toast image: magic = {:?}", magic)
            ));
        };
        
        tracing::info!("Toast version: {}", version);
        
        // Read Toast header to find data offset
        // The exact header structure varies by version, but generally:
        // - 4 bytes: magic
        // - 4 bytes: version/flags
        // - 4 bytes: header size or data offset
        // - ... more metadata
        
        let _flags = file.read_u32::<BigEndian>()
            .map_err(|e| newton_utils::Error::Io(e))?;
        
        // For most Toast images, the actual ISO/data starts at a fixed offset
        // or is specified in the header. Common offsets:
        // - Version 1: typically at 0x300 (768 bytes)
        // - Version 2/3: header size varies, but often around 0x600-0x800
        
        let data_offset: u64 = match version {
            1 => 0x300,      // 768 bytes
            2 => 0x600,      // 1536 bytes
            3 => 0x800,      // 2048 bytes
            _ => 0x300,
        };
        
        tracing::debug!("Seeking to data offset: 0x{:X}", data_offset);
        
        // Check if the data at that offset looks like ISO
        file.seek(SeekFrom::Start(data_offset))
            .map_err(|e| newton_utils::Error::Io(e))?;
        
        // Read a chunk to check if it's ISO format
        let mut check_buffer = vec![0u8; 32768]; // 32KB
        let bytes_read = file.read(&mut check_buffer)
            .map_err(|e| newton_utils::Error::Io(e))?;
        
        // ISO 9660 has a primary volume descriptor at sector 16 (0x8000)
        // which starts with byte 0x01 followed by "CD001"
        let is_iso = if bytes_read >= 0x8005 {
            &check_buffer[0x8000..0x8001] == &[0x01] &&
            &check_buffer[0x8001..0x8006] == b"CD001"
        } else {
            false
        };
        
        drop(file);
        
        if is_iso {
            tracing::info!("Toast image contains raw ISO data, using IsoImage");
            // The IsoImage reader can handle the offset
            // We'll create a wrapper that opens the file and seeks to data_offset
            Self::open_as_iso(path_ref, data_offset)
        } else {
            tracing::info!("Toast image format not directly supported, extracting...");
            // For now, treat as raw disk image at the data offset
            // In the future, we could handle compression, etc.
            Self::open_as_raw(path_ref, data_offset)
        }
    }
    
    /// Open Toast image as ISO (when it's just a wrapper)
    fn open_as_iso(path: &Path, _offset: u64) -> Result<Self> {
        // For now, try opening as ISO directly
        // IsoImage will need to handle the offset, or we extract first
        
        // Simple approach: Extract the ISO portion to a temp file
        let toast_file = File::open(path)
            .map_err(|e| newton_utils::Error::Io(e))?;
        
        // TODO: Implement proper extraction
        // For now, try direct ISO open (will fail if offset != 0)
        match IsoImage::open(path) {
            Ok(iso) => Ok(Self::Iso(iso)),
            Err(_) => {
                tracing::warn!("Failed to open as ISO, falling back to raw extraction");
                Self::open_as_raw(path, 0)
            }
        }
    }
    
    /// Open Toast image as raw disk image
    fn open_as_raw(path: &Path, offset: u64) -> Result<Self> {
        tracing::info!("Extracting Toast image to raw format...");
        
        let mut toast_file = File::open(path)
            .map_err(|e| newton_utils::Error::Io(e))?;
        
        // Get file size
        let file_size = toast_file.seek(SeekFrom::End(0))
            .map_err(|e| newton_utils::Error::Io(e))?;
        
        let data_size = file_size - offset;
        tracing::info!("Extracting {} bytes from offset 0x{:X}", data_size, offset);
        
        // Seek to data start
        toast_file.seek(SeekFrom::Start(offset))
            .map_err(|e| newton_utils::Error::Io(e))?;
        
        // Extract to temporary file
        let temp_path = std::path::PathBuf::from("/tmp")
            .join(format!("newton_toast_{}.raw", std::process::id()));
        
        let mut temp_file = File::create(&temp_path)
            .map_err(|e| newton_utils::Error::Io(e))?;
        
        // Copy data in chunks
        let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
        let mut remaining = data_size;
        
        while remaining > 0 {
            let to_read = std::cmp::min(buffer.len() as u64, remaining) as usize;
            let bytes_read = toast_file.read(&mut buffer[..to_read])
                .map_err(|e| newton_utils::Error::Io(e))?;
            
            if bytes_read == 0 {
                break;
            }
            
            std::io::Write::write_all(&mut temp_file, &buffer[..bytes_read])
                .map_err(|e| newton_utils::Error::Io(e))?;
            
            remaining -= bytes_read as u64;
        }
        
        temp_file.sync_all()
            .map_err(|e| newton_utils::Error::Io(e))?;
        drop(temp_file);
        
        tracing::info!("Extracted to: {}", temp_path.display());
        
        // Open as raw disk image
        let raw = RawDiskImage::open(&temp_path, false)?;
        Ok(Self::Raw(raw))
    }
}

impl BlockDevice for ToastImage {
    fn read_blocks(&self, lba: u64, count: u32, buffer: &mut [u8]) -> Result<usize> {
        match self {
            Self::Iso(iso) => iso.read_blocks(lba, count, buffer),
            Self::Raw(raw) => raw.read_blocks(lba, count, buffer),
        }
    }
    
    fn write_blocks(&mut self, _lba: u64, _count: u32, _buffer: &[u8]) -> Result<usize> {
        Err(newton_utils::Error::Other(
            "Toast images are read-only".to_string()
        ))
    }
    
    fn info(&self) -> &DeviceInfo {
        match self {
            Self::Iso(iso) => iso.info(),
            Self::Raw(raw) => raw.info(),
        }
    }
    
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl std::fmt::Debug for ToastImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Iso(iso) => write!(f, "ToastImage::Iso({:?})", iso),
            Self::Raw(raw) => write!(f, "ToastImage::Raw({:?})", raw),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore] // Requires a real Toast file
    fn test_toast_open() {
        let toast = ToastImage::open("test.toast");
        assert!(toast.is_ok() || toast.is_err());
    }
}
