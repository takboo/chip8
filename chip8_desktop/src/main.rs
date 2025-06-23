use std::fs;

use crate::gui::Framework;
use chip8_driver::Driver;
use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

mod gui;

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut driver = Driver::new().expect("Failed to create driver");
    let rom = fs::read("/Users/jam/Downloads/Chip8 Picture.ch8").expect("Failed to read ROM");
    driver.load_rom(&rom).expect("Failed to load ROM");

    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        // Create a window with a reasonable initial size.
        let size = LogicalSize::new(
            driver.pixels_width() as f64 * 10.0,
            driver.pixels_height() as f64 * 10.0,
        );
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
        let pixels = Pixels::new(
            driver.pixels_width() as u32,
            driver.pixels_height() as u32,
            surface_texture,
        )?;
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
        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
                return;
            }

            // Update the scale factor
            if let Some(scale_factor) = input.scale_factor() {
                framework.scale_factor(scale_factor);
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    elwt.exit();
                    return;
                }
                framework.resize(size.width, size.height);
            }

            // Update internal state and request a redraw
            driver
                .tick()
                .map_err(|err| {
                    log_error("driver.tick", err);
                    elwt.exit();
                })
                .expect("invalid opcode");
            if driver.is_display_updated() {
                window.request_redraw();
            }
        }

        match event {
            // Draw the current frame
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Draw the world
                draw(&driver, pixels.frame_mut());

                // Prepare egui
                framework.prepare(&window);

                // Render everything together
                let render_result = pixels.render_with(|encoder, render_target, context| {
                    // Render the world texture
                    context.scaling_renderer.render(encoder, render_target);

                    // Render egui
                    framework.render(encoder, render_target, context);

                    Ok(())
                });

                // Basic error handling
                if let Err(err) = render_result {
                    log_error("pixels.render", err);
                    elwt.exit();
                }
            }
            Event::WindowEvent { event, .. } => {
                // Update egui inputs
                framework.handle_event(&window, &event);
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
