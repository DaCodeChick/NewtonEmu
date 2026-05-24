// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! PowerPC CPU emulation
//!
//! This crate implements the PowerPC G4 (7400/7450) CPU, including:
//! - Integer and floating-point instructions
//! - AltiVec SIMD instructions
//! - Supervisor mode and privilege levels
//! - Interpreter and JIT compiler

pub mod altivec;
pub mod decoder;
pub mod interpreter;
pub mod jit;
pub mod registers;
pub mod exec_result;
pub mod exceptions;
pub mod mmu;
pub mod mmu_memory;

pub use registers::{Registers, PpcModel};
pub use interpreter::Interpreter;
pub use decoder::{Instruction, decode_instruction};
pub use jit::{JitCompiler, JitStats};
pub use exec_result::ExecResult;
pub use exceptions::{Exception, take_exception};
pub use mmu::Mmu;

use newton_utils::{Result, Error};

/// Memory interface trait for CPU (virtual addresses)
/// 
/// This allows the CPU to access memory without depending on newton-core directly.
/// All methods use &self to allow thread-safe implementations with interior mutability.
pub trait MemoryInterface {
    fn read_u8(&self, addr: u32) -> Result<u8>;
    fn read_u16(&self, addr: u32) -> Result<u16>;
    fn read_u32(&self, addr: u32) -> Result<u32>;
    fn read_u64(&self, addr: u32) -> Result<u64>;
    fn write_u8(&self, addr: u32, value: u8) -> Result<()>;
    fn write_u16(&self, addr: u32, value: u16) -> Result<()>;
    fn write_u32(&self, addr: u32, value: u32) -> Result<()>;
    fn write_u64(&self, addr: u32, value: u64) -> Result<()>;
}

/// Physical memory interface for MMU (physical addresses only, no translation)
/// 
/// This is used by the MMU to read page tables from physical memory.
pub trait PhysicalMemory {
    fn read_u32_phys(&self, paddr: u32) -> Result<u32>;
    fn read_u64_phys(&self, paddr: u32) -> Result<u64>;
}

/// Execution mode for CPU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Pure interpreter (slower, always available)
    Interpreter,
    /// JIT compilation (faster, compiles hot code)
    Jit,
    /// Adaptive (starts with interpreter, compiles hot code)
    Adaptive,
}

/// PowerPC CPU state and execution
pub struct Cpu {
    /// CPU registers
    pub registers: Registers,
    
    /// Interpreter for instruction execution
    interpreter: Interpreter,
    
    /// JIT compiler
    jit: Option<JitCompiler>,
    
    /// Execution mode
    execution_mode: ExecutionMode,
}

impl Cpu {
    /// Create a new PowerPC CPU
    pub fn new(model: PpcModel) -> Self {
        Self {
            registers: Registers::new(model),
            interpreter: Interpreter::new(),
            jit: None,
            execution_mode: ExecutionMode::Interpreter,
        }
    }
    
    /// Create a new PowerPC CPU with JIT enabled
    pub fn new_with_jit(model: PpcModel) -> Result<Self> {
        Ok(Self {
            registers: Registers::new(model),
            interpreter: Interpreter::new(),
            jit: Some(JitCompiler::new()?),
            execution_mode: ExecutionMode::Adaptive,
        })
    }

    /// Reset the CPU to initial state
    pub fn reset(&mut self) {
        self.registers.reset();
        if let Some(jit) = &mut self.jit {
            jit.clear_cache();
        }
    }

    /// Execute a single instruction with memory access
    pub fn step(&mut self, memory: &dyn MemoryInterface) -> Result<()> {
        let pc = self.registers.pc;
        
        // Try JIT execution if enabled
        if let Some(jit) = &mut self.jit {
            match self.execution_mode {
                ExecutionMode::Jit => {
                    // Always try to use JIT
                    if !jit.is_compiled(pc) {
                        jit.compile_block(pc, memory)?;
                    }
                    
                    if let Some(result) = jit.try_execute_compiled(pc, &mut self.registers, memory) {
                        return result;
                    }
                }
                ExecutionMode::Adaptive => {
                    // Check if this is hot code that should be compiled
                    if jit.should_compile(pc) {
                        if let Err(e) = jit.compile_block(pc, memory) {
                            tracing::warn!("JIT compilation failed at 0x{:08X}: {}", pc, e);
                        }
                    }
                    
                    // Try to execute compiled version
                    if let Some(result) = jit.try_execute_compiled(pc, &mut self.registers, memory) {
                        return result;
                    }
                }
                ExecutionMode::Interpreter => {
                    // Fall through to interpreter
                }
            }
        }
        
        // Fallback to interpreter
        self.step_interpreter(memory)
    }
    
    /// Execute using interpreter only
    fn step_interpreter(&mut self, memory: &dyn MemoryInterface) -> Result<()> {
        let pc = self.registers.pc;
        
        // Fetch instruction from memory at PC
        let instr_word = match memory.read_u32(pc) {
            Ok(word) => word,
            Err(Error::Memory(msg)) if msg.contains("Page fault") || msg.contains("Protection") || msg.contains("Direct-store") => {
                // MMU fault during instruction fetch - generate ISI
                let srr1_bits = if msg.contains("Page fault") {
                    exceptions::isi_srr1::PAGE_FAULT
                } else if msg.contains("Protection") {
                    exceptions::isi_srr1::PROTECTION
                } else {
                    exceptions::isi_srr1::DIRECT_STORE
                };
                exceptions::take_exception(
                    &mut self.registers,
                    Exception::InstructionStorage { srr1_bits },
                    pc
                )?;
                return Ok(());
            }
            Err(e) => return Err(e),
        };
        
        // Decode instruction
        let instr = decode_instruction(instr_word)?;
        
        // Execute instruction and get result
        let exec_result = match self.interpreter.execute_with_memory(instr, &mut self.registers, memory) {
            Ok(result) => result,
            Err(Error::Memory(msg)) if msg.contains("Page fault") || msg.contains("Protection") || msg.contains("Direct-store") => {
                // MMU fault during data access - generate DSI
                // Extract the faulting address from the error message
                let vaddr = if let Some(start) = msg.find("0x") {
                    let addr_str = &msg[start+2..];
                    let end = addr_str.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(addr_str.len());
                    u32::from_str_radix(&addr_str[..end], 16).unwrap_or(0)
                } else {
                    0  // Couldn't parse address
                };
                
                let dsisr_val = if msg.contains("Page fault") {
                    let mut val = exceptions::dsisr::PAGE_FAULT;
                    // TODO: Detect if this was a store operation from the instruction
                    val
                } else if msg.contains("Protection") {
                    let mut val = exceptions::dsisr::PROTECTION;
                    // TODO: Detect if this was a store operation from the instruction
                    val
                } else {
                    exceptions::dsisr::DIRECT_STORE
                };
                
                let next_pc = pc.wrapping_add(4);
                exceptions::take_exception(
                    &mut self.registers,
                    Exception::DataStorage { dar: vaddr, dsisr: dsisr_val },
                    next_pc
                )?;
                return Ok(());
            }
            Err(e) => return Err(e),
        };
        
        // Handle execution result
        match exec_result {
            ExecResult::Continue | ExecResult::BranchNotTaken => {
                // Normal instruction - advance PC
                self.registers.pc = pc.wrapping_add(4);
            }
            ExecResult::BranchTaken => {
                // Branch already set PC - don't advance
            }
            ExecResult::Syscall => {
                // System call exception
                let next_pc = pc.wrapping_add(4);
                exceptions::take_exception(&mut self.registers, Exception::SystemCall, next_pc)?;
            }
            ExecResult::OpenFirmwareCall => {
                // OF call is handled by the emulator layer
                // This shouldn't normally be returned from the interpreter
                tracing::warn!("OpenFirmwareCall result unexpected - treating as continue");
                self.registers.pc = pc.wrapping_add(4);
            }
            ExecResult::Trap => {
                // Trap exception
                let next_pc = pc.wrapping_add(4);
                exceptions::take_exception(&mut self.registers, Exception::Program { trap: true }, next_pc)?;
            }
        }
        
        Ok(())
    }

    /// Set execution mode
    pub fn set_execution_mode(&mut self, mode: ExecutionMode) {
        self.execution_mode = mode;
        tracing::info!("CPU execution mode set to: {:?}", mode);
    }
    
    /// Get current execution mode
    pub fn execution_mode(&self) -> ExecutionMode {
        self.execution_mode
    }

    /// Enable or disable JIT compilation
    pub fn set_jit_enabled(&mut self, enabled: bool) {
        if enabled && self.jit.is_none() {
            match JitCompiler::new() {
                Ok(jit) => {
                    self.jit = Some(jit);
                    self.execution_mode = ExecutionMode::Adaptive;
                    tracing::info!("JIT compiler enabled");
                }
                Err(e) => {
                    tracing::error!("Failed to enable JIT: {}", e);
                }
            }
        } else if !enabled {
            self.jit = None;
            self.execution_mode = ExecutionMode::Interpreter;
            tracing::info!("JIT compiler disabled");
        }
    }
    
    /// Check if JIT is enabled
    pub fn is_jit_enabled(&self) -> bool {
        self.jit.is_some()
    }
    
    /// Get JIT statistics
    pub fn jit_stats(&self) -> Option<JitStats> {
        self.jit.as_ref().map(|jit| jit.stats())
    }
    
    /// Translate virtual address for data access using MMU
    /// 
    /// Returns the physical address after translation, or the virtual address
    /// if translation is disabled.
    pub fn translate_data_address(&mut self, vaddr: u32, is_write: bool, memory: &dyn PhysicalMemory) -> Result<u32> {
        self.registers.mmu.translate_data(
            vaddr,
            &self.registers.sr,
            self.registers.msr.bits(),
            is_write,
            memory
        )
    }
    
    /// Translate virtual address for instruction fetch using MMU
    pub fn translate_instruction_address(&mut self, vaddr: u32, memory: &dyn PhysicalMemory) -> Result<u32> {
        self.registers.mmu.translate_instruction(
            vaddr,
            &self.registers.sr,
            self.registers.msr.bits(),
            memory
        )
    }
}

// SAFETY: Cpu can be Send as long as JIT is not enabled.
// When using Cpu in a separate thread (via CpuThread), the CPU must be created
// without JIT (using Cpu::new(), not Cpu::new_with_jit()).
// The JITModule is not Send, but Option<JITModule> is Send when None.
// Users must ensure JIT is disabled before sending Cpu across threads.
unsafe impl Send for Cpu {}
