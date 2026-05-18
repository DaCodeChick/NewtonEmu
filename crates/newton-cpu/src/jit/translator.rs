// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! PowerPC to Cranelift IR translation

use super::block::BasicBlock;
use crate::decoder::Instruction;
use newton_utils::Result;

use cranelift_codegen::ir::{Function, InstBuilder, Signature, UserFuncName, AbiParam, FuncRef};
use cranelift_codegen::ir::types::{I32, I64};
use cranelift_codegen::isa::CallConv;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::JITModule;
use cranelift_module::{Module, Linkage, FuncId};

/// Translator context for PowerPC to Cranelift IR
pub struct Translator<'a> {
    /// Function builder
    builder: FunctionBuilder<'a>,
    
    /// Variables for GPRs (r0-r31)
    gpr_vars: [Variable; 32],
    
    /// Variables for special registers
    pc_var: Variable,
    lr_var: Variable,
    ctr_var: Variable,
    cr_var: Variable,
    xer_var: Variable,
    
    /// Context pointer parameter (passed to function)
    ctx_param: cranelift_codegen::ir::Value,
    
    /// Memory callback function references
    mem_read_u32_ref: FuncRef,
    mem_write_u32_ref: FuncRef,
    mem_read_u16_ref: FuncRef,
    mem_write_u16_ref: FuncRef,
    mem_read_u8_ref: FuncRef,
    mem_write_u8_ref: FuncRef,
}

impl<'a> Translator<'a> {
    /// Create a new translator
    pub fn new(
        builder: FunctionBuilder<'a>,
        ctx_param: cranelift_codegen::ir::Value,
        mem_read_u32_ref: FuncRef,
        mem_write_u32_ref: FuncRef,
        mem_read_u16_ref: FuncRef,
        mem_write_u16_ref: FuncRef,
        mem_read_u8_ref: FuncRef,
        mem_write_u8_ref: FuncRef,
    ) -> Self {
        // Variables will be created by declare_variables()
        // Use placeholder values for now
        let gpr_vars = [Variable::from_u32(0); 32];
        
        Self {
            builder,
            gpr_vars,
            pc_var: Variable::from_u32(0),
            lr_var: Variable::from_u32(0),
            ctr_var: Variable::from_u32(0),
            cr_var: Variable::from_u32(0),
            xer_var: Variable::from_u32(0),
            ctx_param,
            mem_read_u32_ref,
            mem_write_u32_ref,
            mem_read_u16_ref,
            mem_write_u16_ref,
            mem_read_u8_ref,
            mem_write_u8_ref,
        }
    }
    
    /// Generate a memory read (32-bit)
    fn gen_mem_read_u32(&mut self, addr: cranelift_codegen::ir::Value) -> cranelift_codegen::ir::Value {
        let call = self.builder.ins().call(self.mem_read_u32_ref, &[self.ctx_param, addr]);
        self.builder.inst_results(call)[0]
    }
    
    /// Generate a memory write (32-bit)
    fn gen_mem_write_u32(&mut self, addr: cranelift_codegen::ir::Value, value: cranelift_codegen::ir::Value) {
        self.builder.ins().call(self.mem_write_u32_ref, &[self.ctx_param, addr, value]);
    }
    
    /// Generate a memory read (16-bit)
    fn gen_mem_read_u16(&mut self, addr: cranelift_codegen::ir::Value) -> cranelift_codegen::ir::Value {
        let call = self.builder.ins().call(self.mem_read_u16_ref, &[self.ctx_param, addr]);
        self.builder.inst_results(call)[0]
    }
    
    /// Generate a memory write (16-bit)
    fn gen_mem_write_u16(&mut self, addr: cranelift_codegen::ir::Value, value: cranelift_codegen::ir::Value) {
        self.builder.ins().call(self.mem_write_u16_ref, &[self.ctx_param, addr, value]);
    }
    
    /// Generate a memory read (8-bit)
    fn gen_mem_read_u8(&mut self, addr: cranelift_codegen::ir::Value) -> cranelift_codegen::ir::Value {
        let call = self.builder.ins().call(self.mem_read_u8_ref, &[self.ctx_param, addr]);
        self.builder.inst_results(call)[0]
    }
    
    /// Generate a memory write (8-bit)
    fn gen_mem_write_u8(&mut self, addr: cranelift_codegen::ir::Value, value: cranelift_codegen::ir::Value) {
        self.builder.ins().call(self.mem_write_u8_ref, &[self.ctx_param, addr, value]);
    }
    
    /// Load register values from JitContext at function entry
    pub fn load_registers_from_context(&mut self) {
        // JitContext layout:
        // Offset 0-15: memory fat pointer (16 bytes on 64-bit)
        // Offset 16: *mut JitRegisters
        //
        // JitRegisters layout:
        // Offset 0: gpr[32] (128 bytes)
        // Offset 128: pc (4 bytes)
        // Offset 132: lr (4 bytes)
        // Offset 136: ctr (4 bytes)
        // Offset 140: cr (4 bytes)
        // Offset 144: xer (4 bytes)
        
        // First, load the pointer to JitRegisters from offset 16 in JitContext
        let regs_ptr_offset = self.builder.ins().iconst(I64, 16);
        let regs_ptr_addr = self.builder.ins().iadd(self.ctx_param, regs_ptr_offset);
        let regs_ptr = self.builder.ins().load(I64, cranelift_codegen::ir::MemFlags::trusted(), regs_ptr_addr, 0);
        
        // Load each GPR (r0-r31)
        for i in 0..32 {
            let offset = i * 4; // Each register is 4 bytes
            let value = self.builder.ins().load(I32, cranelift_codegen::ir::MemFlags::trusted(), regs_ptr, offset);
            self.builder.def_var(self.gpr_vars[i as usize], value);
        }
        
        // Load special registers
        let pc = self.builder.ins().load(I32, cranelift_codegen::ir::MemFlags::trusted(), regs_ptr, 128);
        self.builder.def_var(self.pc_var, pc);
        
        let lr = self.builder.ins().load(I32, cranelift_codegen::ir::MemFlags::trusted(), regs_ptr, 132);
        self.builder.def_var(self.lr_var, lr);
        
        let ctr = self.builder.ins().load(I32, cranelift_codegen::ir::MemFlags::trusted(), regs_ptr, 136);
        self.builder.def_var(self.ctr_var, ctr);
        
        let cr = self.builder.ins().load(I32, cranelift_codegen::ir::MemFlags::trusted(), regs_ptr, 140);
        self.builder.def_var(self.cr_var, cr);
        
        let xer = self.builder.ins().load(I32, cranelift_codegen::ir::MemFlags::trusted(), regs_ptr, 144);
        self.builder.def_var(self.xer_var, xer);
    }
    
    /// Store register values back to JitContext at function exit
    pub fn store_registers_to_context(&mut self) {
        // Load the pointer to JitRegisters from offset 16 in JitContext
        let regs_ptr_offset = self.builder.ins().iconst(I64, 16);
        let regs_ptr_addr = self.builder.ins().iadd(self.ctx_param, regs_ptr_offset);
        let regs_ptr = self.builder.ins().load(I64, cranelift_codegen::ir::MemFlags::trusted(), regs_ptr_addr, 0);
        
        // Store each GPR (r0-r31)
        for i in 0..32 {
            let offset = i * 4;
            let value = self.builder.use_var(self.gpr_vars[i as usize]);
            self.builder.ins().store(cranelift_codegen::ir::MemFlags::trusted(), value, regs_ptr, offset);
        }
        
        // Store special registers
        let pc = self.builder.use_var(self.pc_var);
        self.builder.ins().store(cranelift_codegen::ir::MemFlags::trusted(), pc, regs_ptr, 128);
        
        let lr = self.builder.use_var(self.lr_var);
        self.builder.ins().store(cranelift_codegen::ir::MemFlags::trusted(), lr, regs_ptr, 132);
        
        let ctr = self.builder.use_var(self.ctr_var);
        self.builder.ins().store(cranelift_codegen::ir::MemFlags::trusted(), ctr, regs_ptr, 136);
        
        let cr = self.builder.use_var(self.cr_var);
        self.builder.ins().store(cranelift_codegen::ir::MemFlags::trusted(), cr, regs_ptr, 140);
        
        let xer = self.builder.use_var(self.xer_var);
        self.builder.ins().store(cranelift_codegen::ir::MemFlags::trusted(), xer, regs_ptr, 144);
    }
    
    /// Declare all variables in the function
    pub fn declare_variables(&mut self) {
        // In Cranelift 0.131+, declare_var() returns the Variable
        // We're pre-creating Variables with from_u32(), so we need to match them
        
        // Declare GPRs - create new variables and map them
        for i in 0..32 {
            let var = self.builder.declare_var(I32);
            self.gpr_vars[i] = var;
        }
        
        // Declare special registers
        self.pc_var = self.builder.declare_var(I32);
        self.lr_var = self.builder.declare_var(I32);
        self.ctr_var = self.builder.declare_var(I32);
        self.cr_var = self.builder.declare_var(I32);
        self.xer_var = self.builder.declare_var(I32);
    }
    
    /// Translate a single instruction
    pub fn translate_instruction(&mut self, instr: &Instruction) -> Result<()> {
        use Instruction::*;
        
        match instr {
            // ===== Integer Arithmetic =====
            
            Add { rt, ra, rb, .. } => {
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let result = self.builder.ins().iadd(a, b);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Addc { rt, ra, rb, .. } => {
                // Add with carry - simplified (no carry tracking yet)
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let result = self.builder.ins().iadd(a, b);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Adde { rt, ra, rb, .. } => {
                // Add extended - simplified (no carry tracking yet)
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let result = self.builder.ins().iadd(a, b);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Addi { rt, ra, simm } => {
                let a = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let imm = self.builder.ins().iconst(I32, *simm as i32 as i64);
                let result = self.builder.ins().iadd(a, imm);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Addic { rt, ra, simm } => {
                // Add immediate with carry - simplified
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let imm = self.builder.ins().iconst(I32, *simm as i32 as i64);
                let result = self.builder.ins().iadd(a, imm);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            AddicDot { rt, ra, simm } => {
                // Add immediate with carry and record - simplified
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let imm = self.builder.ins().iconst(I32, *simm as i32 as i64);
                let result = self.builder.ins().iadd(a, imm);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Addis { rt, ra, simm } => {
                let a = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let imm = self.builder.ins().iconst(I32, ((*simm as i32) << 16) as i64);
                let result = self.builder.ins().iadd(a, imm);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Subf { rt, ra, rb, .. } => {
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let result = self.builder.ins().isub(b, a); // Note: b - a
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Subfc { rt, ra, rb, .. } => {
                // Subtract from with carry - simplified
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let result = self.builder.ins().isub(b, a);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Subfe { rt, ra, rb, .. } => {
                // Subtract from extended - simplified
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let result = self.builder.ins().isub(b, a);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Subfic { rt, ra, simm } => {
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let imm = self.builder.ins().iconst(I32, *simm as i32 as i64);
                let result = self.builder.ins().isub(imm, a); // Note: imm - a
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Neg { rt, ra, .. } => {
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let result = self.builder.ins().ineg(a);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Mullw { rt, ra, rb, .. } => {
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let result = self.builder.ins().imul(a, b);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Mulli { rt, ra, simm } => {
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let imm = self.builder.ins().iconst(I32, *simm as i32 as i64);
                let result = self.builder.ins().imul(a, imm);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Mulhw { rt, ra, rb, .. } => {
                // Multiply high word - need 64-bit intermediate
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                // Extend to 64-bit
                let a64 = self.builder.ins().sextend(I64, a);
                let b64 = self.builder.ins().sextend(I64, b);
                let prod = self.builder.ins().imul(a64, b64);
                // Shift right 32 bits to get high word
                let shift = self.builder.ins().iconst(I64, 32);
                let high = self.builder.ins().sshr(prod, shift);
                let result = self.builder.ins().ireduce(I32, high);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Mulhwu { rt, ra, rb, .. } => {
                // Multiply high word unsigned
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                // Extend to 64-bit unsigned
                let a64 = self.builder.ins().uextend(I64, a);
                let b64 = self.builder.ins().uextend(I64, b);
                let prod = self.builder.ins().imul(a64, b64);
                // Shift right 32 bits to get high word
                let shift = self.builder.ins().iconst(I64, 32);
                let high = self.builder.ins().ushr(prod, shift);
                let result = self.builder.ins().ireduce(I32, high);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Divw { rt, ra, rb, .. } => {
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let result = self.builder.ins().sdiv(a, b);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            Divwu { rt, ra, rb, .. } => {
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let result = self.builder.ins().udiv(a, b);
                self.builder.def_var(self.gpr_vars[*rt as usize], result);
            }
            
            // ===== Logical Operations =====
            
            And { ra, rs, rb, .. } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let result = self.builder.ins().band(s, b);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Andc { ra, rs, rb, .. } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let not_b = self.builder.ins().bnot(b);
                let result = self.builder.ins().band(s, not_b);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Or { ra, rs, rb, .. } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let result = self.builder.ins().bor(s, b);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Orc { ra, rs, rb, .. } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let not_b = self.builder.ins().bnot(b);
                let result = self.builder.ins().bor(s, not_b);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Xor { ra, rs, rb, .. } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let result = self.builder.ins().bxor(s, b);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Nand { ra, rs, rb, .. } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let and_result = self.builder.ins().band(s, b);
                let result = self.builder.ins().bnot(and_result);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Nor { ra, rs, rb, .. } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let or_result = self.builder.ins().bor(s, b);
                let result = self.builder.ins().bnot(or_result);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Eqv { ra, rs, rb, .. } => {
                // Equivalence: ~(a XOR b)
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let xor_result = self.builder.ins().bxor(s, b);
                let result = self.builder.ins().bnot(xor_result);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Ori { ra, rs, uimm } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let imm = self.builder.ins().iconst(I32, *uimm as i64);
                let result = self.builder.ins().bor(s, imm);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Oris { ra, rs, uimm } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let imm = self.builder.ins().iconst(I32, ((*uimm as u32) << 16) as i64);
                let result = self.builder.ins().bor(s, imm);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Andi { ra, rs, uimm } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let imm = self.builder.ins().iconst(I32, *uimm as i64);
                let result = self.builder.ins().band(s, imm);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Andis { ra, rs, uimm } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let imm = self.builder.ins().iconst(I32, ((*uimm as u32) << 16) as i64);
                let result = self.builder.ins().band(s, imm);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Xori { ra, rs, uimm } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let imm = self.builder.ins().iconst(I32, *uimm as i64);
                let result = self.builder.ins().bxor(s, imm);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Xoris { ra, rs, uimm } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let imm = self.builder.ins().iconst(I32, ((*uimm as u32) << 16) as i64);
                let result = self.builder.ins().bxor(s, imm);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Extsb { ra, rs, .. } => {
                // Extend sign byte
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                // Mask to byte, then sign extend
                let byte_mask = self.builder.ins().iconst(I32, 0xFF);
                let byte = self.builder.ins().band(s, byte_mask);
                // Check sign bit
                let sign_bit = self.builder.ins().iconst(I32, 0x80);
                let is_negative = self.builder.ins().band(byte, sign_bit);
                let zero = self.builder.ins().iconst(I32, 0);
                let cmp = self.builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::NotEqual, is_negative, zero);
                // If negative, OR with 0xFFFFFF00, else keep as is
                let ext_mask = self.builder.ins().iconst(I32, 0xFFFFFF00u32 as i32 as i64);
                let extended = self.builder.ins().bor(byte, ext_mask);
                let result = self.builder.ins().select(cmp, extended, byte);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Extsh { ra, rs, .. } => {
                // Extend sign halfword
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                // Shift left 16, then arithmetic shift right 16
                let shift = self.builder.ins().iconst(I32, 16);
                let shifted_left = self.builder.ins().ishl(s, shift);
                let result = self.builder.ins().sshr(shifted_left, shift);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Cntlzw { ra, rs, .. } => {
                // Count leading zeros
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let result = self.builder.ins().clz(s);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            // ===== Shifts and Rotates =====
            
            Slw { ra, rs, rb, .. } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let result = self.builder.ins().ishl(s, b);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Srw { ra, rs, rb, .. } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let result = self.builder.ins().ushr(s, b);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Sraw { ra, rs, rb, .. } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let result = self.builder.ins().sshr(s, b);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            Srawi { ra, rs, sh, .. } => {
                let s = self.builder.use_var(self.gpr_vars[*rs as usize]);
                let shift = self.builder.ins().iconst(I32, *sh as i64);
                let result = self.builder.ins().sshr(s, shift);
                self.builder.def_var(self.gpr_vars[*ra as usize], result);
            }
            
            // Comparisons (simplified - just set CR for now)
            Cmp { crfd, ra, rb, .. } => {
                let a = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let b = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let cmp = self.builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::SignedLessThan, a, b);
                // TODO: Properly encode CR field
                let _ = (crfd, cmp); // Use variables to avoid warnings
            }
            
            // ===== Load/Store Operations =====
            
            Lwz { rt, ra, d } => {
                // Load Word and Zero: rt = MEM[ra + d]
                let base = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let offset = self.builder.ins().iconst(I32, *d as i64);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.gen_mem_read_u32(addr);
                self.builder.def_var(self.gpr_vars[*rt as usize], value);
            }
            
            Lwzu { rt, ra, d } => {
                // Load Word and Zero with Update: rt = MEM[ra + d], ra = ra + d
                let base = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let offset = self.builder.ins().iconst(I32, *d as i64);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.gen_mem_read_u32(addr);
                self.builder.def_var(self.gpr_vars[*rt as usize], value);
                self.builder.def_var(self.gpr_vars[*ra as usize], addr);
            }
            
            Lwzx { rt, ra, rb } => {
                // Load Word and Zero Indexed: rt = MEM[ra + rb]
                let base = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let offset = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.gen_mem_read_u32(addr);
                self.builder.def_var(self.gpr_vars[*rt as usize], value);
            }
            
            Stw { rs, ra, d } => {
                // Store Word: MEM[ra + d] = rs
                let base = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let offset = self.builder.ins().iconst(I32, *d as i64);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.builder.use_var(self.gpr_vars[*rs as usize]);
                self.gen_mem_write_u32(addr, value);
            }
            
            Stwu { rs, ra, d } => {
                // Store Word with Update: MEM[ra + d] = rs, ra = ra + d
                let base = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let offset = self.builder.ins().iconst(I32, *d as i64);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.builder.use_var(self.gpr_vars[*rs as usize]);
                self.gen_mem_write_u32(addr, value);
                self.builder.def_var(self.gpr_vars[*ra as usize], addr);
            }
            
            Stwx { rs, ra, rb } => {
                // Store Word Indexed: MEM[ra + rb] = rs
                let base = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let offset = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.builder.use_var(self.gpr_vars[*rs as usize]);
                self.gen_mem_write_u32(addr, value);
            }
            
            Lhz { rt, ra, d } => {
                // Load Halfword and Zero: rt = MEM[ra + d] (16-bit)
                let base = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let offset = self.builder.ins().iconst(I32, *d as i64);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.gen_mem_read_u16(addr);
                self.builder.def_var(self.gpr_vars[*rt as usize], value);
            }
            
            Lhzu { rt, ra, d } => {
                // Load Halfword and Zero with Update
                let base = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let offset = self.builder.ins().iconst(I32, *d as i64);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.gen_mem_read_u16(addr);
                self.builder.def_var(self.gpr_vars[*rt as usize], value);
                self.builder.def_var(self.gpr_vars[*ra as usize], addr);
            }
            
            Lhzx { rt, ra, rb } => {
                // Load Halfword and Zero Indexed
                let base = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let offset = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.gen_mem_read_u16(addr);
                self.builder.def_var(self.gpr_vars[*rt as usize], value);
            }
            
            Lha { rt, ra, d } => {
                // Load Halfword Algebraic (sign-extend): rt = EXTS(MEM[ra + d])
                let base = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let offset = self.builder.ins().iconst(I32, *d as i64);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.gen_mem_read_u16(addr);
                // Sign extend from 16 to 32 bits
                let shift = self.builder.ins().iconst(I32, 16);
                let shifted_left = self.builder.ins().ishl(value, shift);
                let signed = self.builder.ins().sshr(shifted_left, shift);
                self.builder.def_var(self.gpr_vars[*rt as usize], signed);
            }
            
            Lhax { rt, ra, rb } => {
                // Load Halfword Algebraic Indexed
                let base = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let offset = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.gen_mem_read_u16(addr);
                // Sign extend from 16 to 32 bits
                let shift = self.builder.ins().iconst(I32, 16);
                let shifted_left = self.builder.ins().ishl(value, shift);
                let signed = self.builder.ins().sshr(shifted_left, shift);
                self.builder.def_var(self.gpr_vars[*rt as usize], signed);
            }
            
            Sth { rs, ra, d } => {
                // Store Halfword: MEM[ra + d] = rs (16-bit)
                let base = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let offset = self.builder.ins().iconst(I32, *d as i64);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.builder.use_var(self.gpr_vars[*rs as usize]);
                self.gen_mem_write_u16(addr, value);
            }
            
            Sthx { rs, ra, rb } => {
                // Store Halfword Indexed
                let base = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let offset = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.builder.use_var(self.gpr_vars[*rs as usize]);
                self.gen_mem_write_u16(addr, value);
            }
            
            Lbz { rt, ra, d } => {
                // Load Byte and Zero: rt = MEM[ra + d] (8-bit)
                let base = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let offset = self.builder.ins().iconst(I32, *d as i64);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.gen_mem_read_u8(addr);
                self.builder.def_var(self.gpr_vars[*rt as usize], value);
            }
            
            Lbzu { rt, ra, d } => {
                // Load Byte and Zero with Update
                let base = self.builder.use_var(self.gpr_vars[*ra as usize]);
                let offset = self.builder.ins().iconst(I32, *d as i64);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.gen_mem_read_u8(addr);
                self.builder.def_var(self.gpr_vars[*rt as usize], value);
                self.builder.def_var(self.gpr_vars[*ra as usize], addr);
            }
            
            Lbzx { rt, ra, rb } => {
                // Load Byte and Zero Indexed
                let base = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let offset = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.gen_mem_read_u8(addr);
                self.builder.def_var(self.gpr_vars[*rt as usize], value);
            }
            
            Stb { rs, ra, d } => {
                // Store Byte: MEM[ra + d] = rs (8-bit)
                let base = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let offset = self.builder.ins().iconst(I32, *d as i64);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.builder.use_var(self.gpr_vars[*rs as usize]);
                self.gen_mem_write_u8(addr, value);
            }
            
            Stbx { rs, ra, rb } => {
                // Store Byte Indexed
                let base = if *ra == 0 {
                    self.builder.ins().iconst(I32, 0)
                } else {
                    self.builder.use_var(self.gpr_vars[*ra as usize])
                };
                let offset = self.builder.use_var(self.gpr_vars[*rb as usize]);
                let addr = self.builder.ins().iadd(base, offset);
                let value = self.builder.use_var(self.gpr_vars[*rs as usize]);
                self.gen_mem_write_u8(addr, value);
            }
            
            // Branches - handled at block level
            B { .. } | Bc { .. } | Bclr { .. } | Bcctr { .. } => {
                // Block exit handled separately
            }
            
            // NOP and other simple instructions
            Nop => {
                // No-op, nothing to generate
            }
            
            _ => {
                tracing::debug!("JIT translation not implemented for: {:?}", instr);
            }
        }
        
        Ok(())
    }
}

/// Translate a basic block to Cranelift IR and compile it
pub fn translate_block(block: &BasicBlock, module: &mut JITModule) -> Result<FuncId> {
    // Declare memory callback functions in the module
    let mut mem_sig_read = Signature::new(CallConv::SystemV);
    mem_sig_read.params.push(AbiParam::new(I64)); // ctx pointer
    mem_sig_read.params.push(AbiParam::new(I32)); // address
    mem_sig_read.returns.push(AbiParam::new(I32)); // returned value
    
    let mut mem_sig_write = Signature::new(CallConv::SystemV);
    mem_sig_write.params.push(AbiParam::new(I64)); // ctx pointer
    mem_sig_write.params.push(AbiParam::new(I32)); // address
    mem_sig_write.params.push(AbiParam::new(I32)); // value
    
    // Declare external memory functions (these will be resolved by JITBuilder symbols)
    let mem_read_u32_id = module.declare_function("jit_memory_read_u32", Linkage::Import, &mem_sig_read)
        .map_err(|e| newton_utils::Error::Cpu(format!("Failed to declare mem_read_u32: {}", e)))?;
    let mem_write_u32_id = module.declare_function("jit_memory_write_u32", Linkage::Import, &mem_sig_write)
        .map_err(|e| newton_utils::Error::Cpu(format!("Failed to declare mem_write_u32: {}", e)))?;
    let mem_read_u16_id = module.declare_function("jit_memory_read_u16", Linkage::Import, &mem_sig_read)
        .map_err(|e| newton_utils::Error::Cpu(format!("Failed to declare mem_read_u16: {}", e)))?;
    let mem_write_u16_id = module.declare_function("jit_memory_write_u16", Linkage::Import, &mem_sig_write)
        .map_err(|e| newton_utils::Error::Cpu(format!("Failed to declare mem_write_u16: {}", e)))?;
    let mem_read_u8_id = module.declare_function("jit_memory_read_u8", Linkage::Import, &mem_sig_read)
        .map_err(|e| newton_utils::Error::Cpu(format!("Failed to declare mem_read_u8: {}", e)))?;
    let mem_write_u8_id = module.declare_function("jit_memory_write_u8", Linkage::Import, &mem_sig_write)
        .map_err(|e| newton_utils::Error::Cpu(format!("Failed to declare mem_write_u8: {}", e)))?;
    
    // Create function signature: fn(ctx_ptr: *mut JitContext) -> u32
    // The function takes a pointer to JIT context (for memory callbacks) and returns the new PC
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(I64)); // Pointer to JIT context
    sig.returns.push(AbiParam::new(I32)); // Return new PC value
    
    // Declare function
    let func_name = format!("ppc_block_{:08x}", block.start_addr);
    let func_id = module
        .declare_function(&func_name, Linkage::Local, &sig)
        .map_err(|e| newton_utils::Error::Cpu(format!("Failed to declare function: {}", e)))?;
    
    // Create function
    let mut func = Function::with_name_signature(
        UserFuncName::user(0, block.start_addr),
        sig
    );
    
    // Import the function references for use in the IR
    let mem_read_u32_ref = module.declare_func_in_func(mem_read_u32_id, &mut func);
    let mem_write_u32_ref = module.declare_func_in_func(mem_write_u32_id, &mut func);
    let mem_read_u16_ref = module.declare_func_in_func(mem_read_u16_id, &mut func);
    let mem_write_u16_ref = module.declare_func_in_func(mem_write_u16_id, &mut func);
    let mem_read_u8_ref = module.declare_func_in_func(mem_read_u8_id, &mut func);
    let mem_write_u8_ref = module.declare_func_in_func(mem_write_u8_id, &mut func);
    
    // Create function builder context
    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);
    
    // Create entry block
    let entry_block = builder.create_block();
    builder.append_block_param(entry_block, I64); // ctx_ptr parameter
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);
    
    // Get the context pointer parameter
    let ctx_param = builder.block_params(entry_block)[0];
    
    // Create translator with memory callback refs
    let mut translator = Translator::new(
        builder,
        ctx_param,
        mem_read_u32_ref,
        mem_write_u32_ref,
        mem_read_u16_ref,
        mem_write_u16_ref,
        mem_read_u8_ref,
        mem_write_u8_ref,
    );
    translator.declare_variables();
    
    // Load register values from the context pointer
    translator.load_registers_from_context();
    
    // Translate all instructions in the block
    for instr in &block.instructions {
        translator.translate_instruction(instr)?;
    }
    
    // Store register values back to context
    translator.store_registers_to_context();
    
    // Calculate and return the next PC
    // For now, just return the PC after the last instruction
    let next_pc = block.start_addr.wrapping_add((block.instructions.len() as u32) * 4);
    let next_pc_val = translator.builder.ins().iconst(I32, next_pc as i64);
    translator.builder.ins().return_(&[next_pc_val]);
    
    translator.builder.finalize();
    
    // Define the function in the module
    let mut ctx = cranelift_codegen::Context::for_function(func);
    module
        .define_function(func_id, &mut ctx)
        .map_err(|e| newton_utils::Error::Cpu(format!("Failed to define function: {}", e)))?;
    
    // Clear the context
    module.clear_context(&mut ctx);
    
    tracing::debug!("Translated block at 0x{:08X} to function ID {:?}", block.start_addr, func_id);
    
    Ok(func_id)
}
