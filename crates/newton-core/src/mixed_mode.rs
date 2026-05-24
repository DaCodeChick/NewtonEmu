// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Mixed Mode Manager
//!
//! Handles transitions between PowerPC and 68k code execution.
//!
//! In Mac OS 9, the operating system kernel runs as PowerPC native code,
//! but the Toolbox (UI framework, file I/O, etc.) is 68k code. The Mixed
//! Mode Manager handles the transitions between these two execution modes.

use newton_cpu::Cpu;
use newton_m68k::M68k;
use crate::memory::Memory;
use newton_utils::Result;

/// Mixed Mode Manager
///
/// Handles transitions between PowerPC and 68k code
pub struct MixedModeManager {
    /// Is currently executing 68k code?
    in_68k_mode: bool,
}

impl MixedModeManager {
    /// Create a new Mixed Mode Manager
    pub fn new() -> Self {
        Self {
            in_68k_mode: false,
        }
    }
    
    /// Check if we're currently in 68k mode
    pub fn is_68k_mode(&self) -> bool {
        self.in_68k_mode
    }
    
    /// Check if an address is in 68k code space
    ///
    /// 68k Toolbox code is typically in the range 0x40800000-0x4FFFFFFF
    /// or in the ROM at 0xFFC00000-0xFFFFFFFF
    pub fn is_68k_address(addr: u32) -> bool {
        // Check for 68k Toolbox space
        if addr >= 0x40800000 && addr < 0x50000000 {
            return true;
        }
        
        // Check for ROM space (where decompressed 68k ROM lives)
        if addr >= 0xFFC00000 {
            return true;
        }
        
        false
    }
    
    /// Switch from PowerPC to 68k execution
    ///
    /// This is called when PowerPC code jumps to a 68k address
    pub fn switch_to_68k(
        &mut self,
        ppc_cpu: &mut Cpu,
        m68k_cpu: &mut M68k,
        memory: &Memory,
        target_addr: u32,
    ) -> Result<()> {
        tracing::debug!("Mixed Mode: Switching to 68k at 0x{:08X}", target_addr);
        
        // Save PowerPC state (link register, stack pointer, etc.)
        // We'll need to restore these when we return
        
        // Set up 68k CPU state
        // PC = target address
        m68k_cpu.registers.pc = target_addr;
        
        // Copy parameters from PowerPC registers to 68k stack
        // (this depends on the calling convention - needs more work)
        
        self.in_68k_mode = true;
        Ok(())
    }
    
    /// Switch from 68k to PowerPC execution
    ///
    /// This is called when 68k code returns (RTS) or calls back to PowerPC
    pub fn switch_to_ppc(
        &mut self,
        ppc_cpu: &mut Cpu,
        m68k_cpu: &M68k,
        memory: &Memory,
    ) -> Result<()> {
        tracing::debug!("Mixed Mode: Switching to PowerPC");
        
        // Copy return value from 68k D0 to PowerPC r3
        // ppc_cpu.registers.gpr[3] = m68k_cpu.registers.d[0];
        
        // Restore PowerPC state (link register, stack pointer, etc.)
        
        self.in_68k_mode = false;
        Ok(())
    }
    
    /// Execute one step in the current mode
    ///
    /// This dispatches to either PPC or 68k execution based on current mode
    pub fn step(
        &mut self,
        ppc_cpu: &mut Cpu,
        m68k_cpu: &mut M68k,
        memory: &Memory,
    ) -> Result<()> {
        if self.in_68k_mode {
            // Execute one 68k instruction
            m68k_cpu.step(memory)?;
            
            // Check if we've returned from 68k code
            // (This is simplified - real implementation needs to track call depth)
            // if m68k returned {
            //     self.switch_to_ppc(ppc_cpu, m68k_cpu, memory)?;
            // }
        } else {
            // Execute one PPC instruction
            ppc_cpu.step(memory)?;
            
            // Check if PPC code is jumping to 68k address
            let pc = ppc_cpu.registers.pc;
            if Self::is_68k_address(pc) {
                self.switch_to_68k(ppc_cpu, m68k_cpu, memory, pc)?;
            }
        }
        
        Ok(())
    }
}

impl Default for MixedModeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_68k_address() {
        // Toolbox space
        assert!(MixedModeManager::is_68k_address(0x40800000));
        assert!(MixedModeManager::is_68k_address(0x45000000));
        assert!(MixedModeManager::is_68k_address(0x4FFFFFFF));
        
        // ROM space
        assert!(MixedModeManager::is_68k_address(0xFFC00000));
        assert!(MixedModeManager::is_68k_address(0xFFFFFFFF));
        
        // Not 68k space
        assert!(!MixedModeManager::is_68k_address(0x00000000));
        assert!(!MixedModeManager::is_68k_address(0x00400000));
        assert!(!MixedModeManager::is_68k_address(0x10000000));
    }
}
