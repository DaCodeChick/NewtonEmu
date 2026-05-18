// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2024 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Main emulator implementation

use crate::config::EmulatorConfig;
use crate::memory::Memory;
use crate::rom::Rom;
use newton_cpu::{Cpu, PpcModel};
use newton_devices::video::{Framebuffer, ColorDepth};
use newton_devices::adb::{AdbController, AdbKeyboard, AdbMouse};
use newton_utils::Result;

/// Main emulator state
pub struct Emulator {
    /// CPU
    cpu: Cpu,
    
    /// Memory system
    memory: Memory,
    
    /// Framebuffer
    framebuffer: Framebuffer,
    
    /// ADB controller
    adb: AdbController,
    
    /// Configuration
    config: EmulatorConfig,
    
    /// Is emulator running
    running: bool,
}

impl Emulator {
    /// Create a new emulator with the given configuration
    pub fn new(config: EmulatorConfig) -> Result<Self> {
        tracing::info!("Initializing NewtonEmu");
        
        // Create CPU
        let cpu_model: PpcModel = config.cpu.model.into();
        let cpu = Cpu::new(cpu_model);
        
        // Create memory
        let ram_size = config.memory.ram_size_mb * 1024 * 1024;
        let mut memory = Memory::new(ram_size);
        
        // Load ROM if specified
        if let Some(rom_path) = &config.memory.rom_path {
            let rom = Rom::load_from_file(rom_path)?;
            memory.load_rom(rom);
        }
        
        // Create framebuffer
        let depth = match config.display.color_depth {
            8 => ColorDepth::Indexed8,
            16 => ColorDepth::Rgb16,
            32 => ColorDepth::Rgba32,
            _ => ColorDepth::Rgba32,
        };
        let framebuffer = Framebuffer::new(
            config.display.width,
            config.display.height,
            depth,
        );
        
        // Create ADB controller with keyboard and mouse
        let mut adb = AdbController::new();
        adb.add_device(Box::new(AdbKeyboard::new(2)));
        adb.add_device(Box::new(AdbMouse::new(3)));
        
        Ok(Self {
            cpu,
            memory,
            framebuffer,
            adb,
            config,
            running: false,
        })
    }

    /// Reset the emulator
    pub fn reset(&mut self) {
        tracing::info!("Resetting emulator");
        self.cpu.reset();
        self.running = false;
    }

    /// Start emulation
    pub fn start(&mut self) {
        tracing::info!("Starting emulation");
        self.running = true;
    }

    /// Stop emulation
    pub fn stop(&mut self) {
        tracing::info!("Stopping emulation");
        self.running = false;
    }

    /// Check if emulator is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Execute a single instruction
    pub fn step(&mut self) -> Result<()> {
        self.cpu.step(&self.memory)
    }

    /// Run for a number of cycles
    pub fn run_cycles(&mut self, cycles: u64) -> Result<()> {
        for _ in 0..cycles {
            self.cpu.step(&self.memory)?;
        }
        Ok(())
    }

    /// Get CPU reference
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    /// Get mutable CPU reference
    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    /// Get memory reference
    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    /// Get mutable memory reference
    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }

    /// Get framebuffer reference
    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }

    /// Get mutable framebuffer reference
    pub fn framebuffer_mut(&mut self) -> &mut Framebuffer {
        &mut self.framebuffer
    }

    /// Get configuration
    pub fn config(&self) -> &EmulatorConfig {
        &self.config
    }
}
