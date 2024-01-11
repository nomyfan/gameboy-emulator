mod config;
mod frame;
mod logger;
#[cfg(debug_assertions)]
mod oam_frame;
#[cfg(debug_assertions)]
mod tile_map_frame;

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
    let (
        (mut map1_dbg_writer, map1_dbg_reader),
        (map1_dbg_window, mut map1_dbg_pixels),
        map1_dbg_window_id,
    ) = {
        let dbg_frame = tile_map_frame::new();
        let dbg_window = tile_map_frame::new_tile_map_window("Map 0x9800", &event_loop)?;
        let dbg_window_id = dbg_window.0.id();

        (dbg_frame, dbg_window, dbg_window_id)
    };

    #[cfg(debug_assertions)]
    let (
        (mut map2_dbg_writer, map2_dbg_reader),
        (map2_dbg_window, mut map2_dbg_pixels),
        map2_dbg_window_id,
    ) = {
        let dbg_frame = tile_map_frame::new();
        let dbg_window = tile_map_frame::new_tile_map_window("Map 0x9C00", &event_loop)?;
        let dbg_window_id = dbg_window.0.id();

        (dbg_frame, dbg_window, dbg_window_id)
    };

    #[cfg(debug_assertions)]
    let (
        (mut oam_dbg_writer, oam_dbg_reader),
        (oam_dbg_window, mut oam_dbg_pixels),
        oam_dbg_window_id,
    ) = {
        let dbg_frame = oam_frame::new();
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
                GameBoyEvent::OnDebugFrame(id, buffer) => {
                    if id == 0 {
                        oam_dbg_writer.write(buffer);
                        oam_dbg_writer.flush();
                    } else if id == 1 {
                        map1_dbg_writer.write(buffer);
                        map1_dbg_writer.flush();
                    } else if id == 2 {
                        map2_dbg_writer.write(buffer);
                        map2_dbg_writer.flush();
                    }
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
                    {
                        if window_id == &oam_dbg_window_id {
                            if let Some(guard) = oam_dbg_reader.read() {
                                let frame = guard.as_ref();
                                frame.draw(oam_dbg_pixels.frame_mut());
                                oam_dbg_pixels.render();
                            }
                        }

                        if window_id == &map1_dbg_window_id {
                            if let Some(guard) = map1_dbg_reader.read() {
                                let frame = guard.as_ref();
                                frame.draw(map1_dbg_pixels.frame_mut());
                                map1_dbg_pixels.render();
                            }
                        }

                        if window_id == &map2_dbg_window_id {
                            if let Some(guard) = map2_dbg_reader.read() {
                                let frame = guard.as_ref();
                                frame.draw(map2_dbg_pixels.frame_mut());
                                map2_dbg_pixels.render();
                            }
                        }
                    }
                }
            }
            return;
        }

        #[cfg(debug_assertions)]
        {
            oam_dbg_window.request_redraw();
            map1_dbg_window.request_redraw();
            map2_dbg_window.request_redraw();
        }
        main_window.request_redraw();

        // Handle input events
        // if input.update(&event) {
        //     // Close events
        //     if input.key_pressed(KeyCode::Escape) || input.close_requested() {
        //         // TODO: stop GameBoy instance
        //         target.exit();
        //         return;
        //     }

        //     // Resize the window
        //     if let Some(size) = input.window_resized() {
        //         if let Err(_err) = pixels.resize_surface(size.width, size.height) {
        //             target.exit();
        //             return;
        //         }
        //     }

        //     #[cfg(debug_assertions)]
        //     debug_window.request_redraw();
        //     main_window.request_redraw();
        // }
    })?;

    let _ = gameboy_handle.join().unwrap();
    gameboy_event_handle.join().unwrap();

    Ok(())
}
