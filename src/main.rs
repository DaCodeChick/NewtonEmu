// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! NewtonEmu - PowerPC Macintosh Emulator (CLI)
//!
//! This is the CLI-only emulator core. For GUI tools (debugger, HFS+ manager,
//! network monitor, configuration editor), see the separate Qt frontend.

use anyhow::Result;
use clap::Parser;
use newton_core::{Emulator, EmulatorConfig};
use newton_core::config::{CpuConfig, CpuModel, MemoryConfig, DisplayConfig};
use std::path::PathBuf;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

mod display;
use display::DisplayWindow;

mod config;
use config::EmulatorConfig as FileConfig;

/// NewtonEmu - PowerPC Macintosh Emulator
#[derive(Parser, Debug)]
#[command(name = "newton-emu")]
#[command(version = "0.1.0")]
#[command(about = "PowerPC Macintosh Emulator", long_about = None)]
struct Args {
    /// ROM file path
    #[arg(short, long, value_name = "FILE")]
    rom: Option<PathBuf>,

    /// Configuration file (JSON format)
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// RAM size in MB
    #[arg(long, default_value = "256")]
    ram: usize,

    /// Display width
    #[arg(long, default_value = "800")]
    width: u32,

    /// Display height
    #[arg(long, default_value = "600")]
    height: u32,

    /// Boot CD/DVD ISO
    #[arg(long, value_name = "FILE")]
    cd: Option<PathBuf>,

    /// Boot disk image
    #[arg(long, value_name = "FILE")]
    disk: Option<PathBuf>,

    /// Start paused
    #[arg(short, long)]
    paused: bool,

    /// Headless mode (no display window)
    #[arg(long)]
    headless: bool,

    /// Enable debugger IPC (stdin/stdout JSON protocol)
    #[arg(long)]
    debugger: bool,

    /// Enable GDB server
    #[arg(long, value_name = "HOST:PORT")]
    gdb_server: Option<String>,
}

struct App {
    emulator: Emulator,
    display: Option<DisplayWindow>,
    config: EmulatorConfig,
    headless: bool,
    debugger_ipc: Option<newton_core::DebuggerIpc>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.headless {
            return;
        }

        // Create display window
        if self.display.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("NewtonEmu - PowerPC Macintosh Emulator")
                .with_inner_size(winit::dpi::PhysicalSize::new(
                    self.config.display.width,
                    self.config.display.height,
                ));

            let window = event_loop.create_window(window_attributes).unwrap();

            let display = pollster::block_on(DisplayWindow::new(
                window,
                self.config.display.width,
                self.config.display.height,
            ));

            self.display = Some(display);

            tracing::info!("Display window created!");
            tracing::info!("");
            tracing::info!("Controls:");
            tracing::info!("  ESC: Quit");
            tracing::info!("");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let display = match self.display.as_mut() {
            Some(d) => d,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Window closed, exiting...");
                event_loop.exit();
            }

            WindowEvent::Resized(physical_size) => {
                display.resize(physical_size.width, physical_size.height);
            }

            WindowEvent::RedrawRequested => {
                // Process debugger commands if debugger is enabled
                if let Some(ref mut ipc) = self.debugger_ipc {
                    ipc.process_commands(&mut self.emulator);
                }
                
                // Step emulator if running
                if self.emulator.is_running() {
                    // Run multiple steps per frame for performance
                    for _ in 0..1000 {
                        if let Err(e) = self.emulator.step() {
                            tracing::error!("Emulator error: {}", e);
                            event_loop.exit();
                            break;
                        }
                    }
                }

                // Get framebuffer data and update display texture
                let framebuffer = self.emulator.framebuffer();
                let rgba_data = framebuffer.read().to_rgba();
                display.update_framebuffer(&rgba_data);

                // Render display
                match display.render() {
                    Ok(_) => {}
                    Err(e) if e.contains("lost") || e.contains("outdated") => {
                        let (width, height) = display.dimensions();
                        display.resize(width, height);
                    }
                    Err(e) => {
                        tracing::warn!("Render error: {}", e);
                    }
                }

                // Request next frame
                display.request_redraw();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(display) = &self.display {
            display.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let args = Args::parse();

    tracing::info!("==================================================");
    tracing::info!("NewtonEmu - PowerPC Macintosh Emulator");
    tracing::info!("Copyright (C) 2026 NewtonEmu Contributors");
    tracing::info!("Licensed under GPL v3");
    tracing::info!("==================================================");
    tracing::info!("");

    // Load configuration
    let mut file_config = if let Some(config_path) = &args.config {
        tracing::info!("Loading configuration from: {}", config_path.display());
        match FileConfig::load_from_file(config_path) {
            Ok(cfg) => {
                tracing::info!("Configuration loaded successfully!");
                cfg
            }
            Err(e) => {
                tracing::error!("Failed to load config file: {}", e);
                tracing::info!("Using default configuration");
                FileConfig::default()
            }
        }
    } else {
        // Try to load from default location
        match FileConfig::load_or_default() {
            Ok(cfg) => cfg,
            Err(e) => {
                tracing::warn!("Could not load default config: {}", e);
                tracing::info!("Using built-in defaults");
                FileConfig::default()
            }
        }
    };

    // Apply CLI overrides to file config
    file_config.apply_cli_overrides(args.cd.clone(), args.disk.clone(), args.rom.clone(), args.headless);

    // Convert FileConfig to EmulatorConfig
    let mut config = EmulatorConfig {
        cpu: CpuConfig {
            model: match file_config.cpu.model.as_str() {
                "G3" | "G3_750" => CpuModel::G3,
                "G4" | "G4_7400" | "G4_7450" => CpuModel::G4,
                "G5" | "G5_970" => CpuModel::G5,
                _ => {
                    tracing::warn!("Unknown CPU model '{}', defaulting to G4", file_config.cpu.model);
                    CpuModel::G4
                }
            },
            clock_speed: file_config.cpu.clock_speed,
            enable_jit: false, // Start with interpreter for now
        },
        memory: MemoryConfig {
            ram_size_mb: file_config.memory.ram_size_mb,
            rom_path: file_config.memory.rom_path,
        },
        display: DisplayConfig {
            width: file_config.display.width,
            height: file_config.display.height,
            color_depth: file_config.display.color_depth as u8,
        },
    };

    // Apply direct command-line overrides (these take priority over config file)
    if args.ram != 256 {
        config.memory.ram_size_mb = args.ram;
    }
    if args.width != 800 {
        config.display.width = args.width;
    }
    if args.height != 600 {
        config.display.height = args.height;
    }

    // Debug server
    if let Some(gdb_addr) = &args.gdb_server {
        tracing::info!("GDB server: {}", gdb_addr);
        // TODO: Start GDB server
    }

    tracing::info!("");
    tracing::info!("Configuration:");
    tracing::info!("  CPU Model: {:?}", config.cpu.model);
    tracing::info!("  Clock Speed: {} MHz", config.cpu.clock_speed);
    tracing::info!("  RAM: {} MB", config.memory.ram_size_mb);
    tracing::info!(
        "  Display: {}x{} @ {} bpp",
        config.display.width, config.display.height, config.display.color_depth
    );
    if let Some(ref rom) = config.memory.rom_path {
        tracing::info!("  ROM: {}", rom.display());
    }
    if let Some(ref cd) = file_config.storage.boot_cd {
        tracing::info!("  Boot CD: {}", cd.display());
    }
    if let Some(ref disk) = file_config.storage.boot_disk {
        tracing::info!("  Boot Disk: {}", disk.display());
    }
    tracing::info!("  Headless: {}", args.headless);
    tracing::info!("  Start Paused: {}", args.paused);
    
    // Log storage devices
    if !file_config.storage.scsi.is_empty() || !file_config.storage.ide.is_empty() {
        tracing::info!("");
        tracing::info!("Storage Devices:");
        for scsi in &file_config.storage.scsi {
            tracing::info!("  SCSI ID {}: {} {}", 
                scsi.id, 
                scsi.path.display(),
                if scsi.readonly { "(read-only)" } else { "" }
            );
        }
        for ide in &file_config.storage.ide {
            tracing::info!("  IDE {}/{}: {}", 
                ide.channel, 
                ide.device,
                ide.path.display()
            );
        }
    }
    
    tracing::info!("");

    // Create emulator
    let mut emulator = Emulator::new(config.clone())?;

    // Initialize debugger if requested
    let mut debugger_ipc = if args.debugger {
        tracing::info!("Enabling debugger...");
        emulator.enable_debugger();
        
        match newton_core::DebuggerIpc::start() {
            Ok(ipc) => {
                tracing::info!("Debugger IPC started - listening on stdin");
                Some(ipc)
            }
            Err(e) => {
                tracing::error!("Failed to start debugger IPC: {}", e);
                None
            }
        }
    } else {
        None
    };

    if !args.paused {
        emulator.start();
    }

    tracing::info!("Emulator initialized successfully!");

    // Headless mode - just run the emulator
    if args.headless {
        tracing::info!("Running in headless mode...");
        tracing::info!("Press Ctrl+C to exit");
        tracing::info!("");

        loop {
            // Process debugger commands if debugger is enabled
            if let Some(ref mut ipc) = debugger_ipc {
                ipc.process_commands(&mut emulator);
            }
            
            if let Err(e) = emulator.step() {
                tracing::error!("Emulator error: {}", e);
                break;
            }
        }

        return Ok(());
    }

    // With display
    tracing::info!("Starting display...");
    tracing::info!("");

    // Create event loop
    let event_loop = EventLoop::new()?;

    let mut app = App {
        emulator,
        display: None,
        config,
        headless: args.headless,
        debugger_ipc,
    };

    // Run event loop
    event_loop.run_app(&mut app)?;

    Ok(())
}
