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
use newton_ui::{DebugWindow, DisplayWindow};
use std::path::PathBuf;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
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

struct DebugWindowState {
    window: std::sync::Arc<Window>,
    debug_ui: DebugWindow,
    egui_state: egui_winit::State,
    egui_ctx: egui::Context,
    renderer: egui_wgpu::Renderer,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
}

struct App {
    emulator: Emulator,
    display: Option<DisplayWindow>,
    debug_window: Option<DebugWindowState>,
    config: EmulatorConfig,
    show_debug: bool,
}

impl App {
    fn create_debug_window(&mut self, event_loop: &ActiveEventLoop) {
        if self.debug_window.is_some() {
            return;
        }

        tracing::info!("Creating debug window...");

        // Create debug window
        let window_attributes = Window::default_attributes()
            .with_title("NewtonEmu - Debug")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 900))
            .with_resizable(true);

        let window = std::sync::Arc::new(event_loop.create_window(window_attributes).unwrap());
        let size = window.inner_size();

        // Set up egui
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
            None,
        );

        // Set up wgpu for egui
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: Default::default(),
            display: None,
            memory_budget_thresholds: Default::default(),
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Debug Window Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            trace: wgpu::Trace::Off,
        }))
        .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let renderer = egui_wgpu::Renderer::new(
            &device,
            surface_format,
            egui_wgpu::RendererOptions::default(),
        );

        self.debug_window = Some(DebugWindowState {
            window,
            debug_ui: DebugWindow::new(),
            egui_state,
            egui_ctx,
            renderer,
            device,
            queue,
            surface,
            surface_config,
        });

        tracing::info!("Debug window created!");
    }

    fn close_debug_window(&mut self) {
        if self.debug_window.take().is_some() {
            tracing::info!("Debug window closed");
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
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
            tracing::info!("  F12: Toggle debug window");
            tracing::info!("  ESC: Quit");
            tracing::info!("");
        }

        // Create debug window if requested
        if self.show_debug && self.debug_window.is_none() {
            self.create_debug_window(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Check if event is for debug window
        if let Some(debug) = &mut self.debug_window {
            if debug.window.id() == window_id {
                let event_response = debug.egui_state.on_window_event(&debug.window, &event);

                if event_response.consumed {
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => {
                        self.show_debug = false;
                        self.close_debug_window();
                        return;
                    }
                    WindowEvent::Resized(new_size) => {
                        if new_size.width > 0 && new_size.height > 0 {
                            debug.surface_config.width = new_size.width;
                            debug.surface_config.height = new_size.height;
                            debug
                                .surface
                                .configure(&debug.device, &debug.surface_config);
                        }
                        return;
                    }
                    WindowEvent::RedrawRequested => {
                        // Render debug UI
                        let raw_input = debug.egui_state.take_egui_input(&debug.window);
                        let full_output = debug.egui_ctx.run(raw_input, |ctx| {
                            debug.debug_ui.update(ctx, &mut self.emulator);
                        });

                        debug
                            .egui_state
                            .handle_platform_output(&debug.window, full_output.platform_output);

                        let tris = debug
                            .egui_ctx
                            .tessellate(full_output.shapes, full_output.pixels_per_point);

                        let screen_descriptor = egui_wgpu::ScreenDescriptor {
                            size_in_pixels: [debug.surface_config.width, debug.surface_config.height],
                            pixels_per_point: debug.window.scale_factor() as f32,
                        };

                        for (id, image_delta) in &full_output.textures_delta.set {
                            debug.renderer.update_texture(
                                &debug.device,
                                &debug.queue,
                                *id,
                                image_delta,
                            );
                        }

                        let mut encoder =
                            debug
                                .device
                                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                    label: Some("Debug UI Encoder"),
                                });

                        debug.renderer.update_buffers(
                            &debug.device,
                            &debug.queue,
                            &mut encoder,
                            &tris,
                            &screen_descriptor,
                        );

                        let output_frame = match debug.surface.get_current_texture() {
                            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
                            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
                            other => {
                                tracing::error!("Failed to get debug surface texture: {:?}", other);
                                return;
                            }
                        };

                        let output_view = output_frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        {
                            let render_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("Debug UI Render Pass"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &output_view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                                r: 0.1,
                                                g: 0.1,
                                                b: 0.1,
                                                a: 1.0,
                                            }),
                                            store: wgpu::StoreOp::Store,
                                        },
                                        depth_slice: None,
                                    })],
                                    depth_stencil_attachment: None,
                                    occlusion_query_set: None,
                                    timestamp_writes: None,
                                    multiview_mask: None,
                                });

                            debug.renderer.render(
                                &mut render_pass.forget_lifetime(),
                                &tris,
                                &screen_descriptor,
                            );
                        }

                        debug.queue.submit(std::iter::once(encoder.finish()));
                        output_frame.present();

                        for id in &full_output.textures_delta.free {
                            debug.renderer.free_texture(id);
                        }

                        debug.window.request_redraw();
                        return;
                    }
                    _ => return,
                }
            }
        }

        // Handle display window events
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

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::F12),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                // Toggle debug window
                self.show_debug = !self.show_debug;
                if self.show_debug {
                    self.create_debug_window(event_loop);
                } else {
                    self.close_debug_window();
                }
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
        if let Some(debug) = &self.debug_window {
            debug.window.request_redraw();
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
        debug_window: None,
        config,
        show_debug: args.debug,
    };

    // Run event loop
    event_loop.run_app(&mut app)?;

    Ok(())
}
