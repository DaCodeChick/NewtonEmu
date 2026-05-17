// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! JIT compiler using Cranelift
//!
//! This module will provide dynamic recompilation of PowerPC code to native code.
//! Implementation will come after the interpreter is working.

// Placeholder for now
pub struct JitCompiler {
    _placeholder: (),
}

impl JitCompiler {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for JitCompiler {
    fn default() -> Self {
        Self::new()
    }
}
