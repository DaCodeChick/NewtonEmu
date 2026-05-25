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
use crate::mixed_mode::MixedModeManager;
use newton_cpu::{Cpu, PpcModel};
use newton_m68k::M68k;
use newton_devices::video::{Framebuffer, ColorDepth, FramebufferMmio, TextConsole};
use newton_devices::adb::{AdbController, AdbKeyboard, AdbMouse};
use newton_devices::storage::{StorageBus, MeshController};
use newton_utils::Result;
use parking_lot::RwLock;
use std::sync::Arc;

/// OpenFirmware client interface entry point address
/// Located in low RAM where we have a stub handler
// OpenFirmware client interface is at RAM_BASE + 0x3000
const OF_CLIENT_INTERFACE_OFFSET: u32 = 0x3000;

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
    
    /// 68k CPU for mixed-mode execution
    m68k: M68k,
    
    /// Mixed Mode Manager for PowerPC/68k transitions
    mixed_mode: MixedModeManager,
    
    /// CPU thread (only used in multi-threaded mode)
    cpu_thread: Option<CpuThread>,
    
    /// Memory system (Arc for sharing with CPU thread)
    memory: Arc<Memory>,
    
    /// Framebuffer (Arc for sharing with display and MMIO)
    framebuffer: Arc<RwLock<Framebuffer>>,
    
    /// Text console for OpenFirmware output
    text_console: Arc<RwLock<TextConsole>>,
    
    /// Storage bus (SCSI and IDE devices, Arc+RwLock for sharing with MESH controller)
    storage_bus: Arc<RwLock<StorageBus>>,
    
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
        let mut cpu = Cpu::new(cpu_model);
        
        // Enable execution tracing for debugging
        cpu.trace_buffer.enable();
        tracing::info!("CPU execution tracing enabled (buffer size: 200)");
        
        // Create 68k CPU for mixed-mode execution
        // Mac OS 9 typically ran on 68040-class CPUs for Toolbox code
        let m68k = M68k::new(newton_m68k::M68kModel::M68040);
        
        // Create Mixed Mode Manager
        let mixed_mode = MixedModeManager::new();
        
        // Create memory (memory-mapped, wrapped in Arc for thread sharing)
        let ram_size = config.memory.ram_size_mb * 1024 * 1024;
        let mut memory = Memory::new(ram_size)?;
        
        tracing::info!("RAM mapped to: {}", memory.ram_path().display());
        
        // Load ROM if specified
        let mut openfirmware = None;
        if let Some(rom_path) = &config.memory.rom_path {
            let mut rom = Rom::load_from_file(rom_path)?;
            
            // Apply ROM patches for NewWorld ROMs
            let ram_size = memory.ram_size() as u32;
            rom.apply_patches(crate::memory::RAM_BASE, ram_size)?;
            
            // Initialize OpenFirmware for NewWorld ROMs
            if rom.rom_type() == crate::rom::RomType::NewWorld {
                tracing::info!("NewWorld ROM detected - initializing OpenFirmware");
                openfirmware = Some(OpenFirmware::new());
            }
            
            let rom_base_addr = rom.base_address();
            
            memory.load_rom(rom);
            
            // Set up /rom/macos device tree properties if we have OpenFirmware
            if let Some(of) = &mut openfirmware {
                tracing::info!("Setting up /rom/macos device tree");
                let dt = of.device_tree_mut();
                
                // Add root node properties that the ROM expects
                tracing::info!("Looking up root node \"/\" in device tree...");
                let root_result = dt.find_node_mut("/");
                tracing::info!("Root node lookup result: {}", if root_result.is_some() { "found" } else { "NOT FOUND" });
                
                if let Some(root_node) = root_result {
                    // Add copyright property - the ROM checks for this
                    // Must match the exact format from a real Mac
                    let copyright = b"Copyright 1983-2001 Apple Computer, Inc.";
                    root_node.add_property("copyright", copyright.to_vec());
                    tracing::info!("✓ Added copyright property to root node");
                    
                    // Add memory property - describes physical memory regions
                    // Format: list of (base_address, size) pairs as big-endian u32
                    let ram_size = memory.ram_size() as u32;
                    let mut memory_prop = Vec::new();
                    // Physical memory at 0x00000000
                    memory_prop.extend_from_slice(&0u32.to_be_bytes());
                    memory_prop.extend_from_slice(&ram_size.to_be_bytes());
                    root_node.add_property("memory", memory_prop);
                    tracing::info!("✓ Added memory property: 0x00000000, size 0x{:08X}", ram_size);
                    
                    // Add AAPL,debug property - controls ROM debug output
                    // Setting to 0 for now (no debug output)
                    root_node.add_property("AAPL,debug", vec![0u8, 0, 0, 0]);
                    tracing::info!("✓ Added AAPL,debug property");
                    
                    // Add stdout property - phandle to output device
                    // For now, just use 0 (no output device)
                    // TODO: Should point to an actual console device
                    root_node.add_property("stdout", vec![0u8, 0, 0, 0]);
                    tracing::info!("✓ Added stdout property");
                    
                    // Add rtas-size property - size of RTAS code/data region
                    // RTAS (Runtime Abstraction Services) provides runtime services
                    // For now, set to a reasonable size (64KB)
                    let rtas_size = 0x10000u32; // 64KB
                    root_node.add_property("rtas-size", rtas_size.to_be_bytes().to_vec());
                    tracing::info!("✓ Added rtas-size property: 0x{:X} bytes", rtas_size);
                    
                    // Add AAPL,writable-ROM-aperture property
                    // This tells the ROM where it can write the decompressed Toolbox
                    // The traditional Mac ROM base is 0xFFC00000
                    let aperture_base = 0xFFC00000u32;
                    let aperture_size = 0x400000u32; // 4MB
                    let mut aperture = Vec::new();
                    aperture.extend_from_slice(&aperture_base.to_be_bytes());
                    aperture.extend_from_slice(&aperture_size.to_be_bytes());
                    root_node.add_property("AAPL,writable-ROM-aperture", aperture);
                    tracing::info!("✓ Added AAPL,writable-ROM-aperture: 0x{:08X}, size 0x{:X}", 
                                  aperture_base, aperture_size);
                } else {
                    tracing::warn!("Root node not found!");
                }
                
                // Create /rom device if it doesn't exist
                if dt.find_node("/rom").is_none() {
                    tracing::info!("Creating /rom device node");
                    let mut rom_node = crate::openfirmware::DeviceNode::new("rom", "rom");
                    rom_node.add_property("name", b"rom");
                    rom_node.add_property("device_type", b"rom");
                    dt.add_node("/rom", rom_node);
                }
                
                // Create /rom/macos device
                if dt.find_node("/rom/macos").is_none() {
                    tracing::info!("Creating /rom/macos device node");
                    let mut macos_node = crate::openfirmware::DeviceNode::new("macos", "");
                    macos_node.add_property("name", b"macos");
                    
                    // For NewWorld ROMs, we need to tell the ROM code where the compressed
                    // LZSS toolbox data is located. The boot script expects:
                    // "AAPL,toolbox-image,lzss" property with [address, size]
                    
                    // The LZSS data is at an offset within the ROM file
                    // We need to find the lzss-offset and lzss-size from the boot script
                    if let Some(rom_ref) = memory.rom() {
                        if let Some(boot_script) = rom_ref.get_boot_script() {
                            tracing::debug!("Got boot script, length: {}", boot_script.len());
                            
                            let lzss_offset = Self::find_boot_constant(&boot_script, "lzss-offset")
                                .or_else(|| Self::find_boot_constant(&boot_script, "parcels-offset"))
                                .unwrap_or(0x015690);
                            
                            let lzss_size = Self::find_boot_constant(&boot_script, "lzss-size")
                                .or_else(|| Self::find_boot_constant(&boot_script, "parcels-size"))
                                .unwrap_or(0x208880);
                            
                            tracing::info!("Parsed boot constants: lzss_offset=0x{:X}, lzss_size=0x{:X}", 
                                          lzss_offset, lzss_size);
                            
                            // The address is ROM base + LZSS offset
                            let lzss_address = rom_base_addr.wrapping_add(lzss_offset as u32);
                            
                            // Format: [address, size] as big-endian 32-bit values
                            let mut toolbox_image = Vec::new();
                            toolbox_image.extend_from_slice(&lzss_address.to_be_bytes());
                            toolbox_image.extend_from_slice(&(lzss_size as u32).to_be_bytes());
                            
                            macos_node.add_property("AAPL,toolbox-image,lzss", toolbox_image);
                            tracing::info!("✓ Set AAPL,toolbox-image,lzss property: address=0x{:08X}, size=0x{:X}", 
                                          lzss_address, lzss_size);
                            
                            // Let the ROM decompress the Toolbox itself
                            // The ROM will decompress the LZSS data from AAPL,toolbox-image,lzss
                            // into the writable ROM aperture at the address from AAPL,writable-ROM-aperture
                            if let Ok(_lzss_data) = rom_ref.read_range(lzss_offset as usize, lzss_size as usize) {
                                tracing::info!("ROM will decompress {} bytes of LZSS data from 0x{:08X}", 
                                              lzss_size, lzss_offset);
                                tracing::info!("ROM will write decompressed data to writable ROM aperture at 0xFFC00000");
                                
                                /* COMMENTED OUT: Pre-decompression - let ROM do it instead
                                // Check if it's a 'prcl' parcel container
                                let magic = if lzss_data.len() >= 4 {
                                    std::str::from_utf8(&lzss_data[0..4]).unwrap_or("????")
                                } else {
                                    "????"
                                };
                                tracing::info!("Compressed data magic: '{}'", magic);
                                
                                let decompressed = if magic == "prcl" {
                                    tracing::info!("Parsing parcel structure to find 'rom ' parcel");
                                    Self::decode_rom_from_parcels(&lzss_data)
                                } else {
                                    tracing::info!("Using decompress_lzss for raw LZSS format");
                                    crate::lzss::decompress_lzss(&lzss_data)
                                };
                                
                                match decompressed {
                                    Ok(decompressed) => {
                                        tracing::info!("Toolbox decompressed: {} bytes -> {} bytes", 
                                                      lzss_data.len(), decompressed.len());
                                        
                                        // Debug: show first 512 bytes as ASCII for analysis
                                        let preview_len = std::cmp::min(512, decompressed.len());
                                        let preview_str = String::from_utf8_lossy(&decompressed[..preview_len]);
                                        tracing::debug!("Decompressed data preview (first {} bytes):\n{}", preview_len, preview_str);
                                        
                                        // Check for copyright string
                                        let found_copyright = if let Some(pos) = decompressed.windows(b"Copyright (c)".len())
                                            .position(|w| w == b"Copyright (c)") {
                                            tracing::info!("✓ Found ASCII copyright at offset 0x{:X}", pos);
                                            
                                            // Show the full copyright string
                                            let copyright_end = std::cmp::min(pos + 100, decompressed.len());
                                            let copyright_str = String::from_utf8_lossy(&decompressed[pos..copyright_end]);
                                            tracing::info!("  Copyright text: {}", copyright_str.lines().next().unwrap_or(""));
                                            true
                                        } else {
                                            tracing::warn!("⚠ No copyright string found in decompressed data!");
                                            false
                                        };
                                        
                                        if !found_copyright {
                                            tracing::warn!("This may indicate incorrect decompression or wrong data");
                                        }
                                        
                                        // Write decompressed data to ROM shadow at 0xFFC00000
                                        // This is the traditional Mac ROM base address where the ROM expects
                                        // to find the decompressed 68k Toolbox
                                        let traditional_rom_base = 0xFFC00000u32;
                                        tracing::info!("Writing {} bytes to ROM shadow at 0x{:08X}", 
                                                      decompressed.len(), traditional_rom_base);
                                        
                                        if let Err(e) = memory.write_to_rom_shadow(traditional_rom_base, &decompressed) {
                                            tracing::error!("❌ Failed to write decompressed Toolbox to ROM shadow: {}", e);
                                        } else {
                                            tracing::info!("✓ Wrote decompressed Toolbox to ROM shadow");
                                            
                                            // Verify we can read it back
                                            use newton_cpu::MemoryInterface;
                                            if let Ok(first_bytes) = memory.read_u32(traditional_rom_base) {
                                                tracing::info!("✓ Verified ROM shadow readable: first word = 0x{:08X}", first_bytes);
                                            } else {
                                                tracing::error!("❌ Cannot read back from ROM shadow!");
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("❌ Failed to decompress Toolbox data: {}", e);
                                    }
                                }
                                */
                            } else {
                                tracing::error!("❌ Failed to read LZSS data from ROM");
                            }
                        } else {
                            tracing::warn!("Failed to get boot script from ROM!");
                        }
                    } else {
                        tracing::warn!("Failed to get ROM reference!");
                    }
                    
                    dt.add_node("/rom/macos", macos_node);
                }
            }
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
        
        // Create text console for OpenFirmware output
        let text_console = Arc::new(RwLock::new(TextConsole::new(Arc::clone(&framebuffer))));
        
        // Create storage bus (before MESH, so MESH can reference it)
        let storage_bus = Arc::new(RwLock::new(StorageBus::new()));
        
        // Create MESH SCSI controller and connect it to storage bus
        let mut mesh = MeshController::new();
        mesh.set_storage_bus(Arc::clone(&storage_bus));
        memory.register_mmio(
            MESH_SCSI_BASE_ADDR,
            MESH_SCSI_SIZE,
            Box::new(mesh),
        );
        
        let memory = Arc::new(memory);
        
        tracing::info!("Storage bus initialized");
        
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
            m68k,
            mixed_mode,
            cpu_thread: cpu_thread_opt,
            memory,
            framebuffer,
            text_console,
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
                            cpu.registers.gpr[2] = 0;  // Let ROM initialize its own TOC
                            tracing::info!("NewWorld ROM: Set PC to ROM entry 0x{:08X}", rom_entry);
                        }
                        
                        // Set up DBAT0 to map ROM shadow region (0xFFC00000-0xFFFFFFFF)
                        // This allows the ROM to access and write to the writable ROM aperture
                        // Size: 4MB (0x400000 bytes)
                        if let Err(e) = cpu.setup_bat(
                            0,              // DBAT0
                            true,           // Data BAT
                            0xFFC00000,     // Virtual address
                            0xFFC00000,     // Physical address (identity mapping)
                            4 * 1024 * 1024, // 4MB
                            true,           // Writable
                            true,           // Valid in supervisor mode
                            false,          // Not valid in user mode
                        ) {
                            tracing::error!("Failed to set up DBAT0 for ROM shadow: {}", e);
                        }
                        
                        // Also set up IBAT0 to map ROM region for instruction fetch
                        if let Err(e) = cpu.setup_bat(
                            0,              // IBAT0
                            false,          // Instruction BAT
                            0xFFC00000,     // Virtual address
                            0xFFC00000,     // Physical address
                            4 * 1024 * 1024, // 4MB
                            false,          // Not writable (instructions)
                            true,           // Valid in supervisor mode
                            false,          // Not valid in user mode
                        ) {
                            tracing::error!("Failed to set up IBAT0 for ROM: {}", e);
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
                        // Put a return address pointing to our infinite loop stub
                        // PowerPC stack frame: [r1+0] = back chain, [r1+8] = LR save area
                        self.memory.init_stack_frame(stack_top, 0x1004);
                        
                        // Store OF entry point address in r5
                        // The ROM will look for this to call OpenFirmware
                        cpu.registers.gpr[5] = OF_CLIENT_INTERFACE_OFFSET;
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
                        let of_client_addr = OF_CLIENT_INTERFACE_OFFSET;
                        if pc == of_client_addr {
                            let r3 = cpu.registers.gpr[3];
                            tracing::debug!("OpenFirmware client interface call at PC=0x{:08X}, r3=0x{:08X}", pc, r3);
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
                        let of_client_addr = OF_CLIENT_INTERFACE_OFFSET;
                        if self.openfirmware.is_some() && pc == of_client_addr {
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
    
    /// Get 68k CPU reference
    pub fn m68k(&self) -> &M68k {
        &self.m68k
    }
    
    /// Get mutable 68k CPU reference
    pub fn m68k_mut(&mut self) -> &mut M68k {
        &mut self.m68k
    }
    
    /// Get Mixed Mode Manager reference
    pub fn mixed_mode(&self) -> &MixedModeManager {
        &self.mixed_mode
    }
    
    /// Get mutable Mixed Mode Manager reference
    pub fn mixed_mode_mut(&mut self) -> &mut MixedModeManager {
        &mut self.mixed_mode
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
    pub fn storage_bus(&self) -> Arc<RwLock<StorageBus>> {
        Arc::clone(&self.storage_bus)
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
        
        if service_ptr == 0 {
            tracing::warn!("OF call with null service pointer! args_ptr=0x{:08X}", args_ptr);
            return Ok(());
        }
        
        tracing::debug!("OF call: args_ptr=0x{:08X}, service_ptr=0x{:08X}, n_args={}, n_returns={}", 
                       args_ptr, service_ptr, n_args, n_returns);
        
        // Read service name from memory (null-terminated string)
        let service = self.read_cstring(service_ptr)?;
        
        if service.is_empty() {
            tracing::warn!("OF call with empty service name! service_ptr=0x{:08X}", service_ptr);
            // Try to read raw bytes to see what's there
            let byte1 = self.memory.as_ref().read_u8(service_ptr).unwrap_or(0xFF);
            let byte2 = self.memory.as_ref().read_u8(service_ptr + 1).unwrap_or(0xFF);
            let byte3 = self.memory.as_ref().read_u8(service_ptr + 2).unwrap_or(0xFF);
            let byte4 = self.memory.as_ref().read_u8(service_ptr + 3).unwrap_or(0xFF);
            tracing::warn!("  Bytes at service_ptr: {:02X} {:02X} {:02X} {:02X}", byte1, byte2, byte3, byte4);
            return Ok(());
        }
        
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
        
        // Intercept "write" calls to render to text console
        if service == "write" && !string_args.is_empty() {
            let text = &string_args[0];
            tracing::debug!("Text console write: {:?}", text);
            let mut console = self.text_console.write();
            console.write_str(text);
        }
        
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
            "call-method" if !args.is_empty() => {
                // args: [method_name_ptr, ihandle, ...method_args]
                // First arg is method name pointer
                strings.push(self.read_cstring(args[0])?);
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
        
        // Extract constants from the ROM
        let elf_offset = 0x5000u32;  // From ROM analysis
        let elf_size = 0x20000u32;   // 128KB should be enough for ELF headers and segments
        
        // The load-base is where we'll copy the ELF to before parsing
        // Standard Mac boot allocates at 0x00400000 (4MB)
        let load_base = 0x00400000u32;
        
        // Create a real boot script that:
        // 1. Copies ELF from ROM to load-base
        // 2. Calls init-program to parse it
        // 3. Calls go to start execution
        let boot_script_code = format!(r#"
            hex
            {:08X} constant elf-offset
            {:08X} constant elf-size
            {:08X} constant load-base
            
            \ Allocate memory for ELF at load-base
            \ In a real OF implementation, this would call claim
            \ For now, we assume the memory is available
            
            \ Copy ELF from ROM to load-base
            \ ROM is mapped at high memory, we need to copy the ELF portion
            \ This is done by the Forth interpreter's memory operations
            
            \ Parse and load the ELF
            init-program
            
            \ Start execution
            go
        "#, elf_offset, elf_size, load_base);
        
        tracing::info!("Executing boot script");
        tracing::debug!("Boot script code:\n{}", boot_script_code);
        
        let of = self.openfirmware.as_mut()?;
        
        match of.execute_forth(&boot_script_code) {
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
        let elf_offset = rom.get_elf_offset()?;
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
                        tracing::info!("  Boot script specified load-base: 0x{:08X}", load_base);
                        
                        // Load at ELF's expected address since it contains hardcoded absolute addresses
                        let target_addr = elf_load_addr;
                        
                        tracing::info!("  Loading ELF at expected address: 0x{:08X}", target_addr);
                        tracing::info!("  Writing {} bytes to 0x{:08X}", memory_image.len(), target_addr);
                        
                        for (i, &byte) in memory_image.iter().enumerate() {
                            if let Err(e) = self.memory.write_u8(target_addr + i as u32, byte) {
                                tracing::error!("  Failed to write byte at 0x{:08X}: {}", 
                                              target_addr + i as u32, e);
                                return None;
                            }
                        }
                        
                        tracing::info!("  ✅ ELF loaded successfully!");
                        
                        // Debug: Check what's at critical addresses AFTER loading
                        use newton_cpu::MemoryInterface;
                        
                        let check_addrs = [
                            (0x00100130, "indirect call source"),
                            (0x001155D0, "function descriptor table"),
                            (0x001155DC, "descriptor pointer"),
                        ];
                        
                        for (addr, desc) in check_addrs.iter() {
                            if let Ok(value) = self.memory.as_ref().read_u32(*addr) {
                                tracing::info!("  Debug: 0x{:08X} ({}) = 0x{:08X}", addr, desc, value);
                            }
                        }
                        
                        // Return the entry point from the ELF
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
    
    /// Attach a disk image to the storage bus
    /// 
    /// This automatically detects the image format and attaches it to the appropriate bus.
    /// Supported formats: ISO, DMG, Toast, ZIP, and raw disk images.
    pub fn attach_disk_image<P: AsRef<std::path::Path>>(&mut self, path: P, scsi_id: u8, readonly: bool) -> Result<()> {
        use newton_devices::storage::{BlockDevice, IsoImage, DmgImage, ToastImage, ZipImage, RawDiskImage};
        use std::sync::Arc;
        use parking_lot::RwLock;
        
        let path_ref = path.as_ref();
        let path_str = path_ref.to_string_lossy();
        let path_lower = path_str.to_lowercase();
        
        tracing::info!("Attaching disk image: {}", path_str);
        tracing::info!("  SCSI ID: {}", scsi_id);
        tracing::info!("  Read-only: {}", readonly);
        
        // Determine format and open the appropriate image type
        let device: Arc<RwLock<dyn BlockDevice>> = if path_lower.ends_with(".iso") {
            tracing::info!("  Format: ISO9660");
            let iso = IsoImage::open(path_ref)?;
            Arc::new(RwLock::new(iso))
        } else if path_lower.ends_with(".dmg") {
            tracing::info!("  Format: Apple Disk Image (DMG)");
            let dmg = DmgImage::open(path_ref)?;
            Arc::new(RwLock::new(dmg))
        } else if path_lower.ends_with(".toast") {
            tracing::info!("  Format: Roxio Toast");
            let toast = ToastImage::open(path_ref)?;
            Arc::new(RwLock::new(toast))
        } else if path_lower.ends_with(".zip") {
            tracing::info!("  Format: ZIP archive");
            let zip = ZipImage::open(path_ref)?;
            Arc::new(RwLock::new(zip))
        } else {
            tracing::info!("  Format: Raw disk image");
            let raw = RawDiskImage::open(path_ref, !readonly)?;
            Arc::new(RwLock::new(raw))
        };
        
        // Get device info for logging
        {
            let dev_guard = device.read();
            let info = dev_guard.info();
            tracing::info!("  Device type: {:?}", info.device_type);
            tracing::info!("  Model: {}", info.model);
            tracing::info!("  Size: {} MB", info.size / (1024 * 1024));
            tracing::info!("  Block size: {} bytes", info.block_size);
        }
        
        // Attach to SCSI bus
        self.storage_bus.write().attach_scsi(scsi_id, device)?;
        tracing::info!("✅ Disk image attached successfully");
        
        Ok(())
    }
    
    /// Attach a boot CD/DVD image
    /// This is a convenience method that attaches to SCSI ID 3 (typical CD-ROM)
    pub fn attach_boot_cd<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        self.attach_disk_image(path, 3, true)
    }
    
    /// Attach a boot disk image
    /// This is a convenience method that attaches to SCSI ID 0 (typical boot disk)
    pub fn attach_boot_disk<P: AsRef<std::path::Path>>(&mut self, path: P, readonly: bool) -> Result<()> {
        self.attach_disk_image(path, 0, readonly)
    }
    
    /// Parse a constant from the OpenFirmware boot script
    /// 
    /// Looks for patterns like "h# 015690 constant lzss-offset"
    /// and returns the hex value
    fn find_boot_constant(boot_script: &str, name: &str) -> Option<u32> {
        // Look for exact pattern: "h# <hex> constant <name>"
        // The hex value must be immediately after "h# " and before " constant"
        let constant_pattern = format!(" constant {}", name);
        
        for line in boot_script.lines() {
            // Find "constant <name>" first
            if let Some(const_pos) = line.find(&constant_pattern) {
                // Now look backwards for "h# " before the constant
                let before_const = &line[..const_pos];
                if let Some(h_pos) = before_const.rfind("h# ") {
                    // Extract hex value between "h# " and " constant"
                    let hex_str = before_const[h_pos + 3..].trim();
                    tracing::debug!("Found line with {}: {}", name, line.trim());
                    tracing::debug!("Parsing hex string: '{}'", hex_str);
                    if let Ok(value) = u32::from_str_radix(hex_str, 16) {
                        tracing::info!("Found boot constant {}: 0x{:X}", name, value);
                        return Some(value);
                    }
                }
            }
        }
        tracing::warn!("Could not find boot constant: {}", name);
        None
    }
    
    /// Decode ROM data from parcel structure
    /// NewWorld ROMs use a 'prcl' parcel container with a 'rom ' parcel containing LZSS-compressed data
    fn decode_rom_from_parcels(data: &[u8]) -> Result<Vec<u8>> {
        use byteorder::{BigEndian, ByteOrder};
        
        let mut parcel_offset = 0x14; // First parcel at offset 0x14
        
        while parcel_offset != 0 && parcel_offset + 24 <= data.len() {
            let next_offset = BigEndian::read_u32(&data[parcel_offset..]) as usize;
            let parcel_type = BigEndian::read_u32(&data[parcel_offset + 4..]);
            
            tracing::debug!("Parcel at 0x{:X}: type=0x{:08X} ('{}')", 
                           parcel_offset, parcel_type,
                           String::from_utf8_lossy(&parcel_type.to_be_bytes()));
            
            // Look for 'rom ' parcel (0x726F6D20)
            if parcel_type == 0x726F6D20 {
                let lzss_offset = BigEndian::read_u32(&data[parcel_offset + 8..]) as usize;
                let abs_offset = parcel_offset + lzss_offset;
                
                if next_offset > abs_offset && next_offset <= data.len() {
                    let lzss_size = next_offset - abs_offset;
                    tracing::info!("Found 'rom ' parcel: LZSS at 0x{:X}, size 0x{:X}", abs_offset, lzss_size);
                    
                    // Decompress the LZSS data
                    return crate::lzss::decompress_lzss(&data[abs_offset..abs_offset + lzss_size]);
                } else {
                    tracing::warn!("Invalid 'rom ' parcel: next_offset={}, abs_offset={}", next_offset, abs_offset);
                }
            }
            
            parcel_offset = next_offset;
        }
        
        Err(newton_utils::Error::Other("No 'rom ' parcel found in parcel data".to_string()))
    }
}

