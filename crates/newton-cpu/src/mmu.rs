// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! PowerPC Memory Management Unit (MMU) implementation
//!
//! Implements the 32-bit PowerPC MMU with:
//! - BAT (Block Address Translation) for fast large-block translation
//! - Segment registers for 256MB virtual segments
//! - Page tables (HTAB) for 4KB page translation
//! - TLB cache for performance

use newton_utils::{Result, Error};
use std::collections::HashMap;

use crate::PhysicalMemory;

/// BAT Register Pair (BATU + BATL)
#[derive(Debug, Clone, Copy)]
pub struct BatRegister {
    /// Upper register (BATU) - contains virtual address and protection
    pub upper: u32,
    /// Lower register (BATL) - contains physical address and permissions
    pub lower: u32,
}

impl BatRegister {
    pub fn new() -> Self {
        Self {
            upper: 0,
            lower: 0,
        }
    }

    /// Check if this BAT entry is valid (V bit set in BATU)
    pub fn is_valid(&self) -> bool {
        (self.upper & 0x3) != 0  // Vs or Vp bit set
    }

    /// Get the Block Effective Page Index (BEPI) - virtual address base
    pub fn bepi(&self) -> u32 {
        self.upper & 0xFFFE_0000
    }

    /// Get the Block Length (BL) field - size of block
    pub fn block_length(&self) -> u32 {
        (self.upper >> 2) & 0x7FF
    }

    /// Get block size in bytes
    pub fn block_size(&self) -> u32 {
        let bl = self.block_length();
        128 * 1024 * (bl + 1)  // 128KB * (BL + 1)
    }

    /// Get the Block Real Page Number (BRPN) - physical address base
    pub fn brpn(&self) -> u32 {
        self.lower & 0xFFFE_0000
    }

    /// Check if address matches this BAT entry
    pub fn matches(&self, vaddr: u32, msr_pr: bool) -> bool {
        if !self.is_valid() {
            return false;
        }

        // Check privilege level (Vs=supervisor, Vp=user)
        let vs = (self.upper & 0x2) != 0;
        let vp = (self.upper & 0x1) != 0;
        
        if msr_pr {
            // User mode: need Vp bit
            if !vp {
                return false;
            }
        } else {
            // Supervisor mode: need Vs bit
            if !vs {
                return false;
            }
        }

        // Check if virtual address is within this block
        let bepi = self.bepi();
        let size = self.block_size();
        let mask = size - 1;
        
        (vaddr & !mask) == (bepi & !mask)
    }

    /// Translate virtual address to physical using this BAT
    pub fn translate(&self, vaddr: u32) -> u32 {
        let size = self.block_size();
        let mask = size - 1;
        let offset = vaddr & mask;
        let brpn = self.brpn();
        
        brpn | offset
    }

    /// Get memory protection bits (WIMG)
    pub fn wimg(&self) -> u8 {
        ((self.lower >> 3) & 0xF) as u8
    }

    /// Get access protection (PP bits)
    pub fn pp(&self) -> u8 {
        (self.lower & 0x3) as u8
    }
}

/// Page Table Entry (PTE) structure
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry {
    /// Word 0: V | VSID | H | API
    pub word0: u32,
    /// Word 1: RPN | R | C | WIMG | PP
    pub word1: u32,
}

impl PageTableEntry {
    /// Check if entry is valid
    pub fn is_valid(&self) -> bool {
        (self.word0 & 0x8000_0000) != 0
    }

    /// Get Virtual Segment ID
    pub fn vsid(&self) -> u32 {
        (self.word0 >> 7) & 0x00FF_FFFF
    }

    /// Get hash function indicator
    pub fn hash_secondary(&self) -> bool {
        (self.word0 & 0x0000_0040) != 0
    }

    /// Get Abbreviated Page Index
    pub fn api(&self) -> u32 {
        self.word0 & 0x0000_003F
    }

    /// Get Real Page Number (physical address)
    pub fn rpn(&self) -> u32 {
        self.word1 & 0xFFFF_F000
    }

    /// Get WIMG bits (memory attributes)
    pub fn wimg(&self) -> u8 {
        ((self.word1 >> 3) & 0xF) as u8
    }

    /// Get PP bits (page protection)
    pub fn pp(&self) -> u8 {
        (self.word1 & 0x3) as u8
    }

    /// Get R bit (referenced)
    pub fn referenced(&self) -> bool {
        (self.word1 & 0x0000_0100) != 0
    }

    /// Get C bit (changed/dirty)
    pub fn changed(&self) -> bool {
        (self.word1 & 0x0000_0080) != 0
    }

    /// Set R bit (referenced)
    pub fn set_referenced(&mut self) {
        self.word1 |= 0x0000_0100;
    }

    /// Set C bit (changed)
    pub fn set_changed(&mut self) {
        self.word1 |= 0x0000_0080;
    }
}

/// TLB (Translation Lookaside Buffer) entry
#[derive(Debug, Clone, Copy)]
pub struct TlbEntry {
    /// Virtual address (aligned to page)
    pub vaddr: u32,
    /// Physical address
    pub paddr: u32,
    /// Page protection (PP bits)
    pub pp: u8,
    /// WIMG bits
    pub wimg: u8,
    /// Valid flag
    pub valid: bool,
}

/// PowerPC MMU state
#[derive(Debug, Clone)]
pub struct Mmu {
    /// Instruction BAT registers (IBAT0-IBAT3)
    pub ibat: [BatRegister; 4],
    /// Data BAT registers (DBAT0-DBAT3)
    pub dbat: [BatRegister; 4],
    
    /// SDR1 register - Page Table base address
    pub sdr1: u32,
    
    /// TLB cache (virtual address -> TLB entry)
    tlb: HashMap<u32, TlbEntry>,
}

impl Mmu {
    pub fn new() -> Self {
        Self {
            ibat: [BatRegister::new(); 4],
            dbat: [BatRegister::new(); 4],
            sdr1: 0,
            tlb: HashMap::new(),
        }
    }

    /// Reset MMU state
    pub fn reset(&mut self) {
        self.ibat = [BatRegister::new(); 4];
        self.dbat = [BatRegister::new(); 4];
        self.sdr1 = 0;
        self.tlb.clear();
    }

    /// Invalidate entire TLB
    pub fn tlb_invalidate_all(&mut self) {
        self.tlb.clear();
    }

    /// Invalidate single TLB entry by virtual address
    pub fn tlb_invalidate(&mut self, vaddr: u32) {
        let page_addr = vaddr & 0xFFFF_F000;
        self.tlb.remove(&page_addr);
    }

    /// Translate virtual address to physical (for data access)
    pub fn translate_data(&mut self, vaddr: u32, sr: &[u32; 16], msr: u32, is_write: bool, memory: &dyn PhysicalMemory) -> Result<u32> {
        let msr_dr = (msr & 0x0010) != 0;  // Data address translation enabled
        let msr_pr = (msr & 0x4000) != 0;  // Problem state (user mode)

        // If translation disabled, use physical address directly
        if !msr_dr {
            return Ok(vaddr);
        }

        // Try BAT translation first (fast path)
        for bat in &self.dbat {
            if bat.matches(vaddr, msr_pr) {
                let paddr = bat.translate(vaddr);
                return Ok(paddr);
            }
        }

        // Check TLB cache
        let page_addr = vaddr & 0xFFFF_F000;
        if let Some(entry) = self.tlb.get(&page_addr) {
            if entry.valid {
                let offset = vaddr & 0x0FFF;
                return Ok(entry.paddr | offset);
            }
        }

        // Page table translation (slow path)
        self.translate_page(vaddr, sr, is_write, memory)
    }

    /// Translate virtual address to physical (for instruction fetch)
    pub fn translate_instruction(&mut self, vaddr: u32, sr: &[u32; 16], msr: u32, memory: &dyn PhysicalMemory) -> Result<u32> {
        let msr_ir = (msr & 0x0020) != 0;  // Instruction address translation enabled
        let msr_pr = (msr & 0x4000) != 0;  // Problem state (user mode)

        // If translation disabled, use physical address directly
        if !msr_ir {
            return Ok(vaddr);
        }

        // Try BAT translation first
        for bat in &self.ibat {
            if bat.matches(vaddr, msr_pr) {
                let paddr = bat.translate(vaddr);
                return Ok(paddr);
            }
        }

        // Check TLB cache
        let page_addr = vaddr & 0xFFFF_F000;
        if let Some(entry) = self.tlb.get(&page_addr) {
            if entry.valid {
                let offset = vaddr & 0x0FFF;
                return Ok(entry.paddr | offset);
            }
        }

        // Page table translation
        self.translate_page(vaddr, sr, false, memory)
    }

    /// Perform page table translation
    fn translate_page(&mut self, vaddr: u32, sr: &[u32; 16], is_write: bool, memory: &dyn PhysicalMemory) -> Result<u32> {
        // Get segment register
        let seg = (vaddr >> 28) as usize;
        let sr_val = sr[seg];
        
        // Check T bit (if set, use direct-store segment, not implemented)
        if (sr_val & 0x8000_0000) != 0 {
            return Err(Error::Memory("Direct-store segments not implemented".to_string()));
        }

        // Extract VSID from segment register
        let vsid = sr_val & 0x00FF_FFFF;

        // Calculate page index and hash
        let page_index = (vaddr >> 12) & 0xFFFF;
        let hash1 = (vsid ^ page_index) & 0x7FFFF;

        // Get page table base from SDR1
        let htaborg = self.sdr1 & 0xFFFF_0000;
        let htabmask = ((self.sdr1 & 0x1FF) << 16) | 0xFFFF;

        // Try primary hash
        if let Some(pte) = self.lookup_pte(memory, htaborg, htabmask, hash1, vsid, page_index, false)? {
            return self.complete_translation(vaddr, pte, is_write);
        }

        // Try secondary hash
        let hash2 = !hash1 & 0x7FFFF;
        if let Some(pte) = self.lookup_pte(memory, htaborg, htabmask, hash2, vsid, page_index, true)? {
            return self.complete_translation(vaddr, pte, is_write);
        }

        // Page fault
        Err(Error::Memory(format!("Page fault: no PTE found for vaddr 0x{:08X}", vaddr)))
    }

    /// Look up PTE in page table by reading from physical memory
    fn lookup_pte(&self, memory: &dyn PhysicalMemory, htaborg: u32, htabmask: u32, hash: u32, vsid: u32, page_index: u32, secondary: bool) -> Result<Option<PageTableEntry>> {
        // Calculate PTEG (Page Table Entry Group) address
        let pteg_addr = (htaborg & !htabmask) | ((hash << 6) & htabmask);

        // Each PTEG contains 8 PTEs, each PTE is 8 bytes (2 words)
        for i in 0..8 {
            let pte_addr = pteg_addr + (i * 8);
            
            // Read PTE from physical memory
            let word0 = memory.read_u32_phys(pte_addr)?;
            let word1 = memory.read_u32_phys(pte_addr + 4)?;
            
            let pte = PageTableEntry { word0, word1 };
            
            // Check if PTE is valid
            if !pte.is_valid() {
                continue;
            }
            
            // Check if VSID matches
            if pte.vsid() != vsid {
                continue;
            }
            
            // Check if API matches (low 6 bits of page index)
            let api = page_index & 0x3F;
            if pte.api() != api {
                continue;
            }
            
            // Check if hash function indicator matches
            if pte.hash_secondary() != secondary {
                continue;
            }
            
            // Found matching PTE
            return Ok(Some(pte));
        }
        
        // No matching PTE found in this PTEG
        Ok(None)
    }

    /// Complete translation using PTE
    fn complete_translation(&mut self, vaddr: u32, pte: PageTableEntry, is_write: bool) -> Result<u32> {
        // Check page protection
        // PP bits: 00=read/write, 01=read/write, 10=read-only, 11=no access
        let pp = pte.pp();
        if pp == 0b11 {
            return Err(Error::Memory("Page protection violation: no access".to_string()));
        }
        if is_write && pp == 0b10 {
            return Err(Error::Memory("Page protection violation: read-only".to_string()));
        }

        // Calculate physical address
        let rpn = pte.rpn();
        let offset = vaddr & 0x0FFF;
        let paddr = rpn | offset;

        // Cache in TLB
        let page_addr = vaddr & 0xFFFF_F000;
        self.tlb.insert(page_addr, TlbEntry {
            vaddr: page_addr,
            paddr: rpn,
            pp: pte.pp(),
            wimg: pte.wimg(),
            valid: true,
        });

        Ok(paddr)
    }
}

impl Default for Mmu {
    fn default() -> Self {
        Self::new()
    }
}
