mod audio;
mod config;
mod frame;
mod gamepad;
mod logger;
#[cfg(debug_assertions)]
mod oam_frame;
#[cfg(debug_assertions)]
mod tile_map_frame;

use crate::config::{HEIGHT, SCALE, WIDTH};
use cpal::traits::StreamTrait;
use gb::GameBoy;
#[cfg(debug_assertions)]
use gb_shared::boxed::BoxedArray;
use gb_shared::command::{Command, JoypadCommand, JoypadKey};
use gb_shared::VideoFrame;
use pixels::{Pixels, SurfaceTexture};
use std::sync::{mpsc, Arc, Mutex};
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
    (KeyCode::Enter, JoypadKey::Start),
    (KeyCode::Space, JoypadKey::Select),
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

    let (command_sender, command_receiver) = mpsc::channel();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut input = WinitInputHelper::new();

    #[cfg(debug_assertions)]
    let dbg_windows_flag = std::env::var("GB_DBG_WIN").unwrap_or_default();
    #[cfg(debug_assertions)]
    let dbg_windows_flag = dbg_windows_flag.split(',').collect::<Vec<_>>();

    #[cfg(debug_assertions)]
    let (mut map1_dbg_handle1, mut map1_dbg_handle2) = {
        if !dbg_windows_flag.contains(&"map") {
            None
        } else {
            let (window, pixels) = tile_map_frame::new_window(
                "Map 0x9800",
                &event_loop,
                Position::Logical(LogicalPosition::new(50.0, 100.0)),
            )?;
            let frame = Arc::new(Mutex::new(tile_map_frame::TileMapFrame::default()));

            Some((frame, pixels, Arc::new(window)))
        }
    }
    .map(|x| (Some((x.0.clone(), x.2.clone())), Some(x)))
    .unwrap_or_default();

    #[cfg(debug_assertions)]
    let (mut map2_dbg_handle1, mut map2_dbg_handle2) = {
        if !dbg_windows_flag.contains(&"map") {
            None
        } else {
            let (window, pixels) = tile_map_frame::new_window(
                "Map 0x9C00",
                &event_loop,
                Position::Logical(LogicalPosition::new(50.0, 525.0)),
            )?;
            let frame = Arc::new(Mutex::new(tile_map_frame::TileMapFrame::default()));

            Some((frame, pixels, Arc::new(window)))
        }
    }
    .map(|x| (Some((x.0.clone(), x.2.clone())), Some(x)))
    .unwrap_or_default();

    #[cfg(debug_assertions)]
    let (mut oam_dbg_handle1, mut oam_dbg_handle2) = {
        if !dbg_windows_flag.contains(&"oam") {
            None
        } else {
            let (window, pixels) = oam_frame::new_window(
                &event_loop,
                Position::Logical(LogicalPosition::new(450.0, 100.0)),
            )?;
            let frame = Arc::new(Mutex::new(oam_frame::OamFrame::default()));

            Some((frame, pixels, Arc::new(window)))
        }
    }
    .map(|x| (Some((x.0.clone(), x.2.clone())), Some(x)))
    .unwrap_or_default();

    let (main_window, mut pixels) = main_window(&event_loop)?;
    let main_window = Arc::new(main_window);
    let main_window_id = main_window.id();
    let main_frame = Arc::new(Mutex::new(frame::Frame::default()));

    let (stream, samples_buf, sample_rate) = audio::init_audio()
        .map(|(stream, buf, sample_rate)| (Some(stream), Some(buf), Some(sample_rate)))
        .unwrap_or_default();

    if let Some(stream) = &stream {
        stream.play()?;
    }

    let gameboy_handle = thread::spawn({
        let window = main_window.clone();
        let frame = main_frame.clone();

        let video_handle =
            Box::new(
                move |buffer: &VideoFrame,
                      #[cfg(debug_assertions)] vram_data: Option<(
                    &BoxedArray<u8, 0x2000>,
                    bool,
                )>| {
                    frame.lock().unwrap().update(buffer);
                    window.request_redraw();
                    #[cfg(debug_assertions)]
                    {
                        if let Some((vram_data, lcdc4)) = vram_data {
                            if let Some((frame, window)) = oam_dbg_handle1.as_mut() {
                                frame.lock().unwrap().update(&vram_data[..0x1800]);
                                window.request_redraw();
                            }
                            if let Some((frame, window)) = map1_dbg_handle1.as_mut() {
                                frame.lock().unwrap().update(&vram_data[..], 0x1800, lcdc4);
                                window.request_redraw();
                            }
                            if let Some((frame, window)) = map2_dbg_handle1.as_mut() {
                                frame.lock().unwrap().update(&vram_data[..], 0x1C00, lcdc4);
                                window.request_redraw();
                            }
                        }
                    }
                },
            );

        move || -> anyhow::Result<()> {
            let gb = GameBoy::try_from_path(rom_path, sample_rate)?;
            match samples_buf {
                Some(samples_buf) => {
                    gb.play(
                        video_handle,
                        Some(Box::new(move |sample_data| {
                            samples_buf.lock().unwrap().extend_from_slice(sample_data);
                        })),
                        command_receiver,
                    )?;
                }
                None => {
                    gb.play(video_handle, None, command_receiver)?;
                }
            }
            Ok(())
        }
    });

    // Optional Gamepad event loop
    std::thread::spawn({
        let command_sender = command_sender.clone();
        move || {
            gamepad::run_event_loop(|key, is_pressed| {
                let joypad_cmd = if is_pressed {
                    JoypadCommand::PressKey(key)
                } else {
                    JoypadCommand::ReleaseKey(key)
                };
                command_sender.send(Command::Joypad(joypad_cmd)).unwrap();
            })
        }
    });

    event_loop.run(move |event, elwt| {
        // Draw the current frame
        if let Event::WindowEvent { event, window_id } = &event {
            match event {
                WindowEvent::CloseRequested => {
                    // Once GB instance exits, the GB event handling event thread will exit due to closed channel,
                    // then the whole application will exit.
                    command_sender.send(Command::Exit).unwrap();
                    elwt.exit();
                }
                WindowEvent::RedrawRequested => {
                    if window_id == &main_window_id {
                        main_frame.lock().unwrap().draw(pixels.frame_mut());
                        pixels.render().unwrap();
                    } else {
                        #[cfg(debug_assertions)]
                        {
                            if let Some((frame, pixels, window)) = oam_dbg_handle2.as_mut() {
                                if window_id == &window.id() {
                                    frame.lock().unwrap().draw(pixels.frame_mut());
                                    pixels.render().unwrap();
                                }
                            }
                            if let Some((frame, pixels, window)) = map1_dbg_handle2.as_mut() {
                                if window_id == &window.id() {
                                    frame.lock().unwrap().draw(pixels.frame_mut());
                                    pixels.render().unwrap();
                                }
                            }
                            if let Some((frame, pixels, window)) = map2_dbg_handle2.as_mut() {
                                if window_id == &window.id() {
                                    frame.lock().unwrap().draw(pixels.frame_mut());
                                    pixels.render().unwrap();
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

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

    Ok(())
}
