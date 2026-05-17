// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Load/Store instructions (stubs - require memory interface)

use crate::registers::Registers;
use newton_utils::Result;

// These are stubs for now - will need memory interface to implement properly

pub fn lwz(_regs: &mut Registers, _rt: u8, _ra: u8, _d: i16) -> Result<()> {
    tracing::debug!("lwz instruction - requires memory interface");
    Ok(())
}

pub fn stw(_regs: &mut Registers, _rs: u8, _ra: u8, _d: i16) -> Result<()> {
    tracing::debug!("stw instruction - requires memory interface");
    Ok(())
}
