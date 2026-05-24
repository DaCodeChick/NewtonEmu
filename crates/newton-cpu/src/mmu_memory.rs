// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! MMU-aware memory access wrapper
//!
//! This module provides memory access functions that perform MMU translation.
//! MMU faults are returned as Error::Memory and should be converted to exceptions
//! by the caller.

use crate::registers::Registers;
use crate::{MemoryInterface, PhysicalMemory};
use newton_utils::Result;

/// Translate a data address using the MMU
/// 
/// Returns the physical address, or Error::Memory if translation fails.
/// The caller should convert memory errors to DSI exceptions.
fn translate_data_address<M>(
    regs: &mut Registers,
    vaddr: u32,
    is_write: bool,
    memory: &M,
) -> Result<u32>
where
    M: PhysicalMemory,
{
    // Check if data translation is enabled (MSR[DR])
    let msr_dr = (regs.msr.bits() & 0x0010) != 0;
    
    if !msr_dr {
        // Translation disabled - use physical address directly
        return Ok(vaddr);
    }
    
    // Translate via MMU
    regs.mmu.translate_data(vaddr, &regs.sr, regs.msr.bits(), is_write, memory)
}

/// Read u8 with MMU translation
pub fn read_u8<M>(
    regs: &mut Registers,
    memory: &M,
    vaddr: u32,
) -> Result<u8>
where
    M: MemoryInterface + PhysicalMemory,
{
    let paddr = translate_data_address(regs, vaddr, false, memory)?;
    memory.read_u8(paddr)
}

/// Read u16 with MMU translation
pub fn read_u16<M>(
    regs: &mut Registers,
    memory: &M,
    vaddr: u32,
) -> Result<u16>
where
    M: MemoryInterface + PhysicalMemory,
{
    let paddr = translate_data_address(regs, vaddr, false, memory)?;
    memory.read_u16(paddr)
}

/// Read u32 with MMU translation
pub fn read_u32<M>(
    regs: &mut Registers,
    memory: &M,
    vaddr: u32,
) -> Result<u32>
where
    M: MemoryInterface + PhysicalMemory,
{
    let paddr = translate_data_address(regs, vaddr, false, memory)?;
    memory.read_u32(paddr)
}

/// Read u64 with MMU translation
pub fn read_u64<M>(
    regs: &mut Registers,
    memory: &M,
    vaddr: u32,
) -> Result<u64>
where
    M: MemoryInterface + PhysicalMemory,
{
    let paddr = translate_data_address(regs, vaddr, false, memory)?;
    memory.read_u64(paddr)
}

/// Write u8 with MMU translation
pub fn write_u8<M>(
    regs: &mut Registers,
    memory: &M,
    vaddr: u32,
    value: u8,
) -> Result<()>
where
    M: MemoryInterface + PhysicalMemory,
{
    let paddr = translate_data_address(regs, vaddr, true, memory)?;
    memory.write_u8(paddr, value)
}

/// Write u16 with MMU translation
pub fn write_u16<M>(
    regs: &mut Registers,
    memory: &M,
    vaddr: u32,
    value: u16,
) -> Result<()>
where
    M: MemoryInterface + PhysicalMemory,
{
    let paddr = translate_data_address(regs, vaddr, true, memory)?;
    memory.write_u16(paddr, value)
}

/// Write u32 with MMU translation
pub fn write_u32<M>(
    regs: &mut Registers,
    memory: &M,
    vaddr: u32,
    value: u32,
) -> Result<()>
where
    M: MemoryInterface + PhysicalMemory,
{
    let paddr = translate_data_address(regs, vaddr, true, memory)?;
    memory.write_u32(paddr, value)
}

/// Write u64 with MMU translation
pub fn write_u64<M>(
    regs: &mut Registers,
    memory: &M,
    vaddr: u32,
    value: u64,
) -> Result<()>
where
    M: MemoryInterface + PhysicalMemory,
{
    let paddr = translate_data_address(regs, vaddr, true, memory)?;
    memory.write_u64(paddr, value)
}
