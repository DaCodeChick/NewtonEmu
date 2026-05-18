// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! JSON-based debugger protocol for IPC between frontend and emulator
//!
//! Messages are sent as JSON over stdin/stdout or a socket.

use serde::{Deserialize, Serialize};

/// Breakpoint type for JSON serialization
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BreakpointTypeDto {
    Execute,
    MemoryRead,
    MemoryWrite,
    MemoryAccess,
}

impl From<BreakpointTypeDto> for crate::debugger::BreakpointType {
    fn from(dto: BreakpointTypeDto) -> Self {
        match dto {
            BreakpointTypeDto::Execute => Self::Execute,
            BreakpointTypeDto::MemoryRead => Self::MemoryRead,
            BreakpointTypeDto::MemoryWrite => Self::MemoryWrite,
            BreakpointTypeDto::MemoryAccess => Self::MemoryAccess,
        }
    }
}

impl From<crate::debugger::BreakpointType> for BreakpointTypeDto {
    fn from(bp: crate::debugger::BreakpointType) -> Self {
        match bp {
            crate::debugger::BreakpointType::Execute => Self::Execute,
            crate::debugger::BreakpointType::MemoryRead => Self::MemoryRead,
            crate::debugger::BreakpointType::MemoryWrite => Self::MemoryWrite,
            crate::debugger::BreakpointType::MemoryAccess => Self::MemoryAccess,
        }
    }
}

/// Execution state for JSON serialization
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ExecutionStateDto {
    Running,
    Paused,
    BreakpointHit { address: u32 },
    Stepping,
    SteppingOver,
    SteppingOut,
}

impl From<crate::debugger::ExecutionState> for ExecutionStateDto {
    fn from(state: crate::debugger::ExecutionState) -> Self {
        match state {
            crate::debugger::ExecutionState::Running => Self::Running,
            crate::debugger::ExecutionState::Paused => Self::Paused,
            crate::debugger::ExecutionState::BreakpointHit { address } => {
                Self::BreakpointHit { address }
            }
            crate::debugger::ExecutionState::Stepping => Self::Stepping,
            crate::debugger::ExecutionState::SteppingOver => Self::SteppingOver,
            crate::debugger::ExecutionState::SteppingOut => Self::SteppingOut,
        }
    }
}

/// CPU state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStateDto {
    /// Program counter
    pub pc: u32,
    
    /// Link register
    pub lr: u32,
    
    /// Count register
    pub ctr: u32,
    
    /// Condition register
    pub cr: u32,
    
    /// XER register
    pub xer: u32,
    
    /// General purpose registers (r0-r31)
    pub gpr: [u32; 32],
    
    /// Floating point registers (f0-f31)
    pub fpr: [f64; 32],
    
    /// Execution state
    pub execution_state: ExecutionStateDto,
    
    /// Instruction count
    pub instruction_count: u64,
}

/// Breakpoint info for JSON serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointDto {
    pub address: u32,
    #[serde(rename = "type")]
    pub bp_type: BreakpointTypeDto,
    pub enabled: bool,
    pub hit_count: u32,
}

impl From<&crate::debugger::Breakpoint> for BreakpointDto {
    fn from(bp: &crate::debugger::Breakpoint) -> Self {
        Self {
            address: bp.address,
            bp_type: bp.bp_type.into(),
            enabled: bp.enabled,
            hit_count: bp.hit_count,
        }
    }
}

/// Disassembled instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisassemblyDto {
    pub address: u32,
    pub bytes: Vec<u8>,
    pub mnemonic: String,
    pub operands: String,
}

/// Memory dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDto {
    pub address: u32,
    pub data: Vec<u8>,
}

/// Debugger command from frontend to emulator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum DebuggerCommand {
    /// Enable the debugger
    Enable,
    
    /// Disable the debugger
    Disable,
    
    /// Pause execution
    Pause,
    
    /// Resume execution
    Resume,
    
    /// Step one instruction
    StepInto,
    
    /// Step over (execute function calls without stepping into them)
    StepOver,
    
    /// Step out (run until current function returns)
    StepOut,
    
    /// Add breakpoint
    AddBreakpoint {
        address: u32,
        #[serde(rename = "type")]
        bp_type: BreakpointTypeDto,
    },
    
    /// Remove breakpoint
    RemoveBreakpoint {
        address: u32,
        #[serde(rename = "type")]
        bp_type: BreakpointTypeDto,
    },
    
    /// Get current CPU state
    GetCpuState,
    
    /// Get memory range
    GetMemory {
        address: u32,
        length: u32,
    },
    
    /// Write memory
    WriteMemory {
        address: u32,
        data: Vec<u8>,
    },
    
    /// Get all breakpoints
    GetBreakpoints,
    
    /// Disassemble instructions
    Disassemble {
        address: u32,
        count: u32,
    },
}

/// Debugger response from emulator to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "response", rename_all = "snake_case")]
pub enum DebuggerResponse {
    /// Command executed successfully
    Ok,
    
    /// Command failed
    Error { message: String },
    
    /// CPU state
    CpuState { state: CpuStateDto },
    
    /// Memory data
    Memory { data: MemoryDto },
    
    /// Breakpoint list
    Breakpoints { breakpoints: Vec<BreakpointDto> },
    
    /// Disassembly
    Disassembly { instructions: Vec<DisassemblyDto> },
    
    /// Execution state changed (event)
    StateChanged { state: ExecutionStateDto },
    
    /// Breakpoint hit (event)
    BreakpointHit {
        address: u32,
        #[serde(rename = "type")]
        bp_type: BreakpointTypeDto,
    },
}

