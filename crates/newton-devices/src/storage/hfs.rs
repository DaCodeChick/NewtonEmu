// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Minimal Classic HFS (Hierarchical File System) parser
//! 
//! This implements just enough of classic HFS to read files from Mac OS 9 CD-ROMs.
//! It does NOT implement the full HFS specification - only what's needed to find
//! and read the "Mac OS ROM" file from the System Folder.

use std::io::{Read, Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt};
use anyhow::{Result, bail, Context};

/// HFS Master Directory Block signature ("BD" = 0x4244)
const HFS_SIGNATURE: u16 = 0x4244;

/// HFS Master Directory Block (MDB)
/// Located at offset 1024 bytes from the start of the volume
#[derive(Debug)]
pub struct MasterDirectoryBlock {
    pub signature: u16,
    pub created_at: u32,
    pub modified_at: u32,
    pub attributes: u16,
    pub root_file_count: u16,
    pub volume_bitmap_start: u16,
    pub next_allocation: u16,
    pub total_blocks: u16,
    pub block_size: u32,
    pub clump_size: u32,
    pub first_alloc_block: u16,
    pub next_cnid: u32,
    pub free_blocks: u16,
    pub volume_name: String,
    
    // Extent overflow file
    pub xt_extents: [ExtentDescriptor; 3],
    pub xt_size: u32,
    pub xt_clump: u32,
    
    // Catalog file
    pub ct_extents: [ExtentDescriptor; 3],
    pub ct_size: u32,
    pub ct_clump: u32,
}

/// An extent descriptor: contiguous range of blocks
#[derive(Debug, Clone, Copy, Default)]
pub struct ExtentDescriptor {
    pub start_block: u16,
    pub block_count: u16,
}

impl MasterDirectoryBlock {
    /// Parse the MDB from a reader positioned at the start of the HFS volume
    pub fn parse<R: Read + Seek>(reader: &mut R, volume_offset: u64) -> Result<Self> {
        // MDB is at offset 1024 from the volume start
        let mdb_offset = volume_offset + 1024;
        tracing::debug!("Reading MDB at offset: 0x{:X}", mdb_offset);
        reader.seek(SeekFrom::Start(mdb_offset))?;
        
        let signature = reader.read_u16::<BigEndian>()?;
        tracing::debug!("MDB signature: 0x{:04X}", signature);
        if signature != HFS_SIGNATURE {
            bail!("Not a classic HFS volume (signature: 0x{:04X})", signature);
        }
        
        let created_at = reader.read_u32::<BigEndian>()?;
        let modified_at = reader.read_u32::<BigEndian>()?;
        let attributes = reader.read_u16::<BigEndian>()?;
        let root_file_count = reader.read_u16::<BigEndian>()?;
        let volume_bitmap_start = reader.read_u16::<BigEndian>()?;
        let next_allocation = reader.read_u16::<BigEndian>()?;
        let total_blocks = reader.read_u16::<BigEndian>()?;
        let block_size = reader.read_u32::<BigEndian>()?;
        let clump_size = reader.read_u32::<BigEndian>()?;
        let first_alloc_block = reader.read_u16::<BigEndian>()?;
        let next_cnid = reader.read_u32::<BigEndian>()?;
        let free_blocks = reader.read_u16::<BigEndian>()?;
        
        // Volume name: 1 byte length + up to 27 bytes name
        let name_len = reader.read_u8()?;
        let mut name_bytes = vec![0u8; 27];
        reader.read_exact(&mut name_bytes)?;
        let volume_name = String::from_utf8_lossy(&name_bytes[..name_len.min(27) as usize]).to_string();
        
        // Skip to extent overflow file info (offset 114 from MDB start)
        reader.seek(SeekFrom::Start(volume_offset + 1024 + 114))?;
        
        // Read extent overflow file extents
        let mut xt_extents = [ExtentDescriptor::default(); 3];
        for i in 0..3 {
            xt_extents[i] = ExtentDescriptor {
                start_block: reader.read_u16::<BigEndian>()?,
                block_count: reader.read_u16::<BigEndian>()?,
            };
        }
        let xt_size = reader.read_u32::<BigEndian>()?;
        let xt_clump = reader.read_u32::<BigEndian>()?;
        
        // Read catalog file extents
        let mut ct_extents = [ExtentDescriptor::default(); 3];
        for i in 0..3 {
            ct_extents[i] = ExtentDescriptor {
                start_block: reader.read_u16::<BigEndian>()?,
                block_count: reader.read_u16::<BigEndian>()?,
            };
        }
        let ct_size = reader.read_u32::<BigEndian>()?;
        let ct_clump = reader.read_u32::<BigEndian>()?;
        
        Ok(MasterDirectoryBlock {
            signature,
            created_at,
            modified_at,
            attributes,
            root_file_count,
            volume_bitmap_start,
            next_allocation,
            total_blocks,
            block_size,
            clump_size,
            first_alloc_block,
            next_cnid,
            free_blocks,
            volume_name,
            xt_extents,
            xt_size,
            xt_clump,
            ct_extents,
            ct_size,
            ct_clump,
        })
    }
    
    /// Calculate the byte offset of an allocation block
    pub fn block_offset(&self, block_num: u16) -> u64 {
        // First allocation block starts at first_alloc_block * 512
        let base_offset = self.first_alloc_block as u64 * 512;
        base_offset + (block_num as u64 * self.block_size as u64)
    }
}

/// HFS Volume - provides read access to an HFS filesystem
pub struct HfsVolume<R: Read + Seek> {
    reader: R,
    mdb: MasterDirectoryBlock,
    volume_offset: u64,
}

impl<R: Read + Seek> HfsVolume<R> {
    /// Open an HFS volume from a reader
    /// volume_offset is the byte offset where the HFS volume starts
    pub fn open(mut reader: R, volume_offset: u64) -> Result<Self> {
        let mdb = MasterDirectoryBlock::parse(&mut reader, volume_offset)?;
        
        tracing::info!("HFS volume: '{}', {} blocks of {} bytes", 
                      mdb.volume_name, mdb.total_blocks, mdb.block_size);
        
        Ok(HfsVolume {
            reader,
            mdb,
            volume_offset,
        })
    }
    
    /// Read data from an extent list
    fn read_extent(&mut self, extents: &[ExtentDescriptor], output: &mut Vec<u8>) -> Result<()> {
        for extent in extents {
            if extent.block_count == 0 {
                break;
            }
            
            let offset = self.volume_offset + self.mdb.block_offset(extent.start_block);
            let size = extent.block_count as u64 * self.mdb.block_size as u64;
            
            self.reader.seek(SeekFrom::Start(offset))?;
            let mut buf = vec![0u8; size as usize];
            self.reader.read_exact(&mut buf)?;
            output.extend_from_slice(&buf);
        }
        Ok(())
    }
    
    /// Find a file in the catalog by path (e.g., "System Folder:Mac OS ROM")
    pub fn find_file(&mut self, path: &str) -> Result<Vec<u8>> {
        // For now, just return a placeholder error
        // Full catalog B-tree parsing would be needed here
        bail!("HFS catalog parsing not yet implemented");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    
    #[test]
    fn test_parse_macos_cd() {
        let file = File::open("../../disks/macos-922-uni.iso").unwrap();
        
        // HFS partition starts at block 329, with 512-byte blocks (Apple Partition Map uses 512-byte blocks)
        let hfs_offset = 329 * 512;
        
        println!("Opening HFS volume at offset: 0x{:X}", hfs_offset);
        let volume = HfsVolume::open(file, hfs_offset).unwrap();
        println!("Volume: {:?}", volume.mdb);
    }
}
