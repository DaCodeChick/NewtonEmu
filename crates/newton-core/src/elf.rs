// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! ELF file parser and loader for PowerPC
//!
//! The Mac OS NewWorld ROM contains an ELF executable that serves as the
//! bootloader/nanokernel. This module parses and loads PowerPC ELF files.

use newton_utils::Result;
use std::fmt;

/// ELF magic number
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

/// ELF file class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfClass {
    Elf32 = 1,
    Elf64 = 2,
}

/// ELF data encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfData {
    LittleEndian = 1,
    BigEndian = 2,
}

/// ELF file type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfType {
    None = 0,
    Rel = 1,
    Exec = 2,
    Dyn = 3,
    Core = 4,
}

/// ELF machine architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfMachine {
    None = 0,
    PowerPC = 20,
    PowerPC64 = 21,
}

/// Program header type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhType {
    Null = 0,
    Load = 1,
    Dynamic = 2,
    Interp = 3,
    Note = 4,
    Shlib = 5,
    Phdr = 6,
}

/// ELF file header (32-bit)
#[derive(Debug, Clone)]
pub struct ElfHeader {
    pub class: ElfClass,
    pub data: ElfData,
    pub version: u8,
    pub osabi: u8,
    pub abi_version: u8,
    pub file_type: ElfType,
    pub machine: ElfMachine,
    pub entry: u32,
    pub phoff: u32,
    pub shoff: u32,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

/// Program header (32-bit)
#[derive(Debug, Clone)]
pub struct ProgramHeader {
    pub p_type: u32,
    pub offset: u32,
    pub vaddr: u32,
    pub paddr: u32,
    pub filesz: u32,
    pub memsz: u32,
    pub flags: u32,
    pub align: u32,
}

/// Parsed ELF file
pub struct ElfFile {
    pub header: ElfHeader,
    pub program_headers: Vec<ProgramHeader>,
    data: Vec<u8>,
}

impl fmt::Debug for ElfFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElfFile")
            .field("header", &self.header)
            .field("program_headers", &self.program_headers)
            .field("data_size", &self.data.len())
            .finish()
    }
}

impl ElfFile {
    /// Parse an ELF file from bytes
    pub fn parse(data: Vec<u8>) -> Result<Self> {
        if data.len() < 52 {
            return Err(newton_utils::Error::Other("File too small for ELF header".to_string()));
        }

        // Check magic
        if &data[0..4] != &ELF_MAGIC {
            return Err(newton_utils::Error::Other("Not an ELF file".to_string()));
        }

        // Parse ELF identification
        let class = match data[4] {
            1 => ElfClass::Elf32,
            2 => ElfClass::Elf64,
            _ => return Err(newton_utils::Error::Other("Invalid ELF class".to_string())),
        };

        let data_encoding = match data[5] {
            1 => ElfData::LittleEndian,
            2 => ElfData::BigEndian,
            _ => return Err(newton_utils::Error::Other("Invalid ELF data encoding".to_string())),
        };

        let version = data[6];
        let osabi = data[7];
        let abi_version = data[8];

        // Only support 32-bit big-endian for now
        if class != ElfClass::Elf32 {
            return Err(newton_utils::Error::Other("Only 32-bit ELF supported".to_string()));
        }
        if data_encoding != ElfData::BigEndian {
            return Err(newton_utils::Error::Other("Only big-endian ELF supported".to_string()));
        }

        // Parse header fields (big-endian)
        let e_type = u16::from_be_bytes([data[16], data[17]]);
        let e_machine = u16::from_be_bytes([data[18], data[19]]);
        let _e_version = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        let e_entry = u32::from_be_bytes([data[24], data[25], data[26], data[27]]);
        let e_phoff = u32::from_be_bytes([data[28], data[29], data[30], data[31]]);
        let e_shoff = u32::from_be_bytes([data[32], data[33], data[34], data[35]]);
        let e_flags = u32::from_be_bytes([data[36], data[37], data[38], data[39]]);
        let e_ehsize = u16::from_be_bytes([data[40], data[41]]);
        let e_phentsize = u16::from_be_bytes([data[42], data[43]]);
        let e_phnum = u16::from_be_bytes([data[44], data[45]]);
        let e_shentsize = u16::from_be_bytes([data[46], data[47]]);
        let e_shnum = u16::from_be_bytes([data[48], data[49]]);
        let e_shstrndx = u16::from_be_bytes([data[50], data[51]]);

        let file_type = match e_type {
            0 => ElfType::None,
            1 => ElfType::Rel,
            2 => ElfType::Exec,
            3 => ElfType::Dyn,
            4 => ElfType::Core,
            _ => return Err(newton_utils::Error::Other(format!("Unknown ELF type: {}", e_type))),
        };

        let machine = match e_machine {
            0 => ElfMachine::None,
            20 => ElfMachine::PowerPC,
            21 => ElfMachine::PowerPC64,
            _ => return Err(newton_utils::Error::Other(format!("Unsupported architecture: {}", e_machine))),
        };

        if machine != ElfMachine::PowerPC {
            return Err(newton_utils::Error::Other("Only PowerPC ELF supported".to_string()));
        }

        let header = ElfHeader {
            class,
            data: data_encoding,
            version,
            osabi,
            abi_version,
            file_type,
            machine,
            entry: e_entry,
            phoff: e_phoff,
            shoff: e_shoff,
            flags: e_flags,
            ehsize: e_ehsize,
            phentsize: e_phentsize,
            phnum: e_phnum,
            shentsize: e_shentsize,
            shnum: e_shnum,
            shstrndx: e_shstrndx,
        };

        // Parse program headers
        let mut program_headers = Vec::new();
        if e_phoff > 0 && e_phnum > 0 {
            for i in 0..e_phnum {
                let offset = (e_phoff + (i as u32 * e_phentsize as u32)) as usize;
                if offset + 32 > data.len() {
                    break;
                }

                let p_type = u32::from_be_bytes([
                    data[offset], data[offset+1], data[offset+2], data[offset+3]
                ]);
                let p_offset = u32::from_be_bytes([
                    data[offset+4], data[offset+5], data[offset+6], data[offset+7]
                ]);
                let p_vaddr = u32::from_be_bytes([
                    data[offset+8], data[offset+9], data[offset+10], data[offset+11]
                ]);
                let p_paddr = u32::from_be_bytes([
                    data[offset+12], data[offset+13], data[offset+14], data[offset+15]
                ]);
                let p_filesz = u32::from_be_bytes([
                    data[offset+16], data[offset+17], data[offset+18], data[offset+19]
                ]);
                let p_memsz = u32::from_be_bytes([
                    data[offset+20], data[offset+21], data[offset+22], data[offset+23]
                ]);
                let p_flags = u32::from_be_bytes([
                    data[offset+24], data[offset+25], data[offset+26], data[offset+27]
                ]);
                let p_align = u32::from_be_bytes([
                    data[offset+28], data[offset+29], data[offset+30], data[offset+31]
                ]);

                program_headers.push(ProgramHeader {
                    p_type,
                    offset: p_offset,
                    vaddr: p_vaddr,
                    paddr: p_paddr,
                    filesz: p_filesz,
                    memsz: p_memsz,
                    flags: p_flags,
                    align: p_align,
                });
            }
        }

        Ok(ElfFile {
            header,
            program_headers,
            data,
        })
    }

    /// Get entry point address
    pub fn entry_point(&self) -> u32 {
        self.header.entry
    }

    /// Load ELF into memory
    /// Returns (memory_image, load_address, entry_point)
    pub fn load_to_memory(&self) -> Result<(Vec<u8>, u32, u32)> {
        // Find the minimum and maximum addresses we need to cover
        let mut min_addr = u32::MAX;
        let mut max_addr = 0u32;

        for ph in &self.program_headers {
            if ph.p_type == 1 && ph.memsz > 0 { // PT_LOAD
                min_addr = min_addr.min(ph.vaddr);
                max_addr = max_addr.max(ph.vaddr + ph.memsz);
            }
        }

        if min_addr == u32::MAX {
            return Err(newton_utils::Error::Other("No loadable segments found".to_string()));
        }

        let total_size = (max_addr - min_addr) as usize;
        let mut memory = vec![0u8; total_size];

        tracing::info!("Loading ELF: load range 0x{:08X}-0x{:08X} ({}  bytes)", 
                      min_addr, max_addr, total_size);

        // Load each PT_LOAD segment
        for (i, ph) in self.program_headers.iter().enumerate() {
            if ph.p_type == 1 && ph.memsz > 0 { // PT_LOAD
                tracing::debug!("  Segment[{}]: vaddr=0x{:08X} filesz=0x{:X} memsz=0x{:X}",
                              i, ph.vaddr, ph.filesz, ph.memsz);

                let mem_offset = (ph.vaddr - min_addr) as usize;
                
                // Copy file data
                if ph.filesz > 0 {
                    let file_start = ph.offset as usize;
                    let file_end = file_start + ph.filesz as usize;
                    
                    if file_end > self.data.len() {
                        return Err(newton_utils::Error::Other(
                            format!("Segment data beyond file bounds")
                        ));
                    }
                    
                    let data_to_copy = &self.data[file_start..file_end];
                    let mem_end = mem_offset + ph.filesz as usize;
                    
                    if mem_end > memory.len() {
                        return Err(newton_utils::Error::Other(
                            format!("Segment too large for allocated memory")
                        ));
                    }
                    
                    memory[mem_offset..mem_end].copy_from_slice(data_to_copy);
                }

                // Zero remaining bytes if memsz > filesz (BSS section)
                if ph.memsz > ph.filesz {
                    let zero_start = mem_offset + ph.filesz as usize;
                    let zero_end = mem_offset + ph.memsz as usize;
                    memory[zero_start..zero_end].fill(0);
                }
            }
        }

        Ok((memory, min_addr, self.header.entry))
    }

    /// Get the raw ELF data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf_magic() {
        // Minimal ELF header
        let mut data = vec![0u8; 52];
        data[0..4].copy_from_slice(&ELF_MAGIC);
        data[4] = 1; // 32-bit
        data[5] = 2; // big-endian
        data[18] = 0;
        data[19] = 20; // PowerPC

        let elf = ElfFile::parse(data).unwrap();
        assert_eq!(elf.header.class, ElfClass::Elf32);
        assert_eq!(elf.header.data, ElfData::BigEndian);
        assert_eq!(elf.header.machine, ElfMachine::PowerPC);
    }
}
