// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! PowerPC register definitions

use bitflags::bitflags;

/// PowerPC CPU model variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PpcModel {
    /// PowerPC 750 (G3)
    G3,
    /// PowerPC 7400/7450 (G4)
    G4,
    /// PowerPC 970 (G5)
    G5,
}

/// PowerPC CPU registers
#[derive(Debug, Clone)]
pub struct Registers {
    /// CPU model
    pub model: PpcModel,
    
    /// General Purpose Registers (r0-r31)
    pub gpr: [u32; 32],
    
    /// Floating Point Registers (f0-f31)
    pub fpr: [f64; 32],
    
    /// Vector Registers for AltiVec (v0-v31)
    pub vr: [[u32; 4]; 32],
    
    /// Program Counter
    pub pc: u32,
    
    /// Link Register
    pub lr: u32,
    
    /// Count Register
    pub ctr: u32,
    
    /// Condition Register
    pub cr: ConditionRegister,
    
    /// XER (Fixed-Point Exception Register)
    pub xer: Xer,
    
    /// Machine State Register
    pub msr: MachineStateRegister,
    
    /// Segment Registers (SR0-SR15)
    pub sr: [u32; 16],
    
    /// Special Purpose Registers
    pub spr: [u32; 1024],
}

impl Registers {
    /// Create new registers for the given CPU model
    pub fn new(model: PpcModel) -> Self {
        Self {
            model,
            gpr: [0; 32],
            fpr: [0.0; 32],
            vr: [[0; 4]; 32],
            pc: 0xFFF0_0100, // PowerPC reset vector
            lr: 0,
            ctr: 0,
            cr: ConditionRegister::empty(),
            xer: Xer::empty(),
            msr: MachineStateRegister::empty(),
            sr: [0; 16],
            spr: [0; 1024],
        }
    }

    /// Reset registers to initial state
    pub fn reset(&mut self) {
        self.gpr.fill(0);
        self.fpr.fill(0.0);
        self.vr = [[0; 4]; 32];
        self.pc = 0xFFF0_0100;
        self.lr = 0;
        self.ctr = 0;
        self.cr = ConditionRegister::empty();
        self.xer = Xer::empty();
        self.msr = MachineStateRegister::empty();
        self.sr.fill(0);
    }
}

bitflags! {
    /// Condition Register (CR) - 8 x 4-bit fields
    #[derive(Debug, Clone, Copy)]
    pub struct ConditionRegister: u32 {
        // CR0 field (bits 0-3, numbered from MSB)
        const CR0_LT = 1 << 31;  // Less Than
        const CR0_GT = 1 << 30;  // Greater Than
        const CR0_EQ = 1 << 29;  // Equal
        const CR0_SO = 1 << 28;  // Summary Overflow
        
        // CR1 field (bits 4-7)
        const CR1_LT = 1 << 27;
        const CR1_GT = 1 << 26;
        const CR1_EQ = 1 << 25;
        const CR1_SO = 1 << 24;
        
        // CR2 field (bits 8-11)
        const CR2_LT = 1 << 23;
        const CR2_GT = 1 << 22;
        const CR2_EQ = 1 << 21;
        const CR2_SO = 1 << 20;
        
        // CR3 field (bits 12-15)
        const CR3_LT = 1 << 19;
        const CR3_GT = 1 << 18;
        const CR3_EQ = 1 << 17;
        const CR3_SO = 1 << 16;
        
        // CR4 field (bits 16-19)
        const CR4_LT = 1 << 15;
        const CR4_GT = 1 << 14;
        const CR4_EQ = 1 << 13;
        const CR4_SO = 1 << 12;
        
        // CR5 field (bits 20-23)
        const CR5_LT = 1 << 11;
        const CR5_GT = 1 << 10;
        const CR5_EQ = 1 << 9;
        const CR5_SO = 1 << 8;
        
        // CR6 field (bits 24-27)
        const CR6_LT = 1 << 7;
        const CR6_GT = 1 << 6;
        const CR6_EQ = 1 << 5;
        const CR6_SO = 1 << 4;
        
        // CR7 field (bits 28-31)
        const CR7_LT = 1 << 3;
        const CR7_GT = 1 << 2;
        const CR7_EQ = 1 << 1;
        const CR7_SO = 1 << 0;
    }
}

bitflags! {
    /// XER (Fixed-Point Exception Register)
    #[derive(Debug, Clone, Copy)]
    pub struct Xer: u32 {
        const SO = 1 << 31;  // Summary Overflow
        const OV = 1 << 30;  // Overflow
        const CA = 1 << 29;  // Carry
    }
}

bitflags! {
    /// MSR (Machine State Register)
    #[derive(Debug, Clone, Copy)]
    pub struct MachineStateRegister: u32 {
        const POW = 1 << 18;  // Power Management Enable
        const ILE = 1 << 16;  // Exception Little-Endian Mode
        const EE  = 1 << 15;  // External Interrupt Enable
        const PR  = 1 << 14;  // Privilege Level (0=supervisor, 1=user)
        const FP  = 1 << 13;  // Floating-Point Available
        const ME  = 1 << 12;  // Machine Check Enable
        const FE0 = 1 << 11;  // Floating-Point Exception Mode 0
        const SE  = 1 << 10;  // Single-Step Trace Enable
        const BE  = 1 << 9;   // Branch Trace Enable
        const FE1 = 1 << 8;   // Floating-Point Exception Mode 1
        const IR  = 1 << 5;   // Instruction Relocate
        const DR  = 1 << 4;   // Data Relocate
        const RI  = 1 << 1;   // Recoverable Interrupt
        const LE  = 1 << 0;   // Little-Endian Mode
    }
}

/// Special Purpose Register indices
pub mod spr {
    pub const XER: usize = 1;
    pub const LR: usize = 8;
    pub const CTR: usize = 9;
    pub const DSISR: usize = 18;
    pub const DAR: usize = 19;
    pub const DEC: usize = 22;
    pub const SDR1: usize = 25;
    pub const SRR0: usize = 26;
    pub const SRR1: usize = 27;
    pub const SPRG0: usize = 272;
    pub const SPRG1: usize = 273;
    pub const SPRG2: usize = 274;
    pub const SPRG3: usize = 275;
    pub const PVR: usize = 287;  // Processor Version Register
}
