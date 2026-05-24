// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Motorola 68000 CPU registers

use bitflags::bitflags;

/// 68k CPU registers
#[derive(Debug, Clone)]
pub struct Registers {
    /// Data registers D0-D7
    pub d: [u32; 8],
    
    /// Address registers A0-A7
    /// A7 is the stack pointer (SP)
    pub a: [u32; 8],
    
    /// Program Counter
    pub pc: u32,
    
    /// Status Register
    pub sr: StatusRegister,
    
    /// User Stack Pointer (USP) - saved when in supervisor mode
    pub usp: u32,
    
    /// Supervisor Stack Pointer (SSP) - A7 when in supervisor mode
    pub ssp: u32,
}

impl Registers {
    /// Create new registers initialized to reset state
    pub fn new() -> Self {
        Self {
            d: [0; 8],
            a: [0; 8],
            pc: 0,
            sr: StatusRegister::new(),
            usp: 0,
            ssp: 0,
        }
    }
    
    /// Get the current stack pointer (respects supervisor mode)
    pub fn sp(&self) -> u32 {
        if self.sr.supervisor_mode() {
            self.ssp
        } else {
            self.usp
        }
    }
    
    /// Set the stack pointer (respects supervisor mode)
    pub fn set_sp(&mut self, value: u32) {
        if self.sr.supervisor_mode() {
            self.ssp = value;
        } else {
            self.usp = value;
        }
        self.a[7] = value;
    }
    
    /// Switch between user and supervisor mode
    pub fn set_supervisor_mode(&mut self, supervisor: bool) {
        let was_supervisor = self.sr.supervisor_mode();
        
        if was_supervisor != supervisor {
            // Save current A7
            if was_supervisor {
                self.ssp = self.a[7];
            } else {
                self.usp = self.a[7];
            }
            
            // Load new A7
            self.a[7] = if supervisor { self.ssp } else { self.usp };
        }
        
        self.sr.set_supervisor_mode(supervisor);
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

/// Status Register (SR) - 16 bits
/// 
/// Format:
/// ```text
/// 15 14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
///  T  -  S  -  -  I  I  I  -  -  -  X  N  Z  V  C
/// ```
/// 
/// - T: Trace mode
/// - S: Supervisor mode
/// - I: Interrupt mask (3 bits)
/// - X: Extend flag
/// - N: Negative flag
/// - Z: Zero flag
/// - V: Overflow flag
/// - C: Carry flag
#[derive(Debug, Clone, Copy)]
pub struct StatusRegister {
    value: u16,
}

bitflags! {
    /// Status Register flags
    pub struct SrFlags: u16 {
        /// Carry flag
        const CARRY    = 1 << 0;
        /// Overflow flag
        const OVERFLOW = 1 << 1;
        /// Zero flag
        const ZERO     = 1 << 2;
        /// Negative flag
        const NEGATIVE = 1 << 3;
        /// Extend flag (used for multi-precision arithmetic)
        const EXTEND   = 1 << 4;
        
        /// Interrupt mask bits (3 bits: 8, 9, 10)
        const INT_MASK = 0x0700;
        
        /// Supervisor mode
        const SUPERVISOR = 1 << 13;
        /// Trace mode
        const TRACE      = 1 << 15;
    }
}

impl StatusRegister {
    /// Create new SR with default state (supervisor mode)
    pub fn new() -> Self {
        Self {
            value: SrFlags::SUPERVISOR.bits(),
        }
    }
    
    /// Get the raw SR value
    pub fn value(&self) -> u16 {
        self.value
    }
    
    /// Set the raw SR value
    pub fn set_value(&mut self, value: u16) {
        self.value = value;
    }
    
    /// Get the Condition Code Register (CCR) - lower 8 bits
    pub fn ccr(&self) -> u8 {
        (self.value & 0xFF) as u8
    }
    
    /// Set the Condition Code Register (CCR) - lower 8 bits
    pub fn set_ccr(&mut self, ccr: u8) {
        self.value = (self.value & 0xFF00) | (ccr as u16);
    }
    
    // Flag getters
    
    pub fn carry(&self) -> bool {
        (self.value & SrFlags::CARRY.bits()) != 0
    }
    
    pub fn overflow(&self) -> bool {
        (self.value & SrFlags::OVERFLOW.bits()) != 0
    }
    
    pub fn zero(&self) -> bool {
        (self.value & SrFlags::ZERO.bits()) != 0
    }
    
    pub fn negative(&self) -> bool {
        (self.value & SrFlags::NEGATIVE.bits()) != 0
    }
    
    pub fn extend(&self) -> bool {
        (self.value & SrFlags::EXTEND.bits()) != 0
    }
    
    pub fn supervisor_mode(&self) -> bool {
        (self.value & SrFlags::SUPERVISOR.bits()) != 0
    }
    
    pub fn trace_mode(&self) -> bool {
        (self.value & SrFlags::TRACE.bits()) != 0
    }
    
    pub fn interrupt_mask(&self) -> u8 {
        ((self.value & SrFlags::INT_MASK.bits()) >> 8) as u8
    }
    
    // Flag setters
    
    pub fn set_carry(&mut self, value: bool) {
        if value {
            self.value |= SrFlags::CARRY.bits();
        } else {
            self.value &= !SrFlags::CARRY.bits();
        }
    }
    
    pub fn set_overflow(&mut self, value: bool) {
        if value {
            self.value |= SrFlags::OVERFLOW.bits();
        } else {
            self.value &= !SrFlags::OVERFLOW.bits();
        }
    }
    
    pub fn set_zero(&mut self, value: bool) {
        if value {
            self.value |= SrFlags::ZERO.bits();
        } else {
            self.value &= !SrFlags::ZERO.bits();
        }
    }
    
    pub fn set_negative(&mut self, value: bool) {
        if value {
            self.value |= SrFlags::NEGATIVE.bits();
        } else {
            self.value &= !SrFlags::NEGATIVE.bits();
        }
    }
    
    pub fn set_extend(&mut self, value: bool) {
        if value {
            self.value |= SrFlags::EXTEND.bits();
        } else {
            self.value &= !SrFlags::EXTEND.bits();
        }
    }
    
    pub fn set_supervisor_mode(&mut self, value: bool) {
        if value {
            self.value |= SrFlags::SUPERVISOR.bits();
        } else {
            self.value &= !SrFlags::SUPERVISOR.bits();
        }
    }
    
    pub fn set_trace_mode(&mut self, value: bool) {
        if value {
            self.value |= SrFlags::TRACE.bits();
        } else {
            self.value &= !SrFlags::TRACE.bits();
        }
    }
    
    pub fn set_interrupt_mask(&mut self, mask: u8) {
        let mask = (mask & 0x07) as u16;
        self.value = (self.value & !SrFlags::INT_MASK.bits()) | (mask << 8);
    }
    
    /// Test a condition code
    pub fn test_condition(&self, condition: ConditionCode) -> bool {
        match condition {
            ConditionCode::True => true,
            ConditionCode::False => false,
            ConditionCode::High => !self.carry() && !self.zero(),
            ConditionCode::LowOrSame => self.carry() || self.zero(),
            ConditionCode::CarryClear => !self.carry(),
            ConditionCode::CarrySet => self.carry(),
            ConditionCode::NotEqual => !self.zero(),
            ConditionCode::Equal => self.zero(),
            ConditionCode::OverflowClear => !self.overflow(),
            ConditionCode::OverflowSet => self.overflow(),
            ConditionCode::Plus => !self.negative(),
            ConditionCode::Minus => self.negative(),
            ConditionCode::GreaterOrEqual => self.negative() == self.overflow(),
            ConditionCode::LessThan => self.negative() != self.overflow(),
            ConditionCode::GreaterThan => !self.zero() && (self.negative() == self.overflow()),
            ConditionCode::LessOrEqual => self.zero() || (self.negative() != self.overflow()),
        }
    }
}

/// Condition codes for branch instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionCode {
    True,           // T  - Always true
    False,          // F  - Always false
    High,           // HI - Higher than (unsigned)
    LowOrSame,      // LS - Lower or same (unsigned)
    CarryClear,     // CC/HS - Carry clear / Higher or same
    CarrySet,       // CS/LO - Carry set / Lower than
    NotEqual,       // NE - Not equal
    Equal,          // EQ - Equal
    OverflowClear,  // VC - Overflow clear
    OverflowSet,    // VS - Overflow set
    Plus,           // PL - Plus (positive)
    Minus,          // MI - Minus (negative)
    GreaterOrEqual, // GE - Greater or equal (signed)
    LessThan,       // LT - Less than (signed)
    GreaterThan,    // GT - Greater than (signed)
    LessOrEqual,    // LE - Less or equal (signed)
}

impl ConditionCode {
    /// Decode condition code from instruction bits
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x0F {
            0x0 => ConditionCode::True,
            0x1 => ConditionCode::False,
            0x2 => ConditionCode::High,
            0x3 => ConditionCode::LowOrSame,
            0x4 => ConditionCode::CarryClear,
            0x5 => ConditionCode::CarrySet,
            0x6 => ConditionCode::NotEqual,
            0x7 => ConditionCode::Equal,
            0x8 => ConditionCode::OverflowClear,
            0x9 => ConditionCode::OverflowSet,
            0xA => ConditionCode::Plus,
            0xB => ConditionCode::Minus,
            0xC => ConditionCode::GreaterOrEqual,
            0xD => ConditionCode::LessThan,
            0xE => ConditionCode::GreaterThan,
            0xF => ConditionCode::LessOrEqual,
            _ => unreachable!(),
        }
    }
}
