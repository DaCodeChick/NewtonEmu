// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Memory management system

use crate::rom::Rom;
use newton_devices::MmioDevice;
use newton_utils::Result;
use byteorder::{BigEndian, ByteOrder};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use memmap2::MmapMut;

// Re-export the memory interface trait
pub use newton_cpu::MemoryInterface;

/// Memory address space
///
/// Thread-safe memory system using interior mutability.
/// ROM is immutable and requires no locking.
/// RAM is always memory-mapped for direct debugger access.
pub struct Memory {
    /// System RAM (memory-mapped, thread-safe with RwLock)
    ram_mmap: Arc<RwLock<MmapMut>>,
    
    /// RAM file handle (kept alive for mmap)
    #[allow(dead_code)]
    ram_file: File,
    
    /// RAM file path
    ram_path: PathBuf,
    
    /// Boot ROM (immutable, no lock needed)
    rom: Option<Rom>,
    
    /// ROM shadow RAM (writable overlay for ROM region, used during boot)
    /// When present, writes to ROM addresses go here, reads check here first
    rom_shadow: Option<Arc<RwLock<Vec<u8>>>>,
    
    /// Memory-mapped I/O devices (base_address -> (size, device))
    mmio_devices: HashMap<u32, (u32, Arc<RwLock<Box<dyn MmioDevice>>>)>,
}

impl Memory {
    /// Create new memory with specified RAM size (in bytes)
    /// RAM is always memory-mapped to /tmp for debugger access
    pub fn new(ram_size: usize) -> Result<Self> {
        tracing::info!("Initializing {} MB of memory-mapped RAM", ram_size / (1024 * 1024));
        
        // Create a temporary file for the RAM
        let path = PathBuf::from(format!("/tmp/newton_emu_ram_{}.bin", std::process::id()));
        
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        
        // Set the file size
        file.set_len(ram_size as u64)?;
        
        // Zero out the file
        file.write_all(&vec![0u8; ram_size])?;
        file.flush()?;
        
        // Memory-map the file
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        
        tracing::info!("RAM mapped to file: {}", path.display());
        
        Ok(Self {
            ram_mmap: Arc::new(RwLock::new(mmap)),
            ram_file: file,
            ram_path: path,
            rom: None,
            rom_shadow: None,
            mmio_devices: HashMap::new(),
        })
    }
    
    /// Get the path to the memory-mapped RAM file
    pub fn ram_path(&self) -> &PathBuf {
        &self.ram_path
    }

    /// Load ROM into memory
    pub fn load_rom(&mut self, rom: Rom) {
        tracing::info!("Loading ROM at 0x{:08X}, size {} bytes", rom.base_address(), rom.size());
        
        // Create ROM shadow buffer initialized with ROM contents
        // This allows the ROM code to write to ROM addresses during boot
        // Make it 8MB to accommodate decompressed Toolbox data (larger than raw ROM)
        // ROM shadow always covers 0xFFC00000-0xFFFFFFFF (4MB address space)
        const ROM_SHADOW_BASE: u32 = 0xFFC00000;
        let shadow_size = std::cmp::max(rom.size(), 8 * 1024 * 1024);
        let mut shadow = vec![0u8; shadow_size];
        
        // Copy ROM contents to the correct offset in shadow
        // For NewWorld ROMs at 0xFFC8E018, this is offset 0x8E018 in the shadow
        let rom_offset_in_shadow = if rom.base_address() >= ROM_SHADOW_BASE {
            (rom.base_address() - ROM_SHADOW_BASE) as usize
        } else {
            0
        };
        
        if rom_offset_in_shadow + rom.size() <= shadow_size {
            shadow[rom_offset_in_shadow..rom_offset_in_shadow + rom.size()].copy_from_slice(rom.data());
            tracing::info!("Copied ROM to shadow at offset 0x{:X} (address 0x{:08X})", 
                          rom_offset_in_shadow, rom.base_address());
        } else {
            tracing::error!("ROM doesn't fit in shadow: rom_offset={}, rom_size={}, shadow_size={}", 
                           rom_offset_in_shadow, rom.size(), shadow_size);
        }
        
        self.rom_shadow = Some(Arc::new(RwLock::new(shadow)));
        tracing::info!("ROM shadow enabled: {} bytes covering 0x{:08X}-0x{:08X}", 
                      shadow_size, ROM_SHADOW_BASE, ROM_SHADOW_BASE + shadow_size as u32 - 1);
        
        self.rom = Some(rom);
    }

    /// Register a memory-mapped I/O device
    pub fn register_mmio(&mut self, base_address: u32, size: u32, device: Box<dyn MmioDevice>) {
        tracing::info!("Registering MMIO device '{}' at 0x{:08X}-0x{:08X}", 
                       device.name(), base_address, base_address.wrapping_add(size - 1));
        self.mmio_devices.insert(base_address, (size, Arc::new(RwLock::new(device))));
    }

    /// Get RAM size
    pub fn ram_size(&self) -> usize {
        self.ram_mmap.read().len()
    }
    
    /// Get ROM reference
    pub fn rom(&self) -> Option<&Rom> {
        self.rom.as_ref()
    }
    
    /// Write data to ROM shadow at a specific ROM address
    /// This is used during boot to decompress the Toolbox into ROM space
    pub fn write_to_rom_shadow(&self, addr: u32, data: &[u8]) -> Result<()> {
        if let Some(shadow) = &self.rom_shadow {
            // ROM shadow always starts at 0xFFC00000 (traditional Mac ROM base)
            // regardless of where the actual NewWorld ROM is mapped
            const ROM_SHADOW_BASE: u32 = 0xFFC00000;
            
            if addr >= ROM_SHADOW_BASE {
                let offset = (addr - ROM_SHADOW_BASE) as usize;
                let mut shadow_buf = shadow.write();
                let end = offset + data.len();
                
                if end <= shadow_buf.len() {
                    shadow_buf[offset..end].copy_from_slice(data);
                    tracing::debug!("Wrote {} bytes to ROM shadow at 0x{:08X}", data.len(), addr);
                    return Ok(());
                } else {
                    return Err(newton_utils::Error::Memory(
                        format!("ROM shadow write would exceed bounds: offset={}, data_len={}, shadow_len={}", 
                               offset, data.len(), shadow_buf.len())
                    ));
                }
            } else {
                return Err(newton_utils::Error::Memory(
                    format!("Address 0x{:08X} is before ROM shadow base 0x{:08X}", addr, ROM_SHADOW_BASE)
                ));
            }
        } else {
            Err(newton_utils::Error::Memory("ROM shadow not enabled".to_string()))
        }
    }
    
    /// Initialize boot-time RAM structures
    ///
    /// Sets up minimal RAM initialization needed for ROM boot:
    /// - Exception vectors
    /// - Function descriptor tables
    /// - Stub functions that return immediately
    /// - Initial stack frame
    pub fn init_boot_ram(&self) {
        tracing::info!("Initializing boot-time RAM structures");
        
        let mut ram = self.ram_mmap.write();
        let ram = ram.as_mut();
        
        // Create a stub function at 0x1000 that just returns (blr)
        // blr = 0x4E800020
        let stub_addr = 0x1000;
        BigEndian::write_u32(&mut ram[stub_addr..], 0x4E800020);
        
        // Create an infinite loop stub at 0x1004 for "final return"
        // b -4 = branch to self = 0x4BFFFFFC
        BigEndian::write_u32(&mut ram[stub_addr + 4..], 0x48000000);  // b 0 (branch to self)
        
        // Create OpenFirmware client interface stub at 0x3000
        // Uses sc (system call) instruction to trap into emulator
        // r3 points to argument structure in memory
        let of_client_addr = 0x3000;
        if of_client_addr + 8 < ram.len() {
            BigEndian::write_u32(&mut ram[of_client_addr..], 0x44000002);      // sc (system call)
            BigEndian::write_u32(&mut ram[of_client_addr + 4..], 0x4E800020);  // blr (return)
        }
        
        // Initialize function descriptor at 0x2000
        // The descriptor is a data structure that POINTS to the function code
        // Function descriptor format on PowerPC:
        //   [0]: function address (where the code is)
        //   [4]: TOC pointer (r2)
        //   [8]: environment pointer (r11) - often unused  
        let func_descriptor = 0x2000;
        if func_descriptor + 12 < ram.len() {
            BigEndian::write_u32(&mut ram[func_descriptor..], stub_addr as u32);   // Function code at 0x1000
            BigEndian::write_u32(&mut ram[func_descriptor + 4..], 0x5100);         // TOC (r2) for callee
            BigEndian::write_u32(&mut ram[func_descriptor + 8..], 0);              // Environment
        }
        
        // Initialize function pointer table at 0x4DB0
        // This table contains pointers TO descriptors (not descriptors themselves)
        let func_ptr_table = 0x4DB0;
        if func_ptr_table + 4 < ram.len() {
            BigEndian::write_u32(&mut ram[func_ptr_table..], func_descriptor as u32);  // Point to descriptor
        }
        
        // Initialize Mac ROM globals/TOC structure at 0x5000
        // This is a data structure that r2 will point to
        // Mac ROM uses r2 to access system globals and function pointer tables
        let globals_base = 0x5000;
        if globals_base + 0x200 < ram.len() {
            // Clear the globals area
            for i in 0..0x200 {
                ram[globals_base + i] = 0;
            }
            
            // Set up function pointer table entries  
            // The globals table at [r2-72] should point to a function pointer table
            // The function pointer table contains pointers to function descriptors
            // r2 will be set to globals_base + 0x100, so -72 = globals_base + 0xB8
            let func_table_offset = 0xB8;
            BigEndian::write_u32(&mut ram[globals_base + func_table_offset..], func_ptr_table as u32);
            
            tracing::info!("  Mac ROM globals at 0x{:08X}, r2 will be 0x{:08X}", globals_base, globals_base + 0x100);
            tracing::info!("  Globals[0xB8] -> 0x{:08X} (function pointer table)", func_ptr_table);
        }
        
        tracing::info!("  Stub function code at 0x{:08X} (blr)", stub_addr);
        tracing::info!("  OpenFirmware client interface stub at 0x{:08X} (returns -1)", of_client_addr);
        tracing::info!("  Function pointer table at 0x{:08X} -> 0x{:08X} (descriptor)", func_ptr_table, func_descriptor);
        tracing::info!("  Function descriptor at 0x{:08X}: [func=0x{:08X}, toc=0x5100, env=0]", 
            func_descriptor, stub_addr);
    }
    
    /// Initialize a stack frame with a return address
    pub fn init_stack_frame(&self, stack_addr: u32, return_addr: u32) {
        let mut ram = self.ram_mmap.write();
        let ram = ram.as_mut();
        let addr = stack_addr as usize;
        
        // Initialize stack frames in both directions to handle any growth pattern
        // Cover 8KB total (4KB down, 4KB up) to handle deep nesting
        for offset in (0..4096).step_by(16) {
            // Frames below initial SP (normal stack growth downward)
            if addr >= offset && addr - offset + 12 < ram.len() {
                let frame_addr = addr - offset;
                BigEndian::write_u32(&mut ram[frame_addr..], 0);
                BigEndian::write_u32(&mut ram[frame_addr + 8..], return_addr);
            }
            
            // Frames above initial SP (for epilogue/deallocation)
            if addr + offset + 12 < ram.len() {
                let frame_addr = addr + offset;
                BigEndian::write_u32(&mut ram[frame_addr..], 0);
                BigEndian::write_u32(&mut ram[frame_addr + 8..], return_addr);
            }
        }
        
        tracing::info!("Initialized stack frames at 0x{:08X}-0x{:08X} with return to 0x{:08X}", 
                       stack_addr.saturating_sub(4096), stack_addr + 4096, return_addr);
    }
}

// Implement MemoryInterface for CPU access
impl MemoryInterface for Memory {
    /// Read a byte from memory
    fn read_u8(&self, addr: u32) -> Result<u8> {
        // Check ROM shadow first (always at 0xFFC00000 base)
        const ROM_SHADOW_BASE: u32 = 0xFFC00000;
        const ROM_SHADOW_END: u32 = 0xFFFFFFFF;
        
        if addr >= ROM_SHADOW_BASE && addr <= ROM_SHADOW_END {
            if let Some(shadow) = &self.rom_shadow {
                let offset = (addr - ROM_SHADOW_BASE) as usize;
                let shadow_buf = shadow.read();
                if offset < shadow_buf.len() {
                    let value = shadow_buf[offset];
                    // Log ALL reads in the first 64KB of ROM shadow
                    if offset < 0x10000 {
                        tracing::info!("ROM shadow read: 0x{:08X} (offset 0x{:X}) = 0x{:02X}", addr, offset, value);
                    }
                    return Ok(value);
                }
            }
        }
        
        // Check ROM range (for NewWorld ROMs with different base)
        if let Some(rom) = &self.rom {
            if let Some(offset) = rom.address_to_offset(addr) {
                return Ok(rom.read_u8(offset));
            }
        }

        // Check MMIO devices
        for (&base, (size, device)) in &self.mmio_devices {
            let offset_opt = addr.checked_sub(base);
            if let Some(offset) = offset_opt {
                if offset < *size {
                    return Ok(device.read().read(offset, 1)? as u8);
                }
            }
        }

        // RAM access - use read lock for shared access
        let ram = self.ram_mmap.read();
        if (addr as usize) < ram.len() {
            Ok(ram[addr as usize])
        } else {
            // Unmapped read - return 0
            if addr >= 0x80000000 {
                tracing::trace!("Unmapped I/O read: 0x{:08X} -> 0x00", addr);
            } else {
                tracing::debug!("Unexpected unmapped read: 0x{:08X} -> 0x00", addr);
            }
            Ok(0)
        }
    }

    /// Read a 16-bit word from memory (big-endian)
    fn read_u16(&self, addr: u32) -> Result<u16> {
        let b0 = self.read_u8(addr)?;
        let b1 = self.read_u8(addr + 1)?;
        Ok(BigEndian::read_u16(&[b0, b1]))
    }

    /// Read a 32-bit word from memory (big-endian)
    fn read_u32(&self, addr: u32) -> Result<u32> {
        // Check ROM shadow first (always at 0xFFC00000 base)
        const ROM_SHADOW_BASE: u32 = 0xFFC00000;
        const ROM_SHADOW_END: u32 = 0xFFFFFFFF;
        
        if addr >= ROM_SHADOW_BASE && addr <= ROM_SHADOW_END - 3 {
            if let Some(shadow) = &self.rom_shadow {
                let offset = (addr - ROM_SHADOW_BASE) as usize;
                let shadow_buf = shadow.read();
                if offset + 4 <= shadow_buf.len() {
                    let value = BigEndian::read_u32(&shadow_buf[offset..]);
                    // Log reads that return 0 (potential null pointers)
                    if value == 0 && offset < 0x100000 {
                        tracing::debug!("ROM shadow read: 0x{:08X} -> 0x00000000 (NULL)", addr);
                    }
                    return Ok(value);
                }
            }
        }
        
        // Check MMIO
        for (&base, (size, device)) in &self.mmio_devices {
            let offset_opt = addr.checked_sub(base);
            if let Some(offset) = offset_opt {
                if offset + 3 < *size {  // Need 4 bytes
                    return device.read().read(offset, 4);
                }
            }
        }
        
        // RAM - use read lock for shared access
        let ram = self.ram_mmap.read();
        if (addr as usize) + 4 <= ram.len() {
            let value = BigEndian::read_u32(&ram[addr as usize..]);
            // Log reads from the critical address
            if addr == 0x00100130 {
                tracing::warn!("🔵 READ from 0x00100130: value=0x{:08X}", value);
            }
            // Log reads from addresses loaded from 0x00100130
            if addr == 0x001155DC {
                tracing::warn!("🔵 READ from 0x001155DC (pointed to by 0x00100130): value=0x{:08X}", value);
            }
            // Log reads from function descriptor
            if addr == 0x001155D0 {
                tracing::warn!("🔵 READ from 0x001155D0 (function descriptor): value=0x{:08X}", value);
            }
            // Log reads that return 0 from potentially important regions
            if value == 0 {
                // Log null reads from claimed region or low memory
                if (addr >= 0x00400000 && addr < 0x004C0000) || (addr < 0x00010000) {
                    tracing::debug!("RAM read: 0x{:08X} -> 0x00000000 (NULL)", addr);
                }
            }
            Ok(value)
        } else {
            // Unmapped read - return 0
            if addr >= 0x80000000 {
                tracing::trace!("Unmapped I/O read: 0x{:08X} -> 0x00000000", addr);
            } else {
                tracing::debug!("Unexpected unmapped read: 0x{:08X} -> 0x00000000", addr);
            }
            Ok(0)
        }
    }

    /// Write a byte to memory
    fn write_u8(&self, addr: u32, value: u8) -> Result<()> {
        // Check if this is a ROM shadow address (0xFFC00000-0xFFFFFFFF)
        const ROM_SHADOW_BASE: u32 = 0xFFC00000;
        const ROM_SHADOW_END: u32 = 0xFFFFFFFF;
        
        if addr >= ROM_SHADOW_BASE && addr <= ROM_SHADOW_END {
            if let Some(shadow) = &self.rom_shadow {
                let offset = (addr - ROM_SHADOW_BASE) as usize;
                let mut shadow_buf = shadow.write();
                if offset < shadow_buf.len() {
                    shadow_buf[offset] = value;
                    return Ok(());
                }
            }
            // ROM range but no shadow - silently ignore
            tracing::trace!("ROM write ignored (no shadow): 0x{:08X} <- 0x{:02X}", addr, value);
            return Ok(());
        }
        
        // Check MMIO devices
        for (&base, (size, device)) in &self.mmio_devices {
            let offset_opt = addr.checked_sub(base);
            if let Some(offset) = offset_opt {
                if offset < *size {
                    return device.write().write(offset, 1, value as u32);
                }
            }
        }

        // RAM access - use write lock for exclusive access
        let mut ram = self.ram_mmap.write();
        if (addr as usize) < ram.len() {
            // Track writes to critical region
            if addr >= 0x001155D0 && addr <= 0x001155DF {
                tracing::error!("🔴 BYTE WRITE to 0x{:08X}: value=0x{:02X} <<< ZEROING CRITICAL DATA!", addr, value);
            }
            ram[addr as usize] = value;
            Ok(())
        } else {
            // Unmapped write - log at different levels based on address range
            if addr >= 0x80000000 {
                // I/O space - expected unmapped writes during initialization
                tracing::debug!("Unmapped I/O write: 0x{:08X} <- 0x{:02X}", addr, value);
            } else {
                // Unexpected address range
                tracing::warn!("Unexpected unmapped write: 0x{:08X} <- 0x{:02X}", addr, value);
            }
            Ok(())
        }
    }

    /// Write a 16-bit word to memory (big-endian)
    fn write_u16(&self, addr: u32, value: u16) -> Result<()> {
        let bytes = value.to_be_bytes();
        self.write_u8(addr, bytes[0])?;
        self.write_u8(addr + 1, bytes[1])?;
        Ok(())
    }

    /// Write a 32-bit word to memory (big-endian)
    fn write_u32(&self, addr: u32, value: u32) -> Result<()> {
        // Check if this is a ROM shadow address (0xFFC00000-0xFFFFFFFF)
        const ROM_SHADOW_BASE: u32 = 0xFFC00000;
        const ROM_SHADOW_END: u32 = 0xFFFFFFFF;
        
        if addr >= ROM_SHADOW_BASE && addr <= ROM_SHADOW_END - 3 {
            if let Some(shadow) = &self.rom_shadow {
                let offset = (addr - ROM_SHADOW_BASE) as usize;
                let mut shadow_buf = shadow.write();
                if offset + 4 <= shadow_buf.len() {
                    // Log ROM shadow writes during boot (first 1000 writes)
                    static ROM_WRITE_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
                    let count = ROM_WRITE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if count < 20 {
                        tracing::info!("ROM shadow write #{}: 0x{:08X} <- 0x{:08X}", count + 1, addr, value);
                    } else if count == 20 {
                        tracing::info!("ROM shadow write logging stopped (20+ writes detected)");
                    }
                    
                    BigEndian::write_u32(&mut shadow_buf[offset..], value);
                    return Ok(());
                }
            }
            // ROM range but no shadow - silently ignore
            tracing::trace!("ROM write ignored (no shadow): 0x{:08X} <- 0x{:08X}", addr, value);
            return Ok(());
        }
        
        // Check MMIO
        for (&base, (size, device)) in &self.mmio_devices {
            let offset_opt = addr.checked_sub(base);
            if let Some(offset) = offset_opt {
                if offset + 3 < *size {  // Need 4 bytes
                    return device.write().write(offset, 4, value);
                }
            }
        }

        // RAM - use write lock for exclusive access
        let mut ram = self.ram_mmap.write();
        if (addr as usize) + 4 <= ram.len() {
            // Log writes to critical address 0x00100130
            if addr == 0x00100130 {
                tracing::warn!("🔴 WRITE to 0x00100130: value=0x{:08X}", value);
                tracing::warn!("   This is the critical function pointer address!");
            }
            if addr == 0x001155DC {
                tracing::warn!("🔴 WRITE to 0x001155DC: value=0x{:08X}", value);
                tracing::warn!("   This is the indirect pointer!");
            }
            // Track writes to the entire 0x001155D0-0x001155DC region
            if addr >= 0x001155D0 && addr <= 0x001155E0 {
                tracing::warn!("🔴 WRITE to 0x{:08X} in critical region: value=0x{:08X}", addr, value);
                // Check if this write might overlap 0x001155DC
                if addr == 0x001155D8 || addr == 0x001155DC {
                    tracing::error!("   ⚠️  CRITICAL: This write affects the function pointer at 0x001155DC!");
                }
            }
            
            // Log writes to claimed region (0x00400000-0x004C0000)
            if addr >= 0x00400000 && addr < 0x004C0000 {
                static CLAIMED_WRITE_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
                let count = CLAIMED_WRITE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if count < 20 {
                    tracing::info!("Claimed region write #{}: 0x{:08X} <- 0x{:08X}", count + 1, addr, value);
                } else if count == 20 {
                    tracing::info!("Claimed region write logging stopped (20+ writes detected)");
                }
            }
            
            BigEndian::write_u32(&mut ram[addr as usize..], value);
            Ok(())
        } else {
            // Unmapped write - log at different levels based on address range
            if addr >= 0x80000000 {
                // I/O space - expected unmapped writes during initialization
                tracing::debug!("Unmapped I/O write: 0x{:08X} <- 0x{:08X}", addr, value);
            } else {
                // Unexpected address range
                tracing::warn!("Unmapped write: 0x{:08X} <- 0x{:08X}", addr, value);
            }
            Ok(())
        }
    }

    /// Read a 64-bit word from memory (big-endian)
    fn read_u64(&self, addr: u32) -> Result<u64> {
        // Check ROM (immutable, no lock)
        if let Some(rom) = &self.rom {
            if addr >= rom.base_address() {
                let offset = (addr - rom.base_address()) as usize;
                if offset + 8 <= rom.size() {
                    let b0 = rom.read_u32(offset) as u64;
                    let b1 = rom.read_u32(offset + 4) as u64;
                    return Ok((b0 << 32) | b1);
                }
            }
        }

        // MMIO devices don't typically support 64-bit reads
        // Fall back to two 32-bit reads
        let high = self.read_u32(addr)?;
        let low = self.read_u32(addr + 4)?;
        Ok(((high as u64) << 32) | (low as u64))
    }

    /// Write a 64-bit word to memory (big-endian)
    fn write_u64(&self, addr: u32, value: u64) -> Result<()> {
        // Write as two 32-bit words (big-endian)
        let high = (value >> 32) as u32;
        let low = value as u32;
        self.write_u32(addr, high)?;
        self.write_u32(addr + 4, low)?;
        Ok(())
    }
    
    // Physical memory access for MMU (reading page tables)
    fn read_u32_phys(&self, paddr: u32) -> Result<u32> {
        // Physical memory access - same as virtual but without MMU translation
        // Used by MMU to read page tables
        self.read_u32(paddr)
    }

    fn read_u64_phys(&self, paddr: u32) -> Result<u64> {
        // Physical memory access - same as virtual but without MMU translation
        self.read_u64(paddr)
    }
}

// Implement 68k MemoryInterface trait
impl newton_m68k::MemoryInterface for Memory {
    fn read_u8(&self, addr: u32) -> Result<u8> {
        <Self as newton_cpu::MemoryInterface>::read_u8(self, addr)
    }
    
    fn read_u16(&self, addr: u32) -> Result<u16> {
        <Self as newton_cpu::MemoryInterface>::read_u16(self, addr)
    }
    
    fn read_u32(&self, addr: u32) -> Result<u32> {
        <Self as newton_cpu::MemoryInterface>::read_u32(self, addr)
    }
    
    fn write_u8(&self, addr: u32, value: u8) -> Result<()> {
        <Self as newton_cpu::MemoryInterface>::write_u8(self, addr, value)
    }
    
    fn write_u16(&self, addr: u32, value: u16) -> Result<()> {
        <Self as newton_cpu::MemoryInterface>::write_u16(self, addr, value)
    }
    
    fn write_u32(&self, addr: u32, value: u32) -> Result<()> {
        <Self as newton_cpu::MemoryInterface>::write_u32(self, addr, value)
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        // Clean up memory-mapped file
        tracing::info!("Cleaning up memory-mapped RAM file: {}", self.ram_path.display());
        if let Err(e) = std::fs::remove_file(&self.ram_path) {
            tracing::warn!("Failed to remove RAM file {}: {}", self.ram_path.display(), e);
        }
    }
}
