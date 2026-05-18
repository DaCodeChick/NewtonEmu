// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Basic block detection and building

use crate::decoder::{Instruction, decode_instruction};
use crate::MemoryInterface;
use newton_utils::Result;

/// Maximum instructions in a single basic block
const MAX_BLOCK_SIZE: usize = 1000;

/// Type of block exit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitType {
    /// Unconditional branch to target
    Branch(u32),
    /// Conditional branch (may fall through)
    ConditionalBranch(u32),
    /// Branch to link register (return)
    BranchToLR,
    /// Branch to count register
    BranchToCTR,
    /// Fall through to next instruction
    Fallthrough,
    /// System call or trap
    Syscall,
    /// Unknown/invalid instruction
    Unknown,
}

/// A basic block of PowerPC instructions
/// 
/// A basic block is a sequence of instructions with:
/// - Single entry point (at the start)
/// - Single exit point (at the end)
/// - No branches in the middle
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Starting address of the block
    pub start_addr: u32,
    
    /// Instructions in this block
    pub instructions: Vec<Instruction>,
    
    /// How this block exits
    pub exit_type: ExitType,
    
    /// Address of next instruction after block
    pub next_addr: u32,
}

/// Builder for basic blocks
pub struct BlockBuilder;

impl BlockBuilder {
    /// Build a basic block starting at the given address
    pub fn build_from_memory(start_addr: u32, memory: &dyn MemoryInterface) -> Result<BasicBlock> {
        let mut instructions = Vec::new();
        let mut current_addr = start_addr;
        let exit_type;
        
        // Scan instructions until we hit a branch or reach max size
        loop {
            // Fetch and decode instruction
            let instr_word = memory.read_u32(current_addr)?;
            let instr = decode_instruction(instr_word)?;
            
            // Check if this instruction ends the block
            let is_block_end = Self::is_block_terminator(&instr);
            
            instructions.push(instr.clone());
            current_addr += 4;
            
            // Determine exit type
            if is_block_end {
                exit_type = Self::get_exit_type(&instr, current_addr - 4);
                break;
            }
            
            // Safety limit
            if instructions.len() >= MAX_BLOCK_SIZE {
                tracing::warn!("Block at 0x{:08X} reached max size, truncating", start_addr);
                exit_type = ExitType::Fallthrough;
                break;
            }
        }
        
        Ok(BasicBlock {
            start_addr,
            instructions,
            exit_type,
            next_addr: current_addr,
        })
    }
    
    /// Check if instruction terminates a basic block
    fn is_block_terminator(instr: &Instruction) -> bool {
        use Instruction::*;
        
        matches!(instr,
            // Unconditional branches
            B { .. } |
            
            // Conditional branches
            Bc { .. } |
            
            // Branch to register
            Bclr { .. } |
            Bcctr { .. } |
            
            // System calls and traps
            Sc | Tw { .. } | Twi { .. } |
            
            // Return from interrupt
            Rfi
        )
    }
    
    /// Determine the exit type from a terminator instruction
    fn get_exit_type(instr: &Instruction, instr_addr: u32) -> ExitType {
        use Instruction::*;
        
        match instr {
            // Unconditional branch
            B { li, aa: true, .. } => {
                // Absolute address
                ExitType::Branch(*li as u32)
            }
            B { li, aa: false, .. } => {
                // Relative to instruction address
                let target = instr_addr.wrapping_add(*li as u32);
                ExitType::Branch(target)
            }
            
            // Conditional branch
            Bc { bd, aa: true, .. } => {
                ExitType::ConditionalBranch(*bd as i32 as u32)
            }
            Bc { bd, aa: false, .. } => {
                let target = instr_addr.wrapping_add(*bd as i32 as u32);
                ExitType::ConditionalBranch(target)
            }
            
            // Branch to link register (return)
            Bclr { .. } => ExitType::BranchToLR,
            
            // Branch to count register
            Bcctr { .. } => ExitType::BranchToCTR,
            
            // System call
            Sc => ExitType::Syscall,
            
            // Traps
            Tw { .. } | Twi { .. } => ExitType::Syscall,
            
            // Return from interrupt
            Rfi => ExitType::BranchToLR, // Similar to return
            
            _ => ExitType::Unknown,
        }
    }
}

impl BasicBlock {
    /// Get the size of this block in bytes
    pub fn size_bytes(&self) -> usize {
        self.instructions.len() * 4
    }
    
    /// Check if this block can fall through to the next instruction
    pub fn can_fallthrough(&self) -> bool {
        matches!(self.exit_type, 
            ExitType::Fallthrough | ExitType::ConditionalBranch(_))
    }
    
    /// Get the branch target if this is a direct branch
    pub fn branch_target(&self) -> Option<u32> {
        match self.exit_type {
            ExitType::Branch(target) | ExitType::ConditionalBranch(target) => Some(target),
            _ => None,
        }
    }
}
