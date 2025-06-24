use std::fs;
use std::path::PathBuf;

use crate::gui::Framework;
use chip8_driver::{Driver, DriverError};
use error_iter::ErrorIter as _;
use log::{error, info};
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowBuilder};

mod gui;

pub enum UserCommand {
    LoadRom(PathBuf),
}

struct AppState {
    driver: Driver,
    rom_loaded: bool,
}

impl AppState {
    fn new() -> Result<Self, DriverError> {
        let driver = Driver::new()?;
        Ok(Self {
            driver,
            rom_loaded: false,
        })
    }

    fn load_rom(&mut self, rom: &[u8]) -> Result<(), DriverError> {
        self.driver.load_rom(rom)?;
        self.rom_loaded = true;
        Ok(())
    }

    fn tick(&mut self) -> Result<(), DriverError> {
        self.driver.tick()
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut app = AppState::new().expect("Failed to create driver");
    let width = chip8_driver::pixels_width() as u32;
    let height = chip8_driver::pixels_height() as u32;

    let event_loop = EventLoop::new().unwrap();
    let window = {
        // Create a window with a reasonable initial size.
        let size = LogicalSize::new(width as f64 * 10.0, height as f64 * 10.0);
        WindowBuilder::new()
            .with_title("Chip8 Desktop")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let (mut pixels, mut framework) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        // Create a pixels buffer matching the CHIP-8 resolution.
        let pixels = Pixels::new(width as u32, height as u32, surface_texture)?;
        let framework = Framework::new(
            &event_loop,
            window_size.width,
            window_size.height,
            scale_factor,
            &pixels,
        );

        (pixels, framework)
    };

    let res = event_loop.run(|event, elwt| {
        // Handle user commands
        for command in framework.drain_commands() {
            match command {
                UserCommand::LoadRom(path) => {
                    if app.rom_loaded {
                        if let Err(e) = app.driver.reset() {
                            framework.show_error(
                                "Reset Failed",
                                format!("Could not reset driver: {}", e),
                            );
                        }
                        app.rom_loaded = false;
                    }
                    info!("begin to load rom: {:?}", path);
                    match fs::read(&path) {
                        Ok(rom) => {
                            if let Err(e) = app.load_rom(&rom) {
                                framework.show_error(
                                    "ROM Load Failed",
                                    format!("Could not load ROM from {:?}: {}", path, e),
                                );
                            }
                        }
                        Err(e) => {
                            framework.show_error(
                                "ROM Read Failed",
                                format!("Could not read ROM from {:?}: {}", path, e),
                            );
                        }
                    }
                }
            }
        }

        // Update internal state and request a redraw
        if app.rom_loaded {
            if let Err(err) = app.tick() {
                log_error("driver.tick", err);
                elwt.exit();
            }

            if app.driver.is_display_updated() {
                window.request_redraw();
            }
        }

        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            }
            | Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                if let Err(err) =
                    handle_redraw_requested(&app, &mut pixels, &mut framework, &window)
                {
                    log_error("handle_redraw_requested", err);
                    elwt.exit();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { scale_factor, .. },
                ..
            } => {
                framework.scale_factor(scale_factor);
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    elwt.exit();
                    return;
                }
                framework.resize(size.width, size.height);
            }
            Event::WindowEvent { event, .. } => {
                // Update egui inputs
                framework.handle_event(&window, &event);
                window.request_redraw();
            }
            _ => (),
        }
    });
    res.map_err(|e| Error::UserDefined(Box::new(e)))
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

fn handle_redraw_requested(
    app: &AppState,
    pixels: &mut Pixels,
    framework: &mut Framework,
    window: &Window,
) -> Result<(), Error> {
    // Draw the world
    draw(&app.driver, pixels.frame_mut());

    // Prepare egui
    framework.prepare(window);

    // Render everything together
    let render_result = pixels.render_with(|encoder, render_target, context| {
        // Render the world texture
        context.scaling_renderer.render(encoder, render_target);

        // Render egui
        framework.render(encoder, render_target, context);

        Ok(())
    });

    render_result?;

    Ok(())
}

fn draw(driver: &Driver, frame: &mut [u8]) {
    let chip8_framebuffer = driver.framebuffer();

    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let chip8_pixel_state = chip8_framebuffer[i];
        let rgba = if chip8_pixel_state == 1 {
            [0xFF, 0xFF, 0xFF, 0xFF]
        } else {
            [0x00, 0x00, 0x00, 0xFF]
        };
        pixel.copy_from_slice(&rgba);
    }
}
