mod config;
mod frame;
mod gamepad;
mod logger;
#[cfg(debug_assertions)]
mod oam_frame;
#[cfg(debug_assertions)]
mod tile_map_frame;

use crate::config::{HEIGHT, SCALE, WIDTH};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use gb::GameBoy;
use gb_shared::command::{Command, JoypadCommand, JoypadKey};
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

    let audio_samples = Arc::new(Mutex::new(Vec::new()));
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    log::debug!("Open the audio player: {}", device.name().unwrap());
    let config = device.default_output_config().unwrap();
    let sample_format = config.sample_format();
    log::debug!("Sample format: {}", sample_format);
    let config: cpal::StreamConfig = config.into();
    log::debug!("Stream config: {:?}", config);
    let sample_rate = config.sample_rate.0 as u32;

    let stream = {
        let audio_samples = audio_samples.clone();
        let stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        let mut audio_samples = audio_samples.lock().unwrap();
                        let len = std::cmp::min(data.len() / 2, audio_samples.len());
                        for (i, (data_l, data_r)) in audio_samples.drain(..len).enumerate() {
                            data[i * 2] = data_l;
                            data[i * 2 + 1] = data_r;
                        }
                    },
                    move |err| log::error!("{}", err),
                    None,
                )
                .unwrap(),
            cpal::SampleFormat::F64 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [f64], _: &cpal::OutputCallbackInfo| {
                        let mut audio_samples = audio_samples.lock().unwrap();
                        let len = std::cmp::min(data.len() / 2, audio_samples.len());
                        for (i, (data_l, data_r)) in audio_samples.drain(..len).enumerate() {
                            data[i * 2] = data_l.to_sample::<f64>();
                            data[i * 2 + 1] = data_r.to_sample::<f64>();
                        }
                    },
                    move |err| log::error!("{}", err),
                    None,
                )
                .unwrap(),
            _ => panic!("unreachable"),
        };

        stream
    };
    stream.play().unwrap();

    let gameboy_handle = thread::spawn({
        let window = main_window.clone();
        let frame = main_frame.clone();

        move || -> anyhow::Result<()> {
            let gb = GameBoy::try_from_path(rom_path, Some(sample_rate))?;
            gb.play(
                {
                    Box::new(move |buffer, #[cfg(debug_assertions)] dbg_buffers| {
                        frame.lock().unwrap().update(buffer);
                        window.request_redraw();
                        #[cfg(debug_assertions)]
                        {
                            for buf in dbg_buffers {
                                if buf.0 == 1 {
                                    if let Some((frame, window)) = map1_dbg_handle1.as_mut() {
                                        frame.lock().unwrap().update(&buf.1);
                                        window.request_redraw();
                                    }
                                } else if buf.0 == 2 {
                                    if let Some((frame, window)) = map2_dbg_handle1.as_mut() {
                                        frame.lock().unwrap().update(&buf.1);
                                        window.request_redraw();
                                    }
                                } else if buf.0 == 0 {
                                    if let Some((frame, window)) = oam_dbg_handle1.as_mut() {
                                        frame.lock().unwrap().update(&buf.1);
                                        window.request_redraw();
                                    }
                                }
                            }
                        }
                    })
                },
                Box::new(move |sample_data| {
                    // TODO: measure consume speed vs. produce speed
                    audio_samples.lock().unwrap().extend_from_slice(sample_data);
                }),
                command_receiver,
            )?;
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
