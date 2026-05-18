// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use thiserror::Error;

/// Common error type for NewtonEmu
#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("CPU error: {0}")]
    Cpu(String),

    #[error("Memory error: {0}")]
    Memory(String),

    #[error("Device error: {0}")]
    Device(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

/// Result type alias for NewtonEmu operations
pub type Result<T> = std::result::Result<T, Error>;
