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

pub use registers::{Registers, PpcModel};
pub use interpreter::Interpreter;
pub use decoder::{Instruction, decode_instruction};
pub use jit::{JitCompiler, JitStats};
pub use exec_result::ExecResult;

use newton_utils::Result;

/// Memory interface trait for CPU
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
        // Fetch instruction from memory at PC
        let instr_word = memory.read_u32(self.registers.pc)?;
        
        // Decode instruction
        let instr = decode_instruction(instr_word)?;
        
        // Execute instruction and get result
        let exec_result = self.interpreter.execute_with_memory(instr, &mut self.registers, memory)?;
        
        // Only advance PC if instruction didn't modify it
        if exec_result.should_advance_pc() {
            self.registers.pc = self.registers.pc.wrapping_add(4);
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
}

// SAFETY: Cpu can be Send as long as JIT is not enabled.
// When using Cpu in a separate thread (via CpuThread), the CPU must be created
// without JIT (using Cpu::new(), not Cpu::new_with_jit()).
// The JITModule is not Send, but Option<JITModule> is Send when None.
// Users must ensure JIT is disabled before sending Cpu across threads.
unsafe impl Send for Cpu {}
