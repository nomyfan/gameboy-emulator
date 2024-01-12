mod config;
mod frame;
mod logger;
#[cfg(debug_assertions)]
mod oam_frame;
#[cfg(debug_assertions)]
mod tile_map_frame;

use crate::config::{HEIGHT, SCALE, WIDTH};
use gb::GameBoy;
use gb_shared::command::{Command, JoypadCommand, JoypadKey};
use gb_shared::event::Event as GameBoyEvent;
#[cfg(debug_assertions)]
use gb_shared::Run;
use log::error;
use pixels::{Pixels, SurfaceTexture};
use std::sync::mpsc;
use std::thread;
use winit::dpi::LogicalSize;
#[cfg(debug_assertions)]
use winit::dpi::{LogicalPosition, Position};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::KeyCode;

use winit::window::{Window, WindowBuilder};
use winit_input_helper::WinitInputHelper;

const KEY_CODE_JOYPAD_KEY_PAIRS: [(KeyCode, JoypadKey); 8] = [
    (KeyCode::ArrowUp, JoypadKey::Up),
    (KeyCode::ArrowDown, JoypadKey::Down),
    (KeyCode::ArrowLeft, JoypadKey::Left),
    (KeyCode::ArrowRight, JoypadKey::Right),
    (KeyCode::KeyA, JoypadKey::A),
    (KeyCode::KeyS, JoypadKey::B),
    (KeyCode::KeyZ, JoypadKey::Start),
    (KeyCode::KeyX, JoypadKey::Select),
];

fn main_window(event_loop: &EventLoop<()>) -> anyhow::Result<(Window, Pixels)> {
    let window = {
        let size = LogicalSize::new(WIDTH as f64 * SCALE, HEIGHT as f64 * SCALE);
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

fn main() -> anyhow::Result<()> {
    logger::init();

    let rom_path = std::path::PathBuf::from(std::env::args().nth(1).unwrap());

    let (event_sender, event_receiver) = mpsc::channel();
    let (command_sender, command_receiver) = mpsc::channel();
    let (mut writer, reader) = frame::new();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut input = WinitInputHelper::new();

    #[cfg(debug_assertions)]
    let dbg_windows_flag = std::env::var("GB_DBG_WIN").unwrap_or_default();
    #[cfg(debug_assertions)]
    let dbg_windows_flag = dbg_windows_flag.split(',').collect::<Vec<_>>();

    #[cfg(debug_assertions)]
    let (mut map1_dbg_writer, mut map1_rpw, map1_dbg_window) = {
        if !dbg_windows_flag.contains(&"map") {
            (None, None, None)
        } else {
            let (writer, reader) = tile_map_frame::new();
            let (window, pixels) = tile_map_frame::new_window(
                "Map 0x9800",
                &event_loop,
                Position::Logical(LogicalPosition::new(50.0, 100.0)),
            )?;
            let window_id = window.id();

            (Some(writer), Some((reader, pixels, window_id)), Some(window))
        }
    };

    #[cfg(debug_assertions)]
    let (mut map2_dbg_writer, mut map2_rpw, map2_dbg_window) = {
        if !dbg_windows_flag.contains(&"map") {
            (None, None, None)
        } else {
            let (writer, reader) = tile_map_frame::new();
            let (window, pixels) = tile_map_frame::new_window(
                "Map 0x9C00",
                &event_loop,
                Position::Logical(LogicalPosition::new(50.0, 525.0)),
            )?;
            let window_id = window.id();

            (Some(writer), Some((reader, pixels, window_id)), Some(window))
        }
    };

    #[cfg(debug_assertions)]
    let (mut oam_dbg_writer, mut oam_rpw, oam_dbg_window) = {
        if !dbg_windows_flag.contains(&"oam") {
            (None, None, None)
        } else {
            let (writer, reader) = oam_frame::new();
            let (window, pixels) = oam_frame::new_window(
                &event_loop,
                Position::Logical(LogicalPosition::new(450.0, 100.0)),
            )?;
            let window_id = window.id();

            (Some(writer), Some((reader, pixels, window_id)), Some(window))
        }
    };

    let (main_window, mut pixels) = main_window(&event_loop)?;
    let main_window_id = main_window.id();

    let gameboy_handle = thread::spawn(move || -> anyhow::Result<()> {
        let gb = GameBoy::try_from_path(rom_path)?;
        gb.play(event_sender, command_receiver)?;
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
                        oam_dbg_writer.run_mut(|writer| {
                            writer.write(buffer);
                            writer.flush();
                        });
                    } else if id == 1 {
                        map1_dbg_writer.run_mut(|writer| {
                            writer.write(buffer);
                            writer.flush();
                        });
                    } else if id == 2 {
                        map2_dbg_writer.run_mut(|writer| {
                            writer.write(buffer);
                            writer.flush();
                        });
                    }
                }
            },
            Err(err) => {
                error!("{}", err);
                break;
            }
        }
    });

    event_loop.run(move |event, _target| {
        // Draw the current frame
        if let Event::WindowEvent { event, window_id } = &event {
            if let WindowEvent::RedrawRequested = event {
                if window_id == &main_window_id {
                    if let Some(guard) = reader.read() {
                        let frame = guard.as_ref();
                        frame.draw(pixels.frame_mut());
                        pixels.render().unwrap();
                    }
                } else {
                    #[cfg(debug_assertions)]
                    {
                        if let Some((dbg_reader, dbg_pixels, dbg_window_id)) = oam_rpw.as_mut() {
                            if window_id == dbg_window_id {
                                if let Some(guard) = dbg_reader.read() {
                                    let frame = guard.as_ref();
                                    frame.draw(dbg_pixels.frame_mut());
                                    dbg_pixels.render().unwrap();
                                }
                            }
                        }

                        if let Some((dbg_reader, dbg_pixels, dbg_window_id)) = map1_rpw.as_mut() {
                            if window_id == dbg_window_id {
                                if let Some(guard) = dbg_reader.read() {
                                    let frame = guard.as_ref();
                                    frame.draw(dbg_pixels.frame_mut());
                                    dbg_pixels.render().unwrap();
                                }
                            }
                        }

                        if let Some((dbg_reader, dbg_pixels, dbg_window_id)) = map2_rpw.as_mut() {
                            if window_id == dbg_window_id {
                                if let Some(guard) = dbg_reader.read() {
                                    let frame = guard.as_ref();
                                    frame.draw(dbg_pixels.frame_mut());
                                    dbg_pixels.render().unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(debug_assertions)]
        {
            oam_dbg_window.run(|window| window.request_redraw());
            map1_dbg_window.run(|window| window.request_redraw());
            map2_dbg_window.run(|window| window.request_redraw());
        }
        main_window.request_redraw();

        // Handle input events
        if input.update(&event) {
            for (keycode, joypad_key) in KEY_CODE_JOYPAD_KEY_PAIRS {
                if input.key_pressed(keycode) {
                    command_sender
                        .send(Command::Joypad(JoypadCommand::PressKey(joypad_key)))
                        .unwrap();
                }
                if input.key_released(keycode) {
                    command_sender
                        .send(Command::Joypad(JoypadCommand::ReleaseKey(joypad_key)))
                        .unwrap();
                }
            }
        }
    })?;

    let _ = gameboy_handle.join().unwrap();
    gameboy_event_handle.join().unwrap();

    Ok(())
}
