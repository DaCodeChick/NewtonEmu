// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! PowerPC instruction interpreter

use crate::registers::Registers;
use crate::decoder::Instruction;
use newton_utils::Result;

/// Instruction interpreter
pub struct Interpreter {
    // State will be added as needed
}

impl Interpreter {
    /// Create a new interpreter
    pub fn new() -> Self {
        Self {}
    }

    /// Execute a decoded instruction
    pub fn execute(&mut self, instr: Instruction, regs: &mut Registers) -> Result<()> {
        match instr {
            Instruction::Add { rt, ra, rb, oe, rc } => {
                self.exec_add(regs, rt, ra, rb, oe, rc)
            }
            Instruction::Addi { rt, ra, simm } => {
                self.exec_addi(regs, rt, ra, simm)
            }
            Instruction::Addis { rt, ra, simm } => {
                self.exec_addis(regs, rt, ra, simm)
            }
            Instruction::Nop => Ok(()),
            Instruction::Unknown { opcode } => {
                tracing::warn!("Unknown instruction: 0x{:08X} at PC 0x{:08X}", opcode, regs.pc);
                Ok(())
            }
            _ => {
                tracing::debug!("Unimplemented instruction: {:?}", instr);
                Ok(())
            }
        }
    }

    fn exec_add(&mut self, regs: &mut Registers, rt: u8, ra: u8, rb: u8, _oe: bool, _rc: bool) -> Result<()> {
        let a = regs.gpr[ra as usize];
        let b = regs.gpr[rb as usize];
        regs.gpr[rt as usize] = a.wrapping_add(b);
        // TODO: Handle overflow (oe) and record (rc) flags
        Ok(())
    }

    fn exec_addi(&mut self, regs: &mut Registers, rt: u8, ra: u8, simm: i16) -> Result<()> {
        let a = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
        regs.gpr[rt as usize] = a.wrapping_add(simm as i32 as u32);
        Ok(())
    }

    fn exec_addis(&mut self, regs: &mut Registers, rt: u8, ra: u8, simm: i16) -> Result<()> {
        let a = if ra == 0 { 0 } else { regs.gpr[ra as usize] };
        regs.gpr[rt as usize] = a.wrapping_add(((simm as i32) << 16) as u32);
        Ok(())
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
