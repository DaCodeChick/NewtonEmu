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
pub mod trace;

pub use registers::{Registers, PpcModel};
pub use interpreter::Interpreter;
pub use decoder::{Instruction, decode_instruction};
pub use jit::{JitCompiler, JitStats};
pub use exec_result::ExecResult;
pub use exceptions::{Exception, take_exception};
pub use mmu::Mmu;
pub use trace::{TraceBuffer, TraceEntry};

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
    
    // Physical memory access for MMU (reading page tables)
    fn read_u32_phys(&self, paddr: u32) -> Result<u32>;
    fn read_u64_phys(&self, paddr: u32) -> Result<u64>;
}

/// Physical memory trait for MMU to read page tables
/// 
/// This is automatically implemented for any type that implements MemoryInterface.
pub trait PhysicalMemory {
    fn read_u32_phys(&self, paddr: u32) -> Result<u32>;
    fn read_u64_phys(&self, paddr: u32) -> Result<u64>;
}

// Blanket implementation: any MemoryInterface is also PhysicalMemory
impl<T: MemoryInterface + ?Sized> PhysicalMemory for T {
    fn read_u32_phys(&self, paddr: u32) -> Result<u32> {
        MemoryInterface::read_u32_phys(self, paddr)
    }
    
    fn read_u64_phys(&self, paddr: u32) -> Result<u64> {
        MemoryInterface::read_u64_phys(self, paddr)
    }
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
    
    /// Execution trace buffer
    pub trace_buffer: TraceBuffer,
}

impl Cpu {
    /// Create a new PowerPC CPU
    pub fn new(model: PpcModel) -> Self {
        Self {
            registers: Registers::new(model),
            interpreter: Interpreter::new(),
            jit: None,
            execution_mode: ExecutionMode::Interpreter,
            trace_buffer: TraceBuffer::new(200), // Keep last 200 instructions
        }
    }
    
    /// Create a new PowerPC CPU with JIT enabled
    pub fn new_with_jit(model: PpcModel) -> Result<Self> {
        Ok(Self {
            registers: Registers::new(model),
            interpreter: Interpreter::new(),
            jit: Some(JitCompiler::new()?),
            execution_mode: ExecutionMode::Adaptive,
            trace_buffer: TraceBuffer::new(100),
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
        
        // Debug the function that calls bzero (around 0x00203B68)
        if pc >= 0x00203B00 && pc <= 0x00203B80 {
            let instruction = memory.read_u32(pc)?;
            tracing::warn!("Caller function at PC=0x{:08X} [0x{:08X}]: r3=0x{:08X} r4=0x{:08X} r5=0x{:08X} LR=0x{:08X}",
                          pc, instruction, self.registers.gpr[3], self.registers.gpr[4], self.registers.gpr[5], self.registers.lr);
        }
        
        // Debug bzero function entry - log all parameters
        if pc == 0x0020A7D8 || pc == 0x0020A7DC {
            tracing::error!("bzero function at PC=0x{:08X}: r3=0x{:08X} r4=0x{:08X} r5=0x{:08X} r11=0x{:08X} r12=0x{:08X}",
                          pc, self.registers.gpr[3], self.registers.gpr[4], self.registers.gpr[5],
                          self.registers.gpr[11], self.registers.gpr[12]);
        }
        
        // Debug critical instructions around the crash point
        if pc >= 0x0020A250 && pc <= 0x0020A260 {
            tracing::info!("Critical section at PC=0x{:08X}: r2=0x{:08X} r4=0x{:08X} r12=0x{:08X}",
                          pc, self.registers.gpr[2], self.registers.gpr[4], self.registers.gpr[12]);
        }
        if pc >= 0x0020C490 && pc <= 0x0020C4A8 {
            tracing::info!("Call stub at PC=0x{:08X}: r0=0x{:08X} r12=0x{:08X} CTR=0x{:08X}",
                          pc, self.registers.gpr[0], self.registers.gpr[12], self.registers.ctr);
        }
        
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
        
        // Record trace entry if enabled
        if self.trace_buffer.is_enabled() {
            self.trace_buffer.record(TraceEntry {
                pc,
                instruction: instr_word,
                lr: self.registers.lr,
                sp: self.registers.gpr[1],
                r2: self.registers.gpr[2],
                cr: self.registers.cr.bits(),
            });
        }
        
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
    pub fn translate_data_address(&mut self, vaddr: u32, is_write: bool, memory: &dyn MemoryInterface) -> Result<u32> {
        self.registers.mmu.translate_data(
            vaddr,
            &self.registers.sr,
            self.registers.msr.bits(),
            is_write,
            memory
        )
    }
    
    /// Translate virtual address for instruction fetch using MMU
    pub fn translate_instruction_address(&mut self, vaddr: u32, memory: &dyn MemoryInterface) -> Result<u32> {
        self.registers.mmu.translate_instruction(
            vaddr,
            &self.registers.sr,
            self.registers.msr.bits(),
            memory
        )
    }
    
    /// Set up a BAT register for memory mapping
    /// 
    /// # Arguments
    /// * `index` - BAT register number (0-3)
    /// * `is_data` - true for DBAT, false for IBAT
    /// * `vaddr` - Virtual address base (must be block-aligned)
    /// * `paddr` - Physical address base (must be block-aligned)
    /// * `size` - Block size in bytes (must be power of 2: 128KB to 256MB)
    /// * `writable` - Allow write access
    /// * `supervisor` - Valid in supervisor mode
    /// * `user` - Valid in user mode
    pub fn setup_bat(&mut self, 
                     index: usize, 
                     is_data: bool, 
                     vaddr: u32, 
                     paddr: u32, 
                     size: u32,
                     writable: bool,
                     supervisor: bool,
                     user: bool) -> Result<()> {
        if index >= 4 {
            return Err(newton_utils::Error::Cpu(format!("BAT index {} out of range (0-3)", index)));
        }
        
        // Calculate block length (BL) field
        // size = 128KB * (BL + 1), so BL = (size / 128KB) - 1
        let bl = (size / (128 * 1024)).checked_sub(1)
            .ok_or_else(|| newton_utils::Error::Cpu(format!("BAT size {} too small (minimum 128KB)", size)))?;
        
        if bl > 0x7FF {
            return Err(newton_utils::Error::Cpu(format!("BAT size {} too large (maximum 256MB)", size)));
        }
        
        // Build BATU (upper register)
        let bepi = vaddr & 0xFFFE_0000;  // Block Effective Page Index
        let vs = if supervisor { 0x2 } else { 0 };  // Supervisor valid bit
        let vp = if user { 0x1 } else { 0 };         // User valid bit
        let upper = bepi | (bl << 2) | vs | vp;
        
        // Build BATL (lower register)
        let brpn = paddr & 0xFFFE_0000;  // Block Real Page Number
        let wimg = 0x02;  // WIMG bits: 0010 = cache-inhibited (typical for I/O)
        let pp = if writable { 0x2 } else { 0x1 };  // Page protection: 10=R/W, 01=R/O
        let lower = brpn | wimg | pp;
        
        // Set the BAT register
        let bat_array = if is_data {
            &mut self.registers.mmu.dbat
        } else {
            &mut self.registers.mmu.ibat
        };
        
        bat_array[index].upper = upper;
        bat_array[index].lower = lower;
        
        tracing::info!("Set up {}BAT{}: vaddr=0x{:08X}, paddr=0x{:08X}, size=0x{:X}, writable={}, super={}, user={}",
                      if is_data { "D" } else { "I" },
                      index,
                      vaddr,
                      paddr,
                      size,
                      writable,
                      supervisor,
                      user);
        
        Ok(())
    }
}

// SAFETY: Cpu can be Send as long as JIT is not enabled.
// When using Cpu in a separate thread (via CpuThread), the CPU must be created
// without JIT (using Cpu::new(), not Cpu::new_with_jit()).
// The JITModule is not Send, but Option<JITModule> is Send when None.
// Users must ensure JIT is disabled before sending Cpu across threads.
unsafe impl Send for Cpu {}
