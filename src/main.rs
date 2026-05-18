// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

use anyhow::Result;
use clap::Parser;
use newton_core::{Emulator, EmulatorConfig};
use newton_ui::DisplayWindow;
use std::path::PathBuf;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

/// NewtonEmu - PowerPC Macintosh Emulator
#[derive(Parser, Debug)]
#[command(name = "newton-emu")]
#[command(version = "0.1.0")]
#[command(about = "PowerPC Macintosh Emulator", long_about = None)]
struct Args {
    /// ROM file path
    #[arg(value_name = "ROM")]
    rom: Option<PathBuf>,

    /// Show debug window on startup
    #[arg(short, long)]
    debug: bool,

    /// Start paused
    #[arg(short, long)]
    paused: bool,

    /// RAM size in MB
    #[arg(long, default_value = "256")]
    ram: usize,

    /// Display width
    #[arg(long, default_value = "800")]
    width: u32,

    /// Display height
    #[arg(long, default_value = "600")]
    height: u32,
}

struct App {
    emulator: Emulator,
    display: Option<DisplayWindow>,
    config: EmulatorConfig,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.display.is_none() {
            // Create window
            let window_attributes = Window::default_attributes()
                .with_title("NewtonEmu - PowerPC Macintosh Emulator")
                .with_inner_size(winit::dpi::PhysicalSize::new(
                    self.config.display.width,
                    self.config.display.height,
                ));
            
            let window = event_loop.create_window(window_attributes).unwrap();
            
            // Create display (async initialization)
            let display = pollster::block_on(DisplayWindow::new(
                window,
                self.config.display.width,
                self.config.display.height,
            ));
            
            self.display = Some(display);
            
            tracing::info!("Display window created!");
            tracing::info!("");
            tracing::info!("Controls:");
            tracing::info!("  F12: Toggle debug window (not yet implemented)");
            tracing::info!("  ESC: Quit");
            tracing::info!("");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
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
                let rgba_data = framebuffer.to_rgba();
                display.update_framebuffer(&rgba_data);

                // Render display
                match display.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        let (width, height) = display.dimensions();
                        display.resize(width, height);
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        tracing::error!("Out of memory!");
                        event_loop.exit();
                    }
                    Err(e) => {
                        tracing::error!("Render error: {:?}", e);
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

    tracing::info!("NewtonEmu - PowerPC Macintosh Emulator");
    tracing::info!("Copyright (C) 2026 NewtonEmu Contributors");
    tracing::info!("Licensed under GPL v3");
    tracing::info!("");

    // Create emulator configuration
    let mut config = EmulatorConfig::default();
    config.memory.ram_size_mb = args.ram;
    config.display.width = args.width;
    config.display.height = args.height;

    // Load ROM if provided
    if let Some(rom_path) = &args.rom {
        tracing::info!("Loading ROM: {}", rom_path.display());
        config.memory.rom_path = Some(rom_path.to_string_lossy().to_string());
    }

    tracing::info!("Configuration:");
    tracing::info!("  CPU Model: {:?}", config.cpu.model);
    tracing::info!("  Clock Speed: {} MHz", config.cpu.clock_speed);
    tracing::info!("  RAM: {} MB", config.memory.ram_size_mb);
    tracing::info!(
        "  Display: {}x{} @ {} bpp",
        config.display.width, config.display.height, config.display.color_depth
    );
    tracing::info!("  Debug Window: {}", args.debug);
    tracing::info!("  Start Paused: {}", args.paused);
    tracing::info!("");

    // Create emulator
    let mut emulator = Emulator::new(config.clone())?;
    
    if !args.paused {
        emulator.start();
    }

    tracing::info!("Emulator initialized successfully!");
    tracing::info!("Starting display...");

    // Create event loop
    let event_loop = EventLoop::new()?;
    
    let mut app = App {
        emulator,
        display: None,
        config,
    };

    // Run event loop
    event_loop.run_app(&mut app)?;

    Ok(())
}
