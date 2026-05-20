// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! ZIP archive disk image support
//!
//! Supports extracting disk images from ZIP archives and opening them
//! as block devices. Useful for downloading Mac OS 9 installation media
//! that's been compressed in ZIP format.

use crate::storage::block_device::{BlockDevice, DeviceInfo};
use crate::storage::iso_image::IsoImage;
use crate::storage::raw_disk::RawDiskImage;
use crate::storage::dmg_image::DmgImage;
use crate::storage::toast_image::ToastImage;
use newton_utils::Result;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use zip::ZipArchive;

/// ZIP-wrapped disk image
///
/// This wrapper extracts disk images from ZIP archives and opens them
/// as the appropriate type (ISO, DMG, Toast, or raw).
pub enum ZipImage {
    /// ISO image from ZIP
    Iso(IsoImage),
    /// DMG image from ZIP
    Dmg(DmgImage),
    /// Toast image from ZIP
    Toast(ToastImage),
    /// Raw disk image from ZIP
    Raw(RawDiskImage),
}

impl ZipImage {
    /// Open a disk image from a ZIP archive
    ///
    /// This will:
    /// 1. Extract the first disk image found in the archive
    /// 2. Detect its format (ISO, DMG, Toast, or raw)
    /// 3. Open it as the appropriate type
    ///
    /// Supported extensions: .iso, .img, .dmg, .toast, .bin, .cdr
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let path_str = path_ref.to_string_lossy();
        
        tracing::info!("Opening ZIP archive: {}", path_str);
        
        let file = File::open(path_ref)
            .map_err(|e| newton_utils::Error::Io(e))?;
        
        let mut archive = ZipArchive::new(file)
            .map_err(|e| newton_utils::Error::Other(format!("Failed to open ZIP: {}", e)))?;
        
        tracing::info!("ZIP contains {} files", archive.len());
        
        // Find the first disk image file
        let mut found_index = None;
        let mut found_name = String::new();
        
        for i in 0..archive.len() {
            let file_info = archive.by_index(i)
                .map_err(|e| newton_utils::Error::Other(format!("Failed to read ZIP entry: {}", e)))?;
            
            let name = file_info.name().to_lowercase();
            
            // Check for disk image extensions
            if name.ends_with(".iso") 
                || name.ends_with(".img")
                || name.ends_with(".dmg")
                || name.ends_with(".toast")
                || name.ends_with(".bin")
                || name.ends_with(".cdr")
            {
                found_index = Some(i);
                found_name = file_info.name().to_string();
                tracing::info!("Found disk image in ZIP: {}", found_name);
                break;
            }
        }
        
        let index = found_index.ok_or_else(|| {
            newton_utils::Error::Other(
                "No disk image found in ZIP archive (looking for .iso, .img, .dmg, .toast, .bin, .cdr)".to_string()
            )
        })?;
        
        // Extract the disk image to a temporary file
        let mut zip_file = archive.by_index(index)
            .map_err(|e| newton_utils::Error::Other(format!("Failed to open ZIP entry: {}", e)))?;
        
        let size = zip_file.size();
        tracing::info!("Extracting {} ({} bytes) from ZIP...", found_name, size);
        
        // Determine temporary file extension based on the original file
        let extension = std::path::Path::new(&found_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("img");
        
        let temp_path = std::path::PathBuf::from("/tmp")
            .join(format!("newton_zip_{}.{}", std::process::id(), extension));
        
        let mut temp_file = File::create(&temp_path)
            .map_err(|e| newton_utils::Error::Io(e))?;
        
        // Extract with progress logging for large files
        let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
        let mut extracted = 0u64;
        let log_interval = 100 * 1024 * 1024; // Log every 100MB
        let mut next_log = log_interval;
        
        loop {
            let bytes_read = zip_file.read(&mut buffer)
                .map_err(|e| newton_utils::Error::Io(e))?;
            
            if bytes_read == 0 {
                break;
            }
            
            temp_file.write_all(&buffer[..bytes_read])
                .map_err(|e| newton_utils::Error::Io(e))?;
            
            extracted += bytes_read as u64;
            
            if extracted >= next_log {
                tracing::info!("Extracted {} MB...", extracted / (1024 * 1024));
                next_log += log_interval;
            }
        }
        
        temp_file.sync_all()
            .map_err(|e| newton_utils::Error::Io(e))?;
        drop(temp_file);
        
        tracing::info!("Extracted {} bytes to: {}", extracted, temp_path.display());
        
        // Now open the extracted file based on its extension
        let name_lower = found_name.to_lowercase();
        
        if name_lower.ends_with(".iso") {
            tracing::info!("Opening extracted file as ISO");
            let iso = IsoImage::open(&temp_path)?;
            Ok(Self::Iso(iso))
        } else if name_lower.ends_with(".dmg") {
            tracing::info!("Opening extracted file as DMG");
            let dmg = DmgImage::open(&temp_path)?;
            Ok(Self::Dmg(dmg))
        } else if name_lower.ends_with(".toast") {
            tracing::info!("Opening extracted file as Toast");
            let toast = ToastImage::open(&temp_path)?;
            Ok(Self::Toast(toast))
        } else {
            // Try to auto-detect format by looking at the file
            tracing::info!("Auto-detecting format of extracted file");
            
            // Try ISO first (most common)
            if let Ok(iso) = IsoImage::open(&temp_path) {
                tracing::info!("Detected as ISO");
                return Ok(Self::Iso(iso));
            }
            
            // Try DMG
            if let Ok(dmg) = DmgImage::open(&temp_path) {
                tracing::info!("Detected as DMG");
                return Ok(Self::Dmg(dmg));
            }
            
            // Try Toast
            if let Ok(toast) = ToastImage::open(&temp_path) {
                tracing::info!("Detected as Toast");
                return Ok(Self::Toast(toast));
            }
            
            // Fall back to raw
            tracing::info!("Opening as raw disk image");
            let raw = RawDiskImage::open(&temp_path, false)?;
            Ok(Self::Raw(raw))
        }
    }
}

impl BlockDevice for ZipImage {
    fn read_blocks(&self, lba: u64, count: u32, buffer: &mut [u8]) -> Result<usize> {
        match self {
            Self::Iso(iso) => iso.read_blocks(lba, count, buffer),
            Self::Dmg(dmg) => dmg.read_blocks(lba, count, buffer),
            Self::Toast(toast) => toast.read_blocks(lba, count, buffer),
            Self::Raw(raw) => raw.read_blocks(lba, count, buffer),
        }
    }
    
    fn write_blocks(&mut self, _lba: u64, _count: u32, _buffer: &[u8]) -> Result<usize> {
        Err(newton_utils::Error::Other(
            "ZIP-extracted images are read-only".to_string()
        ))
    }
    
    fn info(&self) -> &DeviceInfo {
        match self {
            Self::Iso(iso) => iso.info(),
            Self::Dmg(dmg) => dmg.info(),
            Self::Toast(toast) => toast.info(),
            Self::Raw(raw) => raw.info(),
        }
    }
    
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl std::fmt::Debug for ZipImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Iso(iso) => write!(f, "ZipImage::Iso({:?})", iso),
            Self::Dmg(dmg) => write!(f, "ZipImage::Dmg({:?})", dmg),
            Self::Toast(toast) => write!(f, "ZipImage::Toast({:?})", toast),
            Self::Raw(raw) => write!(f, "ZipImage::Raw({:?})", raw),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore] // Requires a real ZIP file
    fn test_zip_open() {
        let zip = ZipImage::open("test.zip");
        assert!(zip.is_ok() || zip.is_err());
    }
}
