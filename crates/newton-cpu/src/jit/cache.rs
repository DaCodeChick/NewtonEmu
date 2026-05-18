// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! JIT code cache management

use super::block::BasicBlock;
use std::collections::HashMap;

/// Cache for compiled code blocks
pub struct CodeCache {
    /// Compiled basic blocks indexed by start address
    blocks: HashMap<u32, BasicBlock>,
    
    /// Total size of cached code
    total_size: usize,
}

impl CodeCache {
    /// Create a new code cache
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            total_size: 0,
        }
    }
    
    /// Insert a compiled block into the cache
    pub fn insert(&mut self, addr: u32, block: BasicBlock) {
        self.total_size += block.size_bytes();
        self.blocks.insert(addr, block);
    }
    
    /// Get a block from the cache
    pub fn get_block(&self, addr: u32) -> Option<&BasicBlock> {
        self.blocks.get(&addr)
    }
    
    /// Check if a block is cached
    pub fn contains(&self, addr: u32) -> bool {
        self.blocks.contains_key(&addr)
    }
    
    /// Remove a block from the cache
    pub fn remove(&mut self, addr: u32) -> Option<BasicBlock> {
        if let Some(block) = self.blocks.remove(&addr) {
            self.total_size -= block.size_bytes();
            Some(block)
        } else {
            None
        }
    }
    
    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.total_size = 0;
    }
    
    /// Get number of cached blocks
    pub fn len(&self) -> usize {
        self.blocks.len()
    }
    
    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
    
    /// Get total number of instructions in cache
    pub fn total_instructions(&self) -> usize {
        self.blocks.values().map(|b| b.instructions.len()).sum()
    }
    
    /// Get estimated cache size in bytes
    pub fn estimated_size(&self) -> usize {
        // Basic block metadata + instruction data + estimated compiled code size
        let metadata_size = self.blocks.len() * std::mem::size_of::<BasicBlock>();
        let instruction_size = self.total_size;
        let estimated_compiled_size = instruction_size * 8; // ~8 bytes native code per PPC instruction
        
        metadata_size + instruction_size + estimated_compiled_size
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            block_count: self.len(),
            instruction_count: self.total_instructions(),
            estimated_size: self.estimated_size(),
            hit_rate: 0.0, // TODO: Track hits/misses
        }
    }
}

impl Default for CodeCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of blocks in cache
    pub block_count: usize,
    /// Total instructions cached
    pub instruction_count: usize,
    /// Estimated memory usage in bytes
    pub estimated_size: usize,
    /// Cache hit rate (0.0 - 1.0)
    pub hit_rate: f64,
}
