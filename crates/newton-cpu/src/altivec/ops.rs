// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! AltiVec instruction operations
//!
//! This module implements the actual execution logic for AltiVec instructions.

/// AltiVec arithmetic operations
pub mod arithmetic;

/// AltiVec logical operations
pub mod logical;

/// AltiVec load/store operations
pub mod memory;

/// AltiVec permute and merge operations
pub mod permute;

/// AltiVec comparison operations
pub mod compare;

/// AltiVec shift and rotate operations
pub mod shift;

// Re-export commonly used functions
pub use arithmetic::*;
pub use logical::*;
pub use permute::*;
pub use compare::*;
pub use shift::*;
