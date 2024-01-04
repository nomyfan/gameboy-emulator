mod config;
#[cfg(debug_assertions)]
mod debug_frame;
mod frame;
mod logger;

use crate::config::{HEIGHT, WIDTH};
use gb::GameBoy;
use gb_shared::event::Event as GameBoyEvent;
use log::error;
use pixels::{Pixels, SurfaceTexture};
use std::sync::mpsc;
use std::thread;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowBuilder};
use winit_input_helper::WinitInputHelper;

fn main_window(event_loop: &EventLoop<()>) -> anyhow::Result<(Window, Pixels)> {
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("GameBoy")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(event_loop)
            .unwrap()
    };

    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    Ok((window, pixels))
}

#[cfg(debug_assertions)]
fn debug_window(event_loop: &EventLoop<()>) -> anyhow::Result<(Window, Pixels)> {
    let window = {
        let size = LogicalSize::new(128., 192.);
        WindowBuilder::new()
            .with_title("GameBoy Debug")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(event_loop)
            .unwrap()
    };
    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(128, 192, surface_texture)?
    };

    Ok((window, pixels))
}

fn main() -> anyhow::Result<()> {
    logger::init();

    let rom_path = std::path::PathBuf::from(std::env::args().nth(1).unwrap());

    let (event_sender, event_receiver) = mpsc::channel();
    let (mut writer, reader) = frame::new();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut input = WinitInputHelper::new();

    #[cfg(debug_assertions)]
    let ((mut debug_writer, debug_reader), (debug_window, mut debug_pixels), debug_window_id) = {
        let dbg_frame = debug_frame::new();
        let dbg_window = debug_window(&event_loop)?;
        let dbg_window_id = dbg_window.0.id();

        (dbg_frame, dbg_window, dbg_window_id)
    };

    let (main_window, mut pixels) = main_window(&event_loop)?;
    let main_window_id = main_window.id();

    let gameboy_handle = thread::spawn(move || -> anyhow::Result<()> {
        let gb = GameBoy::try_from_path(rom_path)?;
        gb.play(event_sender)?;
        Ok(())
    });

    let gameboy_event_handle = thread::spawn(move || loop {
        match event_receiver.recv() {
            Ok(event) => match event {
                GameBoyEvent::OnFrame(buffer) => {
                    writer.write(buffer);
                    writer.flush();
                }
                #[cfg(debug_assertions)]
                GameBoyEvent::OnDebugFrame(buffer) => {
                    debug_writer.write(buffer);
                    debug_writer.flush();
                }
            },
            Err(err) => {
                error!("{}", err);
                break;
            }
        }
    });

    event_loop.run(move |event, target| {
        // Draw the current frame
        if let Event::WindowEvent { event, window_id } = &event {
            if let WindowEvent::RedrawRequested = event {
                if window_id == &main_window_id {
                    if let Some(guard) = reader.read() {
                        let frame = guard.as_ref();
                        frame.draw(pixels.frame_mut());
                        pixels.render();
                    }
                } else {
                    #[cfg(debug_assertions)]
                    if window_id == &debug_window_id {
                        if let Some(guard) = debug_reader.read() {
                            let frame = guard.as_ref();
                            frame.draw(debug_pixels.frame_mut());
                            debug_pixels.render();
                        }
                    }
                }
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                // TODO: stop GameBoy instance
                target.exit();
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(_err) = pixels.resize_surface(size.width, size.height) {
                    target.exit();
                    return;
                }
            }

            #[cfg(debug_assertions)]
            debug_window.request_redraw();
            main_window.request_redraw();
        }
    })?;

    let _ = gameboy_handle.join().unwrap();
    gameboy_event_handle.join().unwrap();

    Ok(())
}
