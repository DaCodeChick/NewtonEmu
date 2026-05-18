// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Execution result tracking

/// Result of instruction execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecResult {
    /// Normal instruction - advance PC by 4
    Continue,
    
    /// Branch taken - PC already set, don't advance
    BranchTaken,
    
    /// Conditional branch not taken - advance PC by 4
    BranchNotTaken,
    
    /// System call - special handling may be needed
    Syscall,
    
    /// Trap instruction
    Trap,
}

impl ExecResult {
    /// Check if PC should be auto-incremented
    pub fn should_advance_pc(&self) -> bool {
        matches!(self, ExecResult::Continue | ExecResult::BranchNotTaken)
    }
}
