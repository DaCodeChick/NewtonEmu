// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Main emulator implementation

use crate::config::EmulatorConfig;
use crate::cpu_thread::{CpuThread, CpuCommand, CpuEvent, CpuState};
use crate::memory::Memory;
use crate::rom::Rom;
use crate::openfirmware::OpenFirmware;
use newton_cpu::{Cpu, PpcModel, MemoryInterface};
use newton_devices::video::{Framebuffer, ColorDepth};
use newton_devices::adb::{AdbController, AdbKeyboard, AdbMouse};
use newton_utils::Result;
use std::sync::Arc;

/// Execution mode for the emulator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmulatorMode {
    /// Single-threaded mode (CPU runs in main thread)
    SingleThreaded,
    
    /// Multi-threaded mode (CPU runs in dedicated thread)
    MultiThreaded,
}

/// Main emulator state
pub struct Emulator {
    /// CPU (only used in single-threaded mode)
    cpu: Option<Cpu>,
    
    /// CPU thread (only used in multi-threaded mode)
    cpu_thread: Option<CpuThread>,
    
    /// Memory system (Arc for sharing with CPU thread)
    memory: Arc<Memory>,
    
    /// Framebuffer
    framebuffer: Framebuffer,
    
    /// ADB controller
    adb: AdbController,
    
    /// OpenFirmware
    openfirmware: Option<OpenFirmware>,
    
    /// Configuration
    config: EmulatorConfig,
    
    /// Execution mode
    mode: EmulatorMode,
    
    /// Is emulator running (single-threaded mode only)
    running: bool,
}

impl Emulator {
    /// Create a new emulator with the given configuration (single-threaded mode)
    pub fn new(config: EmulatorConfig) -> Result<Self> {
        Self::with_mode(config, EmulatorMode::SingleThreaded)
    }
    
    /// Create a new emulator with specified execution mode
    pub fn with_mode(config: EmulatorConfig, mode: EmulatorMode) -> Result<Self> {
        tracing::info!("Initializing NewtonEmu in {:?} mode", mode);
        
        // Create CPU
        let cpu_model: PpcModel = config.cpu.model.into();
        let cpu = Cpu::new(cpu_model);
        
        // Create memory (wrapped in Arc for thread sharing)
        let ram_size = config.memory.ram_size_mb * 1024 * 1024;
        let mut memory = Memory::new(ram_size);
        
        // Load ROM if specified
        let mut openfirmware = None;
        if let Some(rom_path) = &config.memory.rom_path {
            let rom = Rom::load_from_file(rom_path)?;
            
            // Initialize OpenFirmware for NewWorld ROMs
            if rom.rom_type() == crate::rom::RomType::NewWorld {
                tracing::info!("NewWorld ROM detected - initializing OpenFirmware");
                openfirmware = Some(OpenFirmware::new());
            }
            
            memory.load_rom(rom);
        }
        
        let memory = Arc::new(memory);
        
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
        
        // Initialize CPU thread or local CPU based on mode
        let (cpu_opt, cpu_thread_opt) = match mode {
            EmulatorMode::SingleThreaded => {
                (Some(cpu), None)
            }
            EmulatorMode::MultiThreaded => {
                let cpu_thread = CpuThread::new(cpu, Arc::clone(&memory));
                (None, Some(cpu_thread))
            }
        };
        
        Ok(Self {
            cpu: cpu_opt,
            cpu_thread: cpu_thread_opt,
            memory,
            framebuffer,
            adb,
            openfirmware,
            config,
            mode,
            running: false,
        })
    }

    /// Reset the emulator
    pub fn reset(&mut self) {
        tracing::info!("Resetting emulator");
        
        match self.mode {
            EmulatorMode::SingleThreaded => {
                if let Some(cpu) = &mut self.cpu {
                    cpu.reset();
                    
                    // For NewWorld ROMs, set up OpenFirmware entry point
                    if self.openfirmware.is_some() {
                        // Store OF entry point address in a known location
                        // The ROM will look for this to call OpenFirmware
                        // We use r5 to pass the OF entry point at boot
                        cpu.registers.gpr[5] = 0xFFF1FFF0; // OF client interface address
                        tracing::info!("Set OpenFirmware entry point to 0x{:08X} (in r5)", cpu.registers.gpr[5]);
                    }
                }
                self.running = false;
            }
            EmulatorMode::MultiThreaded => {
                if let Some(cpu_thread) = &self.cpu_thread {
                    let _ = cpu_thread.send_command(CpuCommand::Reset);
                }
            }
        }
    }

    /// Start emulation
    pub fn start(&mut self) {
        tracing::info!("Starting emulation");
        
        match self.mode {
            EmulatorMode::SingleThreaded => {
                self.running = true;
            }
            EmulatorMode::MultiThreaded => {
                if let Some(cpu_thread) = &self.cpu_thread {
                    let _ = cpu_thread.send_command(CpuCommand::Run);
                }
            }
        }
    }

    /// Stop emulation
    pub fn stop(&mut self) {
        tracing::info!("Stopping emulation");
        
        match self.mode {
            EmulatorMode::SingleThreaded => {
                self.running = false;
            }
            EmulatorMode::MultiThreaded => {
                if let Some(cpu_thread) = &self.cpu_thread {
                    let _ = cpu_thread.send_command(CpuCommand::Pause);
                }
            }
        }
    }

    /// Check if emulator is running
    pub fn is_running(&self) -> bool {
        match self.mode {
            EmulatorMode::SingleThreaded => self.running,
            EmulatorMode::MultiThreaded => {
                self.cpu_thread
                    .as_ref()
                    .map(|t| t.state() == CpuState::Running)
                    .unwrap_or(false)
            }
        }
    }

    /// Execute a single instruction
    pub fn step(&mut self) -> Result<()> {
        match self.mode {
            EmulatorMode::SingleThreaded => {
                if let Some(cpu) = &mut self.cpu {
                    // Check if we need to intercept for OpenFirmware
                    let pc = cpu.registers.pc;
                    
                    // OpenFirmware client interface is typically at a specific address
                    // For now, we'll use 0xFFF00000 as the OF entry point
                    if let Some(of) = &mut self.openfirmware {
                        if pc == 0xFFF1FFF0 { // OpenFirmware client interface address
                            tracing::debug!("OpenFirmware client interface call at PC=0x{:08X}", pc);
                            self.handle_openfirmware_call()?;
                            return Ok(());
                        }
                    }
                    
                    cpu.step(&*self.memory)
                } else {
                    Err(newton_utils::Error::Cpu("CPU not available".to_string()))
                }
            }
            EmulatorMode::MultiThreaded => {
                if let Some(cpu_thread) = &self.cpu_thread {
                    cpu_thread.send_command(CpuCommand::Step)?;
                    Ok(())
                } else {
                    Err(newton_utils::Error::Cpu("CPU thread not available".to_string()))
                }
            }
        }
    }

    /// Run for a number of cycles
    pub fn run_cycles(&mut self, cycles: u64) -> Result<()> {
        match self.mode {
            EmulatorMode::SingleThreaded => {
                if let Some(cpu) = &mut self.cpu {
                    for _ in 0..cycles {
                        cpu.step(&*self.memory)?;
                    }
                    Ok(())
                } else {
                    Err(newton_utils::Error::Cpu("CPU not available".to_string()))
                }
            }
            EmulatorMode::MultiThreaded => {
                if let Some(cpu_thread) = &self.cpu_thread {
                    cpu_thread.send_command(CpuCommand::RunCycles(cycles))?;
                    Ok(())
                } else {
                    Err(newton_utils::Error::Cpu("CPU thread not available".to_string()))
                }
            }
        }
    }
    
    /// Send a command to the CPU thread (multi-threaded mode only)
    pub fn send_cpu_command(&self, cmd: CpuCommand) -> Result<()> {
        match self.mode {
            EmulatorMode::SingleThreaded => {
                Err(newton_utils::Error::Cpu("Cannot send commands in single-threaded mode".to_string()))
            }
            EmulatorMode::MultiThreaded => {
                if let Some(cpu_thread) = &self.cpu_thread {
                    cpu_thread.send_command(cmd)
                } else {
                    Err(newton_utils::Error::Cpu("CPU thread not available".to_string()))
                }
            }
        }
    }
    
    /// Poll for CPU events (multi-threaded mode only)
    pub fn poll_cpu_events(&mut self) -> Vec<CpuEvent> {
        if let Some(cpu_thread) = &mut self.cpu_thread {
            let mut events = Vec::new();
            while let Some(event) = cpu_thread.try_recv_event() {
                events.push(event);
            }
            events
        } else {
            Vec::new()
        }
    }

    /// Get CPU reference (single-threaded mode only)
    pub fn cpu(&self) -> Option<&Cpu> {
        self.cpu.as_ref()
    }

    /// Get mutable CPU reference (single-threaded mode only)
    pub fn cpu_mut(&mut self) -> Option<&mut Cpu> {
        self.cpu.as_mut()
    }
    
    /// Get CPU registers (works in both modes, but may require waiting for state in multi-threaded mode)
    ///
    /// In single-threaded mode, returns the current registers.
    /// In multi-threaded mode, you should use send_cpu_command(CpuCommand::GetState) and
    /// poll_cpu_events() to get a CpuStateSnapshot instead.
    pub fn registers(&self) -> Option<&newton_cpu::Registers> {
        self.cpu.as_ref().map(|c| &c.registers)
    }

    /// Get memory reference
    pub fn memory(&self) -> &Memory {
        &self.memory
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
    
    /// Get execution mode
    pub fn mode(&self) -> EmulatorMode {
        self.mode
    }
    
    /// Get OpenFirmware reference
    pub fn openfirmware(&self) -> Option<&OpenFirmware> {
        self.openfirmware.as_ref()
    }
    
    /// Get mutable OpenFirmware reference
    pub fn openfirmware_mut(&mut self) -> Option<&mut OpenFirmware> {
        self.openfirmware.as_mut()
    }
    
    /// Handle OpenFirmware client interface call
    fn handle_openfirmware_call(&mut self) -> Result<()> {
        // OpenFirmware arguments are passed in r3 (pointer to argument structure)
        let cpu = self.cpu.as_ref().ok_or_else(|| {
            newton_utils::Error::Cpu("CPU not available".to_string())
        })?;
        
        let args_ptr = cpu.registers.gpr[3];
        tracing::debug!("OpenFirmware call: args at 0x{:08X}", args_ptr);
        
        // Read the service name pointer (first word in the structure)
        let service_ptr = self.memory.as_ref().read_u32(args_ptr)?;
        
        // Read service name from memory (null-terminated string)
        let mut service_name = Vec::new();
        let mut offset = 0;
        loop {
            let byte = self.memory.as_ref().read_u8(service_ptr + offset)?;
            if byte == 0 {
                break;
            }
            service_name.push(byte);
            offset += 1;
            if offset > 256 {
                break; // Safety limit
            }
        }
        
        let service = String::from_utf8_lossy(&service_name);
        tracing::info!("OpenFirmware service call: {}", service);
        
        // For now, just return success and restore from the call
        // TODO: Actually implement the service calls
        
        // Return from the call (restore LR)
        if let Some(cpu) = &mut self.cpu {
            cpu.registers.pc = cpu.registers.lr;
        }
        
        Ok(())
    }
}

