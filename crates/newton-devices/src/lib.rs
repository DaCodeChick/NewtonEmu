// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Hardware device emulation
//!
//! This crate provides emulation for Mac peripherals:
//! - Video output (framebuffer)
//! - Input devices (ADB, USB)
//! - Storage (IDE, SCSI)
//! - Audio
//! - Networking

pub mod adb;
pub mod audio;
pub mod diagnostic;
pub mod network;
pub mod storage;
pub mod usb;
pub mod video;

pub use diagnostic::DiagnosticDevice;

use newton_utils::Result;

/// Memory-mapped I/O device trait
///
/// All MMIO devices must be Send + Sync for thread-safe access.
/// Device implementations should use interior mutability (e.g., RwLock, Mutex)
/// for any mutable state.
pub trait MmioDevice: Send + Sync {
    /// Read from device register
    fn read(&self, offset: u32, size: u8) -> Result<u32>;
    
    /// Write to device register
    fn write(&self, offset: u32, size: u8, value: u32) -> Result<()>;
    
    /// Get device name for debugging
    fn name(&self) -> &str;
}
