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
use newton_cpu::{Cpu, PpcModel};
use newton_devices::video::{Framebuffer, ColorDepth};
use newton_devices::adb::{AdbController, AdbKeyboard, AdbMouse};
use newton_utils::Result;
use std::sync::Arc;

/// OpenFirmware client interface entry point address
const OF_CLIENT_INTERFACE_ADDR: u32 = 0xFFF1FFF0;

/// High memory I/O space base address
const HIGH_MEM_IO_BASE: u32 = 0xFFFF0000;

/// High memory I/O space size (64KB)
const HIGH_MEM_IO_SIZE: u32 = 0x10000;

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
    
    /// ADB controller (not yet integrated)
    _adb: AdbController,
    
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
        
        // Register diagnostic devices for hardware register stubs
        // High memory I/O space
        memory.register_mmio(
            HIGH_MEM_IO_BASE,
            HIGH_MEM_IO_SIZE,
            Box::new(newton_devices::DiagnosticDevice::new("HighMemIO", HIGH_MEM_IO_SIZE as usize)),
        );
        
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
            _adb: adb,
            openfirmware,
            config,
            mode,
            running: false,
        })
    }

    /// Reset the emulator
    pub fn reset(&mut self) {
        tracing::info!("Resetting emulator");
        
        // Initialize boot RAM structures for ROM execution
        self.memory.init_boot_ram();
        
        match self.mode {
            EmulatorMode::SingleThreaded => {
                if let Some(cpu) = &mut self.cpu {
                    cpu.reset();
                    
                    // For NewWorld ROMs, override the PC to point to the ROM base
                    // NewWorld ROMs don't use the traditional 0xFFF00100 reset vector
                    if self.openfirmware.is_some() {
                        if let Some(rom) = self.memory.rom() {
                            let rom_entry = rom.base_address();
                            cpu.registers.pc = rom_entry;
                            tracing::info!("NewWorld ROM: Set PC to ROM base 0x{:08X}", rom_entry);
                        }
                        
                        // Store OF entry point address in r5
                        // The ROM will look for this to call OpenFirmware
                        cpu.registers.gpr[5] = OF_CLIENT_INTERFACE_ADDR;
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
                    
                    // OpenFirmware client interface intercept
                    if self.openfirmware.is_some() {
                        if pc == OF_CLIENT_INTERFACE_ADDR {
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
        use newton_cpu::MemoryInterface;
        
        // OpenFirmware arguments are passed in r3 (pointer to argument structure)
        let cpu = self.cpu.as_ref().ok_or_else(|| {
            newton_utils::Error::Cpu("CPU not available".to_string())
        })?;
        
        let args_ptr = cpu.registers.gpr[3];
        tracing::debug!("OpenFirmware call: args at 0x{:08X}", args_ptr);
        
        // Read the argument structure:
        // +0x00: service name pointer
        // +0x04: n_args
        // +0x08: n_returns
        // +0x0C: args[0], args[1], ..., args[n_args-1]
        // +0x??: returns[0], returns[1], ..., returns[n_returns-1]
        
        let service_ptr = self.memory.as_ref().read_u32(args_ptr)?;
        let n_args = self.memory.as_ref().read_u32(args_ptr + 4)? as usize;
        let n_returns = self.memory.as_ref().read_u32(args_ptr + 8)? as usize;
        
        // Read service name from memory (null-terminated string)
        let service = self.read_cstring(service_ptr)?;
        
        // Read input arguments
        let mut args = Vec::new();
        for i in 0..n_args {
            let arg = self.memory.as_ref().read_u32(args_ptr + 12 + (i as u32 * 4))?;
            args.push(arg);
        }
        
        tracing::info!("OpenFirmware call: {} (n_args={}, n_returns={})", service, n_args, n_returns);
        tracing::debug!("  Args: {:08X?}", args);
        
        // For string arguments, read them from memory
        let string_args = self.read_string_args(&service, &args)?;
        
        // Call the service
        let result = if let Some(of) = &mut self.openfirmware {
            of.call_client_service(&service, &args, &string_args)?
        } else {
            crate::openfirmware::ServiceResult::new(vec![u32::MAX]) // Return error if OF not available
        };
        
        // Handle memory writes if any
        if let Some((addr, data)) = result.memory_write {
            tracing::debug!("  Writing {} bytes to 0x{:08X}", data.len(), addr);
            for (i, &byte) in data.iter().enumerate() {
                self.memory.as_ref().write_u8(addr + i as u32, byte)?;
            }
        }
        
        // Write return values back to memory
        let returns_offset = args_ptr + 12 + (n_args as u32 * 4);
        for (i, &ret_val) in result.returns.iter().enumerate().take(n_returns) {
            self.memory.as_ref().write_u32(returns_offset + (i as u32 * 4), ret_val)?;
            tracing::debug!("  Return[{}] = 0x{:08X}", i, ret_val);
        }
        
        // Return from the call (restore LR)
        if let Some(cpu) = &mut self.cpu {
            cpu.registers.pc = cpu.registers.lr;
        }
        
        Ok(())
    }
    
    /// Read a null-terminated C string from memory
    fn read_cstring(&self, ptr: u32) -> Result<String> {
        use newton_cpu::MemoryInterface;
        
        let mut bytes = Vec::new();
        let mut offset = 0;
        loop {
            let byte = self.memory.as_ref().read_u8(ptr + offset)?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
            offset += 1;
            if offset > 256 {
                break; // Safety limit
            }
        }
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }
    
    /// Read string arguments for specific services
    fn read_string_args(&self, service: &str, args: &[u32]) -> Result<Vec<String>> {
        let mut strings = Vec::new();
        
        match service {
            "finddevice" if !args.is_empty() => {
                // First arg is device path pointer
                strings.push(self.read_cstring(args[0])?);
            }
            "getprop" if args.len() >= 2 => {
                // Second arg is property name pointer
                strings.push(String::new()); // phandle (not a string)
                strings.push(self.read_cstring(args[1])?);
            }
            "getproplen" if args.len() >= 2 => {
                // Second arg is property name pointer
                strings.push(String::new()); // phandle (not a string)
                strings.push(self.read_cstring(args[1])?);
            }
            _ => {}
        }
        
        Ok(strings)
    }
}

