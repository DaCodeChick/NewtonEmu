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
use crate::debugger::Debugger;
use crate::memory::Memory;
use crate::rom::Rom;
use crate::openfirmware::OpenFirmware;
use newton_cpu::{Cpu, PpcModel};
use newton_devices::video::{Framebuffer, ColorDepth, FramebufferMmio};
use newton_devices::adb::{AdbController, AdbKeyboard, AdbMouse};
use newton_devices::storage::{StorageBus, MeshController};
use newton_utils::Result;
use parking_lot::RwLock;
use std::sync::Arc;

/// OpenFirmware client interface entry point address
/// Located in low RAM where we have a stub handler
const OF_CLIENT_INTERFACE_ADDR: u32 = 0x3000;

/// Framebuffer base address in emulated address space
/// Placed at 8MB boundary for easy access
const FRAMEBUFFER_BASE_ADDR: u32 = 0x00800000;

/// MESH SCSI controller base address
/// Typical address for MESH on PowerPC Macs
const MESH_SCSI_BASE_ADDR: u32 = 0xF3000000;

/// MESH SCSI controller register space size
const MESH_SCSI_SIZE: u32 = 0x1000;

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
    
    /// Framebuffer (Arc for sharing with display and MMIO)
    framebuffer: Arc<RwLock<Framebuffer>>,
    
    /// Storage bus (SCSI and IDE devices)
    storage_bus: StorageBus,
    
    /// ADB controller (not yet integrated)
    _adb: AdbController,
    
    /// OpenFirmware
    openfirmware: Option<OpenFirmware>,
    
    /// System debugger
    debugger: Option<Debugger>,
    
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
        
        // Create memory (memory-mapped, wrapped in Arc for thread sharing)
        let ram_size = config.memory.ram_size_mb * 1024 * 1024;
        let mut memory = Memory::new(ram_size)?;
        
        tracing::info!("RAM mapped to: {}", memory.ram_path().display());
        
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
        
        // Create framebuffer
        let depth = match config.display.color_depth {
            8 => ColorDepth::Indexed8,
            16 => ColorDepth::Rgb16,
            32 => ColorDepth::Rgba32,
            _ => ColorDepth::Rgba32,
        };
        let framebuffer = Arc::new(RwLock::new(Framebuffer::new(
            config.display.width,
            config.display.height,
            depth,
        )));
        
        // Register framebuffer MMIO device
        let fb_size = {
            let fb = framebuffer.read();
            let (width, height) = fb.dimensions();
            width * height * depth.bytes_per_pixel() as u32
        };
        memory.register_mmio(
            FRAMEBUFFER_BASE_ADDR,
            fb_size,
            Box::new(FramebufferMmio::new(Arc::clone(&framebuffer))),
        );
        
        // Create MESH SCSI controller
        let mesh = MeshController::new();
        memory.register_mmio(
            MESH_SCSI_BASE_ADDR,
            MESH_SCSI_SIZE,
            Box::new(mesh),
        );
        
        let memory = Arc::new(memory);
        
        // Create storage bus
        let storage_bus = StorageBus::new();
        
        // Attach storage devices from config
        // TODO: Load from config.storage devices
        tracing::info!("Storage bus initialized (no devices attached yet)");
        
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
            storage_bus,
            _adb: adb,
            openfirmware,
            debugger: None,  // Debugger disabled by default
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
                    
                    // For NewWorld ROMs, override the PC to point to the ROM entry point
                    // NewWorld ROMs don't use the traditional 0xFFF00100 reset vector
                    if self.openfirmware.is_some() {
                        if let Some(rom) = self.memory.rom() {
                            let rom_entry = rom.entry_address();
                            let _rom_base = rom.base_address();
                            cpu.registers.pc = rom_entry;
                            tracing::info!("NewWorld ROM: Set PC to ROM entry 0x{:08X}", rom_entry);
                            
                            // Set up r2 (TOC/globals pointer) for Mac ROM
                            // Mac ROMs use r2 to access system globals and function tables
                            // Point to globals structure initialized in RAM at 0x5100
                            cpu.registers.gpr[2] = 0x5100;
                            tracing::info!("Set r2 (globals pointer) to 0x{:08X}", cpu.registers.gpr[2]);
                        }
                        
                        // Set up initial stack pointer in high RAM
                        // Stack grows downward from near top of RAM
                        let ram_size = self.memory.ram_size() as u32;
                        let stack_top = ram_size - 0x1000;  // Leave 4KB at top
                        cpu.registers.gpr[1] = stack_top;
                        tracing::info!("Set initial stack pointer to 0x{:08X}", stack_top);
                        
                        // Set up Link Register to point to infinite loop stub
                        // When ROM entry function returns, it will go to this stub
                        cpu.registers.lr = 0x1004;
                        tracing::info!("Set Link Register (LR) to 0x{:08X} (infinite loop stub)", cpu.registers.lr);
                        
                        // Set up an initial stack frame so returns don't crash
                        // Put a return address pointing to our infinite loop stub at 0x1004
                        // PowerPC stack frame: [r1+0] = back chain, [r1+8] = LR save area
                        self.memory.init_stack_frame(stack_top, 0x1004);
                        
                        // Store OF entry point address in r5
                        // The ROM will look for this to call OpenFirmware
                        cpu.registers.gpr[5] = OF_CLIENT_INTERFACE_ADDR;
                        tracing::info!("Set OpenFirmware entry point to 0x{:08X} (in r5)", cpu.registers.gpr[5]);
                    }
                }
                
                // Execute boot script after CPU initialization
                if self.openfirmware.is_some() {
                    let program_entry = self.execute_boot_script_and_get_entry();
                    
                    // Check if boot script set a program entry point
                    if let Some(entry) = program_entry {
                        tracing::info!("Boot script completed - program entry point: 0x{:08X}", entry);
                        tracing::info!("Overriding PC to jump to loaded program");
                        if let Some(cpu) = &mut self.cpu {
                            cpu.registers.pc = entry;
                        }
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

    /// Enable the system debugger
    pub fn enable_debugger(&mut self) {
        if self.debugger.is_none() {
            tracing::info!("Enabling system debugger");
            self.debugger = Some(Debugger::new());
        }
    }

    /// Disable the system debugger
    pub fn disable_debugger(&mut self) {
        if self.debugger.is_some() {
            tracing::info!("Disabling system debugger");
            self.debugger = None;
        }
    }

    /// Get a reference to the debugger (if enabled)
    pub fn debugger(&self) -> Option<&Debugger> {
        self.debugger.as_ref()
    }

    /// Get a mutable reference to the debugger (if enabled)
    pub fn debugger_mut(&mut self) -> Option<&mut Debugger> {
        self.debugger.as_mut()
    }

    /// Execute a single instruction
    pub fn step(&mut self) -> Result<()> {
        match self.mode {
            EmulatorMode::SingleThreaded => {
                if let Some(cpu) = &mut self.cpu {
                    let pc = cpu.registers.pc;
                    
                    // Debugger hook: check before instruction execution
                    if let Some(debugger) = &mut self.debugger {
                        if debugger.before_instruction(pc) {
                            // Debugger wants to pause
                            return Ok(());
                        }
                    }
                    
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
                if self.cpu.is_some() {
                    for _ in 0..cycles {
                        let pc = self.cpu.as_ref().unwrap().registers.pc;
                        
                        // Debugger hook: check before instruction execution
                        if let Some(debugger) = &mut self.debugger {
                            if debugger.before_instruction(pc) {
                                // Debugger wants to pause
                                break;
                            }
                        }
                        
                        // Check for OpenFirmware intercept
                        if self.openfirmware.is_some() && pc == OF_CLIENT_INTERFACE_ADDR {
                            self.handle_openfirmware_call()?;
                        } else {
                            if let Some(cpu) = &mut self.cpu {
                                cpu.step(&*self.memory)?;
                            }
                        }
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
    pub fn memory(&self) -> Arc<Memory> {
        Arc::clone(&self.memory)
    }

    /// Get framebuffer reference (Arc for sharing)
    pub fn framebuffer(&self) -> Arc<RwLock<Framebuffer>> {
        Arc::clone(&self.framebuffer)
    }
    
    /// Get storage bus reference
    pub fn storage_bus(&self) -> &StorageBus {
        &self.storage_bus
    }
    
    /// Get mutable storage bus reference
    pub fn storage_bus_mut(&mut self) -> &mut StorageBus {
        &mut self.storage_bus
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
        tracing::debug!("  String args: {:?}", string_args);
        
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
        
        tracing::debug!("read_cstring from 0x{:08X}", ptr);
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
        let result = String::from_utf8_lossy(&bytes).to_string();
        tracing::debug!("  -> read {} bytes: {:?}", bytes.len(), result);
        Ok(result)
    }
    
    /// Read string arguments for specific services
    fn read_string_args(&self, service: &str, args: &[u32]) -> Result<Vec<String>> {
        use newton_cpu::MemoryInterface;
        
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
            "write" if args.len() >= 3 => {
                // args: [ihandle, buf_ptr, len]
                // Read the buffer content
                let buf_ptr = args[1];
                let len = args[2].min(4096); // Safety limit
                let mut bytes = Vec::new();
                for i in 0..len {
                    if let Ok(byte) = self.memory.as_ref().read_u8(buf_ptr + i) {
                        bytes.push(byte);
                    } else {
                        break;
                    }
                }
                strings.push(String::from_utf8_lossy(&bytes).to_string());
            }
            _ => {}
        }
        
        Ok(strings)
    }
    
    /// Execute the boot script from ROM and return the program entry point
    fn execute_boot_script_and_get_entry(&mut self) -> Option<u32> {
        tracing::info!("Executing boot script from ROM");
        
        // Get ROM and extract boot script
        let (boot_script, rom_size) = if let Some(rom) = self.memory.rom() {
            let script = match rom.get_boot_script() {
                Some(script) => script,
                None => {
                    tracing::error!("Failed to extract boot script");
                    return None;
                }
            };
            let size = rom.data().len() as u32;
            (script, size)
        } else {
            tracing::warn!("No ROM loaded - cannot execute boot script");
            return None;
        };
        
        tracing::info!("Boot script extracted: {} bytes", boot_script.len());
        
        // Set up constants that the boot script expects
        let load_base = 0x00400000u32;  // 4MB load address
        
        // Create simplified boot script that just sets up constants
        // and calls init-program/go
        // The full boot script is too complex and requires many unimplemented services
        let simplified_script = format!(r#"
            hex
            {:08X} constant load-base
            {:08X} constant load-size
            004000 constant elf-offset
            011690 constant elf-size
            015690 constant lzss-offset
            208880 constant lzss-size
            
            init-program
            go
        "#, load_base, rom_size);
        
        tracing::info!("Executing simplified boot script");
        tracing::debug!("Boot script:\n{}", simplified_script);
        
        let of = self.openfirmware.as_mut()?;
        
        match of.execute_forth(&simplified_script) {
            Ok(()) => {
                tracing::info!("Boot script executed successfully");
                
                // Check results
                let forth = of.forth();
                let program_entry = forth.program_entry;
                let load_base = forth.load_base;
                
                if let Some(entry) = program_entry {
                    tracing::info!("  Program entry: 0x{:08X}", entry);
                } else {
                    tracing::warn!("  No program entry set");
                }
                
                if let Some(base) = load_base {
                    tracing::info!("  Load base: 0x{:08X}", base);
                    
                    // Now parse the ELF at load-base and get the real entry point
                    if let Some(real_entry) = self.parse_elf_at_load_base(base) {
                        tracing::info!("  ELF entry point: 0x{:08X}", real_entry);
                        return Some(real_entry);
                    }
                }
                
                program_entry
            }
            Err(e) => {
                tracing::error!("Boot script execution failed: {}", e);
                None
            }
        }
    }
    
    /// Parse ELF from ROM and load it into memory
    fn parse_elf_at_load_base(&self, load_base: u32) -> Option<u32> {
        use newton_cpu::MemoryInterface;
        
        // Extract ELF from ROM
        let rom = self.memory.rom()?;
        
        // Automatically detect ELF offset
        let elf_offset = rom.find_elf_offset()?;
        let elf_size = 0x20000; // Read more than needed, ELF parser will handle it
        
        // Make sure we don't read past the end of ROM
        let actual_size = elf_size.min(rom.size() - elf_offset);
        let elf_data = rom.data()[elf_offset..elf_offset + actual_size].to_vec();
        
        tracing::info!("Parsing ELF from ROM (offset 0x{:X}, size 0x{:X})", elf_offset, elf_size);
        
        match crate::elf::ElfFile::parse(elf_data) {
            Ok(elf) => {
                let entry = elf.entry_point();
                tracing::info!("  ELF parsed successfully, entry point: 0x{:08X}", entry);
                
                // Load ELF into memory
                tracing::info!("Loading ELF segments into memory...");
                match elf.load_to_memory() {
                    Ok((memory_image, elf_load_addr, _elf_entry)) => {
                        tracing::info!("  ELF memory image: {} bytes", memory_image.len());
                        tracing::info!("  ELF expects to be loaded at: 0x{:08X}", elf_load_addr);
                        tracing::info!("  Actual load base (from boot script): 0x{:08X}", load_base);
                        
                        // Copy the ELF memory image to the load_base
                        // The boot script wants us to load at load_base,
                        // but the ELF has its own load address in the headers
                        // For now, load at the ELF's expected address
                        let target_addr = elf_load_addr;
                        
                        tracing::info!("  Writing {} bytes to 0x{:08X}", memory_image.len(), target_addr);
                        
                        for (i, &byte) in memory_image.iter().enumerate() {
                            if let Err(e) = self.memory.write_u8(target_addr + i as u32, byte) {
                                tracing::error!("  Failed to write byte at 0x{:08X}: {}", 
                                              target_addr + i as u32, e);
                                return None;
                            }
                        }
                        
                        tracing::info!("  ✅ ELF loaded successfully!");
                        Some(entry)
                    }
                    Err(e) => {
                        tracing::error!("  Failed to load ELF to memory: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                tracing::error!("  Failed to parse ELF: {}", e);
                None
            }
        }
    }
}

