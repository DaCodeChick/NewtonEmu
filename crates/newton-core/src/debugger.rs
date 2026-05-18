// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! System-level debugger for PowerPC guest system
//!
//! This debugger inspects the **emulated PowerPC system** (guest), not the
//! Rust emulator itself (host). Similar to Mesen2's debugger.
//!
//! Features:
//! - Breakpoints (execution, memory read/write)
//! - Step execution (step into, step over, step out)
//! - Register inspection
//! - Memory inspection
//! - Disassembly view
//! - Call stack tracking

use std::collections::HashMap;

/// Breakpoint type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BreakpointType {
    /// Break on instruction execution at address
    Execute,
    /// Break on memory read from address
    MemoryRead,
    /// Break on memory write to address
    MemoryWrite,
    /// Break on memory read or write
    MemoryAccess,
}

/// Breakpoint definition
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Address to break on
    pub address: u32,
    /// Type of breakpoint
    pub bp_type: BreakpointType,
    /// Is this breakpoint enabled?
    pub enabled: bool,
    /// Optional condition (if None, always breaks)
    pub condition: Option<String>,
    /// Hit count (how many times has this breakpoint been hit?)
    pub hit_count: u32,
}

impl Breakpoint {
    pub fn new(address: u32, bp_type: BreakpointType) -> Self {
        Self {
            address,
            bp_type,
            enabled: true,
            condition: None,
            hit_count: 0,
        }
    }
}

/// Execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionState {
    /// Running normally
    Running,
    /// Paused by user
    Paused,
    /// Paused by breakpoint
    BreakpointHit { address: u32 },
    /// Stepping (will pause after next instruction)
    Stepping,
    /// Step over (will pause after current function returns or next instruction)
    SteppingOver,
    /// Step out (will pause when current function returns)
    SteppingOut,
}

/// Call stack frame for tracking function calls
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Return address
    pub return_address: u32,
    /// Stack pointer at call time
    pub stack_pointer: u32,
    /// Function address (if known)
    pub function_address: Option<u32>,
}

/// System debugger state
pub struct Debugger {
    /// Current execution state
    state: ExecutionState,
    
    /// Execution breakpoints (address -> breakpoint)
    exec_breakpoints: HashMap<u32, Breakpoint>,
    
    /// Memory read breakpoints (address -> breakpoint)
    mem_read_breakpoints: HashMap<u32, Breakpoint>,
    
    /// Memory write breakpoints (address -> breakpoint)
    mem_write_breakpoints: HashMap<u32, Breakpoint>,
    
    /// Call stack (for step out)
    call_stack: Vec<StackFrame>,
    
    /// Step over: target PC where we should break
    step_over_target: Option<u32>,
    
    /// Step out: stack depth when step out was initiated
    step_out_depth: Option<usize>,
    
    /// Instruction count since start
    instruction_count: u64,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            state: ExecutionState::Running,
            exec_breakpoints: HashMap::new(),
            mem_read_breakpoints: HashMap::new(),
            mem_write_breakpoints: HashMap::new(),
            call_stack: Vec::new(),
            step_over_target: None,
            step_out_depth: None,
            instruction_count: 0,
        }
    }
    
    /// Get current execution state
    pub fn state(&self) -> ExecutionState {
        self.state
    }
    
    /// Set execution state
    pub fn set_state(&mut self, state: ExecutionState) {
        self.state = state;
    }
    
    /// Check if emulator should pause
    pub fn should_pause(&self) -> bool {
        !matches!(self.state, ExecutionState::Running)
    }
    
    /// Resume execution
    pub fn resume(&mut self) {
        self.state = ExecutionState::Running;
    }
    
    /// Pause execution
    pub fn pause(&mut self) {
        self.state = ExecutionState::Paused;
    }
    
    /// Step one instruction
    pub fn step_into(&mut self) {
        self.state = ExecutionState::Stepping;
    }
    
    /// Step over (execute but break after current function call or next instruction)
    pub fn step_over(&mut self, current_pc: u32) {
        self.step_over_target = Some(current_pc + 4);
        self.state = ExecutionState::SteppingOver;
    }
    
    /// Step out (run until current function returns)
    pub fn step_out(&mut self) {
        self.step_out_depth = Some(self.call_stack.len());
        self.state = ExecutionState::SteppingOut;
    }
    
    /// Add execution breakpoint
    pub fn add_breakpoint(&mut self, address: u32, bp_type: BreakpointType) -> usize {
        let bp = Breakpoint::new(address, bp_type);
        let id = address as usize; // Simple ID scheme
        
        match bp_type {
            BreakpointType::Execute => {
                self.exec_breakpoints.insert(address, bp);
            }
            BreakpointType::MemoryRead => {
                self.mem_read_breakpoints.insert(address, bp);
            }
            BreakpointType::MemoryWrite => {
                self.mem_write_breakpoints.insert(address, bp);
            }
            BreakpointType::MemoryAccess => {
                self.mem_read_breakpoints.insert(address, bp.clone());
                self.mem_write_breakpoints.insert(address, bp);
            }
        }
        
        id
    }
    
    /// Remove breakpoint
    pub fn remove_breakpoint(&mut self, address: u32, bp_type: BreakpointType) {
        match bp_type {
            BreakpointType::Execute => {
                self.exec_breakpoints.remove(&address);
            }
            BreakpointType::MemoryRead => {
                self.mem_read_breakpoints.remove(&address);
            }
            BreakpointType::MemoryWrite => {
                self.mem_write_breakpoints.remove(&address);
            }
            BreakpointType::MemoryAccess => {
                self.mem_read_breakpoints.remove(&address);
                self.mem_write_breakpoints.remove(&address);
            }
        }
    }
    
    /// Check if we should break on instruction execution
    pub fn check_exec_breakpoint(&mut self, pc: u32) -> bool {
        if let Some(bp) = self.exec_breakpoints.get_mut(&pc) {
            if bp.enabled {
                bp.hit_count += 1;
                self.state = ExecutionState::BreakpointHit { address: pc };
                return true;
            }
        }
        false
    }
    
    /// Check if we should break on memory read
    pub fn check_mem_read_breakpoint(&mut self, address: u32) -> bool {
        if let Some(bp) = self.mem_read_breakpoints.get_mut(&address) {
            if bp.enabled {
                bp.hit_count += 1;
                self.state = ExecutionState::BreakpointHit { address };
                return true;
            }
        }
        false
    }
    
    /// Check if we should break on memory write
    pub fn check_mem_write_breakpoint(&mut self, address: u32) -> bool {
        if let Some(bp) = self.mem_write_breakpoints.get_mut(&address) {
            if bp.enabled {
                bp.hit_count += 1;
                self.state = ExecutionState::BreakpointHit { address };
                return true;
            }
        }
        false
    }
    
    /// Called before executing an instruction
    /// Returns true if execution should pause
    pub fn before_instruction(&mut self, pc: u32) -> bool {
        self.instruction_count += 1;
        
        // Check execution breakpoints
        if self.check_exec_breakpoint(pc) {
            return true;
        }
        
        // Handle stepping
        match self.state {
            ExecutionState::Stepping => {
                // Pause after this instruction
                self.state = ExecutionState::Paused;
                return true;
            }
            ExecutionState::SteppingOver => {
                // Check if we've reached the target
                if let Some(target) = self.step_over_target {
                    if pc == target {
                        self.state = ExecutionState::Paused;
                        self.step_over_target = None;
                        return true;
                    }
                }
            }
            ExecutionState::SteppingOut => {
                // Check if we've returned from the function
                if let Some(depth) = self.step_out_depth {
                    if self.call_stack.len() < depth {
                        self.state = ExecutionState::Paused;
                        self.step_out_depth = None;
                        return true;
                    }
                }
            }
            _ => {}
        }
        
        false
    }
    
    /// Track function call (for stack tracking)
    pub fn track_call(&mut self, return_address: u32, stack_pointer: u32, function_address: u32) {
        self.call_stack.push(StackFrame {
            return_address,
            stack_pointer,
            function_address: Some(function_address),
        });
    }
    
    /// Track function return
    pub fn track_return(&mut self) {
        self.call_stack.pop();
    }
    
    /// Get call stack
    pub fn call_stack(&self) -> &[StackFrame] {
        &self.call_stack
    }
    
    /// Get instruction count
    pub fn instruction_count(&self) -> u64 {
        self.instruction_count
    }
    
    /// Get all breakpoints
    pub fn breakpoints(&self) -> Vec<&Breakpoint> {
        self.exec_breakpoints.values()
            .chain(self.mem_read_breakpoints.values())
            .chain(self.mem_write_breakpoints.values())
            .collect()
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}
