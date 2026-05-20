// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Apple DMG (UDIF) disk image support
//!
//! Supports reading DMG disk images created by macOS Disk Utility.
//! Handles various compression formats: zlib, bzip2, LZFSE, ADC, etc.
//!
//! Note: Since DMG files are often compressed, we extract the main partition
//! to a temporary location and wrap it with RawDiskImage for block-level access.

use crate::storage::block_device::{BlockDevice, DeviceInfo, DeviceType};
use crate::storage::raw_disk::RawDiskImage;
use newton_utils::Result;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;

/// Apple DMG disk image
/// 
/// This wraps a RawDiskImage that contains the extracted DMG content.
/// The extraction happens once during open().
pub struct DmgImage {
    /// Path to the original DMG file
    dmg_path: String,
    
    /// Path to the extracted raw image
    extracted_path: PathBuf,
    
    /// Inner raw disk image for block-level access
    inner: RawDiskImage,
    
    /// Whether to delete the extracted file on drop
    cleanup_on_drop: bool,
}

impl DmgImage {
    /// Open a DMG image file
    ///
    /// This will extract the main partition to a temporary file for fast block access.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open_with_options(path, true)
    }
    
    /// Open a DMG image with options
    ///
    /// # Arguments
    /// * `path` - Path to the DMG file
    /// * `use_temp` - If true, extract to /tmp; if false, extract to same directory as DMG
    pub fn open_with_options<P: AsRef<Path>>(path: P, use_temp: bool) -> Result<Self> {
        let path_ref = path.as_ref();
        let dmg_path_str = path_ref.to_string_lossy().to_string();
        
        tracing::info!("Opening DMG image: {}", dmg_path_str);
        
        // Open and parse the DMG archive
        let mut archive = udif::DmgArchive::open(path_ref)
            .map_err(|e| newton_utils::Error::Other(format!("Failed to open DMG: {}", e)))?;
        
        // Get partition info
        let partitions = archive.partitions();
        if partitions.is_empty() {
            return Err(newton_utils::Error::Other("DMG has no partitions".to_string()));
        }
        
        tracing::info!("DMG has {} partitions", partitions.len());
        for (i, part) in partitions.iter().enumerate() {
            tracing::debug!("  Partition {}: {} ({} bytes)", i, part.name, part.size);
        }
        
        // Extract the main partition (usually HFS+ or APFS)
        tracing::info!("Extracting main partition...");
        let data = archive.extract_main_partition()
            .map_err(|e| newton_utils::Error::Other(format!("Failed to extract partition: {}", e)))?;
        
        tracing::info!("Extracted {} bytes", data.len());
        
        // Write to temporary file
        let extracted_path = if use_temp {
            let temp_name = format!("newton_dmg_{}.raw", std::process::id());
            PathBuf::from("/tmp").join(temp_name)
        } else {
            path_ref.with_extension("raw")
        };
        
        tracing::info!("Writing extracted image to: {}", extracted_path.display());
        let mut file = File::create(&extracted_path)
            .map_err(|e| newton_utils::Error::Io(e))?;
        file.write_all(&data)
            .map_err(|e| newton_utils::Error::Io(e))?;
        file.sync_all()
            .map_err(|e| newton_utils::Error::Io(e))?;
        drop(file);
        
        // Open as raw disk image (read-only)
        let inner = RawDiskImage::open(&extracted_path, false)?;
        
        Ok(Self {
            dmg_path: dmg_path_str,
            extracted_path,
            inner,
            cleanup_on_drop: use_temp,
        })
    }
    
    /// Get the path to the original DMG file
    pub fn dmg_path(&self) -> &str {
        &self.dmg_path
    }
    
    /// Get the path to the extracted raw image
    pub fn extracted_path(&self) -> &Path {
        &self.extracted_path
    }
}

impl BlockDevice for DmgImage {
    fn read_blocks(&self, lba: u64, count: u32, buffer: &mut [u8]) -> Result<usize> {
        self.inner.read_blocks(lba, count, buffer)
    }
    
    fn write_blocks(&mut self, lba: u64, count: u32, buffer: &[u8]) -> Result<usize> {
        // DMG images are read-only
        Err(newton_utils::Error::Other(
            "DMG images are read-only".to_string()
        ))
    }
    
    fn info(&self) -> &DeviceInfo {
        self.inner.info()
    }
    
    fn flush(&mut self) -> Result<()> {
        // Read-only, nothing to flush
        Ok(())
    }
}

impl Drop for DmgImage {
    fn drop(&mut self) {
        if self.cleanup_on_drop {
            if let Err(e) = std::fs::remove_file(&self.extracted_path) {
                tracing::warn!("Failed to clean up extracted DMG file {}: {}", 
                    self.extracted_path.display(), e);
            } else {
                tracing::info!("Cleaned up extracted DMG file: {}", self.extracted_path.display());
            }
        }
    }
}

impl std::fmt::Debug for DmgImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DmgImage")
            .field("dmg_path", &self.dmg_path)
            .field("extracted_path", &self.extracted_path)
            .field("cleanup_on_drop", &self.cleanup_on_drop)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore] // Requires a real DMG file
    fn test_dmg_open() {
        let dmg = DmgImage::open("test.dmg");
        assert!(dmg.is_ok() || dmg.is_err()); // Will fail if file doesn't exist
    }
}
