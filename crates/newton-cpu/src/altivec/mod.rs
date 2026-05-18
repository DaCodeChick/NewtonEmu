// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! AltiVec SIMD instruction support
//!
//! PowerPC G4 includes AltiVec (VMX) SIMD extensions with 32 128-bit vector registers.
//! This module implements the AltiVec instruction set.
//!
//! # Architecture Overview
//!
//! - 32 x 128-bit vector registers (VR0-VR31)
//! - Vector Status and Control Register (VSCR)
//! - VRSAVE register for register usage tracking
//! - Operations on multiple data types:
//!   - 16 x 8-bit signed/unsigned bytes
//!   - 8 x 16-bit signed/unsigned halfwords
//!   - 4 x 32-bit signed/unsigned words
//!   - 4 x 32-bit single-precision floats
//!
//! # Instruction Categories
//!
//! - Load/Store: Vector memory operations
//! - Arithmetic: Add, subtract, multiply, etc.
//! - Logical: AND, OR, XOR, etc.
//! - Compare: Vector comparisons
//! - Shift/Rotate: Bit manipulation
//! - Permute/Merge: Data rearrangement
//! - Pack/Unpack: Type conversion and saturation

use bitflags::bitflags;

pub mod types;
pub mod ops;

pub use types::*;

bitflags! {
    /// Vector Status and Control Register (VSCR)
    ///
    /// Controls saturation mode and non-Java floating-point mode,
    /// and records status from vector floating-point operations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Vscr: u32 {
        /// Non-Java floating-point mode
        /// When set, denormalized values are forced to zero
        const NJ = 1 << 16;
        
        /// Saturation flag
        /// Set when a saturating instruction produces a saturated result
        const SAT = 1 << 0;
    }
}

impl Default for Vscr {
    fn default() -> Self {
        Self::empty()
    }
}

/// AltiVec state
///
/// This holds the AltiVec-specific state separate from the main CPU registers.
/// The vector registers themselves are stored in `Registers::vr`.
#[derive(Debug, Clone)]
pub struct AltivecState {
    /// Vector Status and Control Register
    pub vscr: Vscr,
    
    /// VRSAVE - indicates which vector registers are in use
    /// Each bit corresponds to a VR register (bit 0 = VR0, bit 31 = VR31)
    pub vrsave: u32,
}

impl Default for AltivecState {
    fn default() -> Self {
        Self::new()
    }
}

impl AltivecState {
    /// Create new AltiVec state
    pub const fn new() -> Self {
        Self {
            vscr: Vscr::empty(),
            vrsave: 0,
        }
    }
    
    /// Reset AltiVec state
    pub fn reset(&mut self) {
        self.vscr = Vscr::empty();
        self.vrsave = 0;
    }
}
