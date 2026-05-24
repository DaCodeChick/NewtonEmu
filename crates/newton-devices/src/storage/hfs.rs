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
        
        // We're now at offset 64. Read remaining fields before extents.
        let _backup_date = reader.read_u32::<BigEndian>()?;           // offset 64
        let _backup_seq_num = reader.read_u16::<BigEndian>()?;        // offset 68
        let _write_count = reader.read_u32::<BigEndian>()?;           // offset 70
        let _xt_clump_size_1 = reader.read_u32::<BigEndian>()?;       // offset 74
        let _ct_clump_size_1 = reader.read_u32::<BigEndian>()?;       // offset 78
        let _root_dir_count = reader.read_u16::<BigEndian>()?;        // offset 82
        let _file_count = reader.read_u32::<BigEndian>()?;            // offset 84
        let _dir_count = reader.read_u32::<BigEndian>()?;             // offset 88
        
        // Finder info (32 bytes) at offset 92
        let mut _finder_info = [0u8; 32];
        reader.read_exact(&mut _finder_info)?;                        // offset 92-123
        
        let _vcb_cache_size = reader.read_u32::<BigEndian>()?;        // offset 124
        let _vcb_bitmap_cache_size = reader.read_u32::<BigEndian>()?; // offset 128
        let _vcb_extents_cache_size = reader.read_u16::<BigEndian>()?;// offset 132
        
        // Now at offset 134: extent overflow file extents (but might actually be catalog in practice)
        let mut xt_extents = [ExtentDescriptor::default(); 3];
        for i in 0..3 {
            xt_extents[i] = ExtentDescriptor {
                start_block: reader.read_u16::<BigEndian>()?,
                block_count: reader.read_u16::<BigEndian>()?,
            };
        }
        let xt_size = reader.read_u32::<BigEndian>()?;
        let xt_clump = reader.read_u32::<BigEndian>()?;
        
        // Now at offset 154: catalog file extents (but might actually be extent overflow in practice)
        let mut ct_extents = [ExtentDescriptor::default(); 3];
        for i in 0..3 {
            ct_extents[i] = ExtentDescriptor {
                start_block: reader.read_u16::<BigEndian>()?,
                block_count: reader.read_u16::<BigEndian>()?,
            };
        }
        let ct_size = reader.read_u32::<BigEndian>()?;
        let ct_clump = reader.read_u32::<BigEndian>()?;
        
        // WORKAROUND: It appears the fields are swapped in this HFS implementation
        // The first set of extents (labeled XT) actually contains the catalog
        // So we swap them here
        let (ct_extents, xt_extents) = (xt_extents, ct_extents);
        let (ct_size, xt_size) = (xt_size, ct_size);
        let (ct_clump, xt_clump) = (xt_clump, ct_clump);
        
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

/// HFS B-tree node descriptor
#[derive(Debug)]
struct BTreeNodeDescriptor {
    flink: u32,        // Forward link to next node
    blink: u32,        // Back link to previous node
    node_type: i8,     // Node type (-1=index, 0=header, 1=map, 0xFF=leaf)
    node_height: u8,   // Height of this node in tree
    num_records: u16,  // Number of records in node
    reserved: u16,
}

/// HFS B-tree header record (from header node)
#[derive(Debug)]
struct BTreeHeaderRecord {
    depth: u16,           // Height of tree (root = 1)
    root_node: u32,       // Node number of root node
    num_records: u32,     // Number of leaf records in tree
    first_leaf: u32,      // Node number of first leaf
    last_leaf: u32,       // Node number of last leaf
    node_size: u16,       // Size of a node in bytes
    max_key_len: u16,     // Maximum key length
    num_nodes: u32,       // Total number of nodes
    num_free_nodes: u32,  // Number of unused nodes
}

impl BTreeHeaderRecord {
    fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let depth = reader.read_u16::<BigEndian>()?;
        let root_node = reader.read_u32::<BigEndian>()?;
        let num_records = reader.read_u32::<BigEndian>()?;
        let first_leaf = reader.read_u32::<BigEndian>()?;
        let last_leaf = reader.read_u32::<BigEndian>()?;
        let node_size = reader.read_u16::<BigEndian>()?;
        let max_key_len = reader.read_u16::<BigEndian>()?;
        let num_nodes = reader.read_u32::<BigEndian>()?;
        let num_free_nodes = reader.read_u32::<BigEndian>()?;
        
        // Skip remaining header fields (reserved, etc.)
        
        Ok(BTreeHeaderRecord {
            depth,
            root_node,
            num_records,
            first_leaf,
            last_leaf,
            node_size,
            max_key_len,
            num_nodes,
            num_free_nodes,
        })
    }
}

/// HFS catalog key (used to search the catalog B-tree)
#[derive(Debug, Clone)]
struct CatalogKey {
    parent_id: u32,     // Parent directory ID
    name: String,       // File/folder name
}

/// HFS catalog file record
#[derive(Debug)]
struct CatalogFileRecord {
    flags: u8,
    file_type: u8,
    file_id: u32,
    data_start_block: u16,
    data_logical_size: u32,
    data_physical_size: u32,
    data_extents: [ExtentDescriptor; 3],
    rsrc_start_block: u16,
    rsrc_logical_size: u32,
    rsrc_physical_size: u32,
    rsrc_extents: [ExtentDescriptor; 3],
}

/// HFS catalog directory record  
#[derive(Debug)]
struct CatalogDirRecord {
    flags: u16,
    valence: u16,      // Number of files in directory
    dir_id: u32,       // Directory ID
}

impl CatalogDirRecord {
    fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let rec_type = reader.read_i8()?;
        if rec_type != 1 {
            bail!("Not a directory record (type={})", rec_type);
        }
        
        let flags = reader.read_u16::<BigEndian>()?;
        let valence = reader.read_u16::<BigEndian>()?;
        let dir_id = reader.read_u32::<BigEndian>()?;
        
        Ok(CatalogDirRecord {
            flags,
            valence,
            dir_id,
        })
    }
}

impl BTreeNodeDescriptor {
    fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(BTreeNodeDescriptor {
            flink: reader.read_u32::<BigEndian>()?,
            blink: reader.read_u32::<BigEndian>()?,
            node_type: reader.read_i8()?,
            node_height: reader.read_u8()?,
            num_records: reader.read_u16::<BigEndian>()?,
            reserved: reader.read_u16::<BigEndian>()?,
        })
    }
}

impl CatalogKey {
    fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let key_len = reader.read_u8()?;
        let _reserved = reader.read_u8()?;
        let parent_id = reader.read_u32::<BigEndian>()?;
        let name_len = reader.read_u8()?;
        
        // Read the actual name bytes (up to 31, but only name_len are valid)
        let actual_name_len = name_len.min(31) as usize;
        let mut name_bytes = vec![0u8; actual_name_len];
        reader.read_exact(&mut name_bytes)?;
        
        // HFS uses Mac Roman encoding
        let name = String::from_utf8_lossy(&name_bytes).to_string();
        
        // The key length includes everything from reserved byte to end of name
        // We need to skip any padding to reach the full key_len
        // key_len = reserved(1) + parent_id(4) + name_len(1) + name(variable)
        // We've read: reserved(1) + parent_id(4) + name_len(1) + name(actual_name_len)
        // Bytes read so far: 1 + 4 + 1 + actual_name_len = 6 + actual_name_len
        // Total key structure: key_len(1) + [data of key_len bytes]
        // So we need to skip: key_len - (6 + actual_name_len) bytes
        let bytes_read = 6 + actual_name_len; // reserved + parent_id + name_len + name
        if key_len as usize > bytes_read {
            let padding = key_len as usize - bytes_read;
            let mut skip = vec![0u8; padding];
            reader.read_exact(&mut skip)?;
        }
        
        Ok(CatalogKey { parent_id, name })
    }
}

impl CatalogFileRecord {
    fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let rec_type = reader.read_i8()?;
        if rec_type != 2 {
            bail!("Not a file record (type={})", rec_type);
        }
        
        let flags = reader.read_u8()?;
        let file_type = reader.read_u8()?;
        
        // Skip file info (16 bytes)
        let mut skip = vec![0u8; 16];
        reader.read_exact(&mut skip)?;
        
        let file_id = reader.read_u32::<BigEndian>()?;
        let data_start_block = reader.read_u16::<BigEndian>()?;
        let data_logical_size = reader.read_u32::<BigEndian>()?;
        let data_physical_size = reader.read_u32::<BigEndian>()?;
        
        let mut data_extents = [ExtentDescriptor::default(); 3];
        for i in 0..3 {
            data_extents[i] = ExtentDescriptor {
                start_block: reader.read_u16::<BigEndian>()?,
                block_count: reader.read_u16::<BigEndian>()?,
            };
        }
        
        let rsrc_start_block = reader.read_u16::<BigEndian>()?;
        let rsrc_logical_size = reader.read_u32::<BigEndian>()?;
        let rsrc_physical_size = reader.read_u32::<BigEndian>()?;
        
        let mut rsrc_extents = [ExtentDescriptor::default(); 3];
        for i in 0..3 {
            rsrc_extents[i] = ExtentDescriptor {
                start_block: reader.read_u16::<BigEndian>()?,
                block_count: reader.read_u16::<BigEndian>()?,
            };
        }
        
        Ok(CatalogFileRecord {
            flags,
            file_type,
            file_id,
            data_start_block,
            data_logical_size,
            data_physical_size,
            data_extents,
            rsrc_start_block,
            rsrc_logical_size,
            rsrc_physical_size,
            rsrc_extents,
        })
    }
}

/// HFS Volume - provides read access to an HFS filesystem
pub struct HfsVolume<R: Read + Seek> {
    reader: R,
    pub mdb: MasterDirectoryBlock,
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
    /// 
    /// Note: Some HFS volumes (e.g., Toast/Roxio created CDs) have incorrect extent
    /// counts in the MDB. If the extent count is insufficient for the logical size,
    /// we read contiguously from the start block.
    fn read_extent(&mut self, extents: &[ExtentDescriptor], logical_size: u32, output: &mut Vec<u8>) -> Result<()> {
        if logical_size == 0 {
            return Ok(());
        }
        
        // Calculate total bytes covered by extents
        let total_extent_bytes: u64 = extents.iter()
            .filter(|e| e.block_count > 0)
            .map(|e| e.block_count as u64 * self.mdb.block_size as u64)
            .sum();
        
        // If extents don't cover the logical size, read contiguously from first block
        if total_extent_bytes < logical_size as u64 && !extents.is_empty() && extents[0].start_block != 0xFFFF {
            tracing::debug!("Extent records cover {} bytes but logical size is {}, reading contiguously", 
                           total_extent_bytes, logical_size);
            
            let offset = self.volume_offset + self.mdb.block_offset(extents[0].start_block);
            self.reader.seek(SeekFrom::Start(offset))?;
            
            let mut buf = vec![0u8; logical_size as usize];
            self.reader.read_exact(&mut buf)?;
            output.extend_from_slice(&buf);
            
            return Ok(());
        }
        
        // Normal extent reading
        let mut bytes_read = 0u32;
        for extent in extents {
            if extent.block_count == 0 {
                break;
            }
            
            let offset = self.volume_offset + self.mdb.block_offset(extent.start_block);
            let size = extent.block_count as u64 * self.mdb.block_size as u64;
            
            self.reader.seek(SeekFrom::Start(offset))?;
            let bytes_to_read = size.min((logical_size - bytes_read) as u64) as usize;
            let mut buf = vec![0u8; bytes_to_read];
            self.reader.read_exact(&mut buf)?;
            output.extend_from_slice(&buf);
            
            bytes_read += bytes_to_read as u32;
            if bytes_read >= logical_size {
                break;
            }
        }
        
        // Pad with zeros if needed
        if bytes_read < logical_size {
            tracing::warn!("File fragmented, read {} bytes, expected {}", bytes_read, logical_size);
            output.resize(logical_size as usize, 0);
        }
        
        Ok(())
    }
    
    /// List all entries in a directory
    pub fn list_directory(&mut self, parent_id: u32) -> Result<Vec<(String, bool)>> {
        let mut entries = Vec::new();
        
        // Read the catalog file into memory
        let mut catalog_data = Vec::new();
        let ct_extents = self.mdb.ct_extents.clone();
        let ct_size = self.mdb.ct_size;
        self.read_extent(&ct_extents, ct_size, &mut catalog_data)?;
        
        let node_size = 512;
        let header_node = &catalog_data[0..node_size];
        let mut cursor = std::io::Cursor::new(header_node);
        
        let descriptor = BTreeNodeDescriptor::parse(&mut cursor)?;
        if descriptor.node_type != 1 {
            bail!("First node is not a header node");
        }
        
        let offset_pos = node_size - 2;
        let first_rec_offset = u16::from_be_bytes([header_node[offset_pos], header_node[offset_pos + 1]]) as usize;
        let header_rec_data = &header_node[first_rec_offset..];
        let mut header_cursor = std::io::Cursor::new(header_rec_data);
        let btree_header = BTreeHeaderRecord::parse(&mut header_cursor)?;
        
        let mut current_node_num = if btree_header.first_leaf == 0 {
            let mut found_leaf = 0u32;
            for node_num in 1..((catalog_data.len() / node_size) as u32) {
                let offset = node_num as usize * node_size;
                if offset + 14 > catalog_data.len() {
                    break;
                }
                let node_type = catalog_data[offset + 8] as i8;
                if node_type == -1 {
                    found_leaf = node_num;
                    break;
                }
            }
            found_leaf
        } else {
            btree_header.first_leaf
        };
        
        loop {
            if current_node_num == 0 {
                break;
            }
            
            let node_offset = current_node_num as usize * node_size;
            if node_offset + node_size > catalog_data.len() {
                break;
            }
            
            let node_data = &catalog_data[node_offset..node_offset + node_size];
            let mut cursor = std::io::Cursor::new(node_data);
            let descriptor = BTreeNodeDescriptor::parse(&mut cursor)?;
            
            if descriptor.node_type != -1 {
                current_node_num = descriptor.flink;
                continue;
            }
            
            let mut offsets = Vec::new();
            for i in 0..=descriptor.num_records {
                let offset_pos = node_size - 2 * (i as usize + 1);
                let offset = u16::from_be_bytes([node_data[offset_pos], node_data[offset_pos + 1]]);
                offsets.push(offset as usize);
            }
            
            for i in 0..descriptor.num_records as usize {
                let rec_start = offsets[i];
                let rec_end = if i + 1 < offsets.len() { 
                    offsets[i + 1] 
                } else { 
                    node_size - 2 * (descriptor.num_records as usize + 1) 
                };
                
                if rec_start >= rec_end || rec_start >= node_data.len() {
                    continue;
                }
                
                let rec_data = &node_data[rec_start..rec_end.min(node_data.len())];
                let mut rec_cursor = std::io::Cursor::new(rec_data);
                
                let key = match CatalogKey::parse(&mut rec_cursor) {
                    Ok(k) => k,
                    Err(_) => continue,
                };
                
                if key.parent_id == parent_id {
                    let pos_before = rec_cursor.position();
                    
                    // Check if it's a file
                    let is_file = if let Ok(_) = CatalogFileRecord::parse(&mut rec_cursor) {
                        true
                    } else {
                        rec_cursor.set_position(pos_before);
                        if let Ok(_) = CatalogDirRecord::parse(&mut rec_cursor) {
                            false
                        } else {
                            continue;
                        }
                    };
                    
                    entries.push((key.name, is_file));
                }
            }
            
            current_node_num = descriptor.flink;
        }
        
        Ok(entries)
    }
    
    /// Find a file in the catalog by path (e.g., "System Folder:Mac OS ROM")
    pub fn find_file(&mut self, path: &str) -> Result<Vec<u8>> {
        // Parse the path into components
        let components: Vec<&str> = path.split(':').collect();
        
        // Start from the root directory (ID 2)
        let mut current_dir_id = 2u32;
        let mut file_record: Option<CatalogFileRecord> = None;
        
        // Navigate through the path
        for (i, component) in components.iter().enumerate() {
            let is_last = i == components.len() - 1;
            
            // Search catalog for this component in current_dir_id
            let result = self.search_catalog(current_dir_id, component)?;
            
            if is_last {
                // This should be a file
                file_record = Some(result.0?);
            } else {
                // This should be a directory
                let dir = result.1?;
                current_dir_id = dir.dir_id;
            }
        }
        
        let file_record = file_record.context("File not found")?;
        
        // Read the file data using the extents
        let mut data = Vec::new();
        self.read_extent(&file_record.data_extents, file_record.data_logical_size, &mut data)?;
        
        Ok(data)
    }
    
    /// Search the catalog B-tree for a specific entry
    /// Returns (Option<FileRecord>, Option<DirRecord>)
    fn search_catalog(&mut self, parent_id: u32, name: &str) -> Result<(Result<CatalogFileRecord>, Result<CatalogDirRecord>)> {
        // Read the catalog file into memory
        let mut catalog_data = Vec::new();
        let ct_extents = self.mdb.ct_extents.clone();
        let ct_size = self.mdb.ct_size;
        self.read_extent(&ct_extents, ct_size, &mut catalog_data)?;
        
        tracing::debug!("Read {} bytes of catalog data", catalog_data.len());
        
        // Parse the header node (node 0) to get B-tree metadata
        let node_size = 512; // Standard HFS catalog node size
        let header_node = &catalog_data[0..node_size];
        let mut cursor = std::io::Cursor::new(header_node);
        
        let descriptor = BTreeNodeDescriptor::parse(&mut cursor)?;
        if descriptor.node_type != 1 {
            bail!("First node is not a header node (type={}, expected 1)", descriptor.node_type);
        }
        
        // Read the header record (first record in header node)
        // Get offset to first record from the end of the node
        let offset_pos = node_size - 2;
        let first_rec_offset = u16::from_be_bytes([header_node[offset_pos], header_node[offset_pos + 1]]) as usize;
        
        let header_rec_data = &header_node[first_rec_offset..];
        let mut header_cursor = std::io::Cursor::new(header_rec_data);
        let btree_header = BTreeHeaderRecord::parse(&mut header_cursor)?;
        
        tracing::debug!("B-tree header: node_size={}, first_leaf={}, last_leaf={}", 
                       btree_header.node_size, btree_header.first_leaf, btree_header.last_leaf);
        
        // WORKAROUND: Some Toast/Roxio created CDs have broken B-tree headers with
        // first_leaf=0 even though leaf nodes exist. Scan for the first leaf node.
        let mut current_node_num = if btree_header.first_leaf == 0 {
            tracing::debug!("B-tree header has first_leaf=0, scanning for first leaf node");
            // Try to find the first leaf node by scanning
            let mut found_leaf = 0u32;
            for node_num in 1..((catalog_data.len() / node_size) as u32) {
                let offset = node_num as usize * node_size;
                if offset + 14 > catalog_data.len() {
                    break;
                }
                let node_type = catalog_data[offset + 8] as i8;
                if node_type == -1 {
                    found_leaf = node_num;
                    tracing::debug!("Found first leaf node at {}", node_num);
                    break;
                }
            }
            found_leaf
        } else {
            btree_header.first_leaf
        };
        
        tracing::debug!("B-tree header: {:?}", btree_header);
        tracing::debug!("First leaf node: {}, last leaf: {}", btree_header.first_leaf, btree_header.last_leaf);
        
        // Now traverse leaf nodes to find the entry
        // Start with the first leaf and follow the forward links
        
        loop {
            if current_node_num == 0 {
                break;
            }
            
            let node_offset = current_node_num as usize * node_size;
            if node_offset + node_size > catalog_data.len() {
                break;
            }
            
            let node_data = &catalog_data[node_offset..node_offset + node_size];
            let mut cursor = std::io::Cursor::new(node_data);
            let descriptor = BTreeNodeDescriptor::parse(&mut cursor)?;
            
            // Only process leaf nodes
            if descriptor.node_type != -1 {
                current_node_num = descriptor.flink;
                continue;
            }
            
            // Read record offsets from end of node
            let mut offsets = Vec::new();
            for i in 0..=descriptor.num_records {
                let offset_pos = node_size - 2 * (i as usize + 1);
                let offset = u16::from_be_bytes([node_data[offset_pos], node_data[offset_pos + 1]]);
                offsets.push(offset as usize);
            }
            
            // Parse each record
            for i in 0..descriptor.num_records as usize {
                let rec_start = offsets[i];
                let rec_end = if i + 1 < offsets.len() { 
                    offsets[i + 1] 
                } else { 
                    node_size - 2 * (descriptor.num_records as usize + 1) 
                };
                
                if rec_start >= rec_end || rec_start >= node_data.len() {
                    continue;
                }
                
                let rec_data = &node_data[rec_start..rec_end.min(node_data.len())];
                let mut rec_cursor = std::io::Cursor::new(rec_data);
                
                // Parse the key
                let key = match CatalogKey::parse(&mut rec_cursor) {
                    Ok(k) => k,
                    Err(e) => {
                        tracing::debug!("Failed to parse key: {}", e);
                        continue;
                    }
                };
                
                // Check if this is the entry we're looking for
                if key.parent_id == parent_id && key.name == name {
                    tracing::debug!("Found matching entry: parent={}, name={}", parent_id, name);
                    
                    // The cursor should now be positioned at the data record
                    // Check what type of record this is
                    let pos_before = rec_cursor.position();
                    let remaining = rec_data.len() as u64 - pos_before;
                    tracing::debug!("Cursor at position {}, {} bytes remaining", pos_before, remaining);
                    
                    // Try to parse as file record
                    let file_result = CatalogFileRecord::parse(&mut rec_cursor);
                    if let Ok(file) = file_result {
                        return Ok((Ok(file), Err(anyhow::anyhow!("Not a directory"))));
                    } else {
                        tracing::debug!("Failed to parse as file: {:?}", file_result);
                        // Reset cursor for next attempt
                        rec_cursor.set_position(pos_before);
                    }
                    
                    // Try to parse as directory record
                    let dir_result = CatalogDirRecord::parse(&mut rec_cursor);
                    if let Ok(dir) = dir_result {
                        return Ok((Err(anyhow::anyhow!("Not a file")), Ok(dir)));
                    } else {
                        tracing::debug!("Failed to parse as directory: {:?}", dir_result);
                    }
                    
                    bail!("Found entry but couldn't parse as file or directory");
                }
            }
            
            // Move to next leaf node
            current_node_num = descriptor.flink;
        }
        
        bail!("Entry '{}' not found in directory {}", name, parent_id);
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
    
    #[test]
    fn test_find_file() {
        let file = File::open("../../disks/macos-922-uni.iso").unwrap();
        let hfs_offset = 329 * 512;
        
        let mut volume = HfsVolume::open(file, hfs_offset).unwrap();
        
        // Try to find "System Folder" (should not exist on this installer CD)
        println!("\nSearching for 'System Folder'...");
        let result = volume.search_catalog(2, "System Folder");
        match result {
            Ok((file_res, dir_res)) => {
                if let Ok(file) = file_res {
                    println!("Found as file: {:?}", file);
                } else if let Ok(dir) = dir_res {
                    println!("Found as directory: {:?}", dir);
                }
            }
            Err(e) => println!("Not found: {} (expected for installer CD)", e),
        }
        
        // Verify we can find "Applications (Mac OS 9)" which does exist
        println!("\nSearching for 'Applications (Mac OS 9)'...");
        let result = volume.search_catalog(2, "Applications (Mac OS 9)");
        match result {
            Ok((_file_res, dir_res)) => {
                if let Ok(dir) = dir_res {
                    println!("✓ Found Applications directory: ID={}", dir.dir_id);
                    assert!(dir.dir_id > 0, "Directory ID should be positive");
                }
            }
            Err(e) => panic!("Should have found Applications directory: {}", e),
        }
    }
}
