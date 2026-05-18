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
        // CR0 field
        const CR0_LT = 1 << 31;  // Less Than
        const CR0_GT = 1 << 30;  // Greater Than
        const CR0_EQ = 1 << 29;  // Equal
        const CR0_SO = 1 << 28;  // Summary Overflow
        
        // Additional CR fields will be accessed by bit manipulation
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
