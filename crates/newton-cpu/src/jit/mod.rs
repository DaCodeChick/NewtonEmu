// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! JIT compiler using Cranelift
//!
//! This module provides dynamic recompilation of PowerPC code to native code
//! for significant performance improvements.

mod block;
mod translator;
mod cache;
mod memory;

pub use block::{BasicBlock, BlockBuilder};
pub use cache::CodeCache;
pub use memory::{JitContext, JitRegisters};

use crate::registers::Registers;
use crate::MemoryInterface;
use newton_utils::Result;

use cranelift_codegen::settings::{self, Configurable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::FuncId;

use std::collections::HashMap;

/// Threshold for hot code detection
const HOT_THRESHOLD: u32 = 100;

/// JIT compiler for PowerPC to native code
pub struct JitCompiler {
    /// Cranelift JIT module
    module: JITModule,
    
    /// Code cache for compiled blocks
    cache: CodeCache,
    
    /// Execution counter for hot code detection
    exec_counters: HashMap<u32, u32>,
    
    /// Compiled function IDs
    compiled_func_ids: HashMap<u32, FuncId>,
    
    /// Compiled function pointers (native code)
    compiled_functions: HashMap<u32, *const u8>,
}

impl JitCompiler {
    /// Create a new JIT compiler
    pub fn new() -> Result<Self> {
        // Configure Cranelift
        let mut flag_builder = settings::builder();
        flag_builder.set("opt_level", "speed").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder()
            .map_err(|e| newton_utils::Error::Cpu(format!("Failed to create ISA builder: {}", e)))?;
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| newton_utils::Error::Cpu(format!("Failed to create ISA: {}", e)))?;
        
        // Create JIT module with symbol lookup
        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        
        // Register memory callback symbols
        builder.symbol("jit_memory_read_u32", memory::jit_memory_read_u32 as *const u8);
        builder.symbol("jit_memory_write_u32", memory::jit_memory_write_u32 as *const u8);
        builder.symbol("jit_memory_read_u16", memory::jit_memory_read_u16 as *const u8);
        builder.symbol("jit_memory_write_u16", memory::jit_memory_write_u16 as *const u8);
        builder.symbol("jit_memory_read_u8", memory::jit_memory_read_u8 as *const u8);
        builder.symbol("jit_memory_write_u8", memory::jit_memory_write_u8 as *const u8);
        
        let module = JITModule::new(builder);
        
        tracing::info!("JIT compiler initialized with Cranelift");
        
        Ok(Self {
            module,
            cache: CodeCache::new(),
            exec_counters: HashMap::new(),
            compiled_func_ids: HashMap::new(),
            compiled_functions: HashMap::new(),
        })
    }
    
    /// Check if a block should be compiled (hot code detection)
    pub fn should_compile(&mut self, addr: u32) -> bool {
        let counter = self.exec_counters.entry(addr).or_insert(0);
        *counter += 1;
        
        // Compile when execution count exceeds threshold
        if *counter >= HOT_THRESHOLD && !self.compiled_functions.contains_key(&addr) {
            tracing::debug!("Hot code detected at 0x{:08X} (executed {} times)", addr, counter);
            true
        } else {
            false
        }
    }
    
    /// Check if a block is already compiled
    pub fn is_compiled(&self, addr: u32) -> bool {
        self.compiled_functions.contains_key(&addr)
    }
    
    /// Compile a basic block at the given address
    pub fn compile_block(&mut self, start_addr: u32, memory: &dyn MemoryInterface) -> Result<()> {
        tracing::debug!("Compiling basic block at 0x{:08X}", start_addr);
        
        // Build basic block
        let block = BlockBuilder::build_from_memory(start_addr, memory)?;
        
        // Translate to Cranelift IR and compile
        let func_id = translator::translate_block(&block, &mut self.module)?;
        
        // Finalize the function (link it to native code)
        self.module.finalize_definitions()
            .map_err(|e| newton_utils::Error::Cpu(format!("Failed to finalize function: {}", e)))?;
        
        // Get the function pointer
        let code_ptr = self.module.get_finalized_function(func_id);
        
        // Store the function ID and pointer
        self.compiled_func_ids.insert(start_addr, func_id);
        self.compiled_functions.insert(start_addr, code_ptr);
        
        // Cache the compiled block
        self.cache.insert(start_addr, block);
        
        tracing::info!("Successfully compiled block at 0x{:08X} ({} instructions, code @ {:p})", 
                      start_addr, self.cache.get_block(start_addr).unwrap().instructions.len(), code_ptr);
        
        Ok(())
    }
    
    /// Execute a compiled block if available
    pub fn try_execute_compiled(&self, addr: u32, regs: &mut Registers, memory: &mut dyn MemoryInterface) -> Option<Result<()>> {
        // Check if we have a compiled version
        let code_ptr = self.compiled_functions.get(&addr)?;
        
        tracing::trace!("Executing compiled code at 0x{:08X} (code @ {:p})", addr, code_ptr);
        
        // Cast function pointer to the correct signature: fn(*mut JitContext) -> u32
        // SAFETY: We trust Cranelift generated safe code and the signature matches
        let func: unsafe extern "C" fn(*mut memory::JitContext) -> u32 = unsafe {
            std::mem::transmute(*code_ptr)
        };
        
        // Create JIT register state from CPU registers
        let mut jit_regs = memory::JitRegisters::from_registers(regs);
        
        // Create JIT context for memory callbacks and register access
        let mut ctx = memory::JitContext::new(memory, &mut jit_regs);
        
        // Call the compiled code with context pointer
        let new_pc = unsafe {
            func(&mut ctx as *mut memory::JitContext)
        };
        
        // Write register state back to CPU
        jit_regs.write_to_registers(regs);
        
        // Update PC from return value
        regs.pc = new_pc;
        
        Some(Ok(()))
    }
    
    /// Get statistics about JIT compilation
    pub fn stats(&self) -> JitStats {
        JitStats {
            compiled_blocks: self.cache.len(),
            total_instructions: self.cache.total_instructions(),
            cache_size_bytes: self.cache.estimated_size(),
        }
    }
    
    /// Clear the JIT cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.compiled_func_ids.clear();
        self.compiled_functions.clear();
        self.exec_counters.clear();
        tracing::info!("JIT cache cleared");
    }
}

impl Default for JitCompiler {
    fn default() -> Self {
        Self::new().expect("Failed to create JIT compiler")
    }
}

/// JIT compiler statistics
#[derive(Debug, Clone)]
pub struct JitStats {
    /// Number of compiled blocks
    pub compiled_blocks: usize,
    /// Total instructions in cache
    pub total_instructions: usize,
    /// Estimated cache size in bytes
    pub cache_size_bytes: usize,
}
