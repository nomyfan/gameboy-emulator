mod audio;
mod utils;

use cpal::Stream;
use gb::wasm::{Cartridge, GameBoy, Manifest};
use gb_shared::command::{Command, JoypadCommand, JoypadKey};
use wasm_bindgen::prelude::*;
use web_sys::{
    js_sys::Uint8ClampedArray, MessagePort, OffscreenCanvas, OffscreenCanvasRenderingContext2d,
};

const COLOR_PALETTES: [&str; 4] = ["#FFFFFF", "#AAAAAA", "#555555", "#000000"];

#[wasm_bindgen(js_name = GameBoy)]
pub struct GameBoyHandle {
    gb: GameBoy,
    frame_id: u32,
    _audio_stream: Option<Stream>,
}

#[wasm_bindgen(js_class = GameBoy)]
impl GameBoyHandle {
    #[wasm_bindgen]
    pub fn __for_emitting_types_only__(_: JoypadKey) {}

    #[wasm_bindgen]
    pub fn create(
        rom: Uint8ClampedArray,
        canvas: OffscreenCanvas,
        sample_rate: Option<u32>,
        audio_port: Option<MessagePort>,
    ) -> GameBoyHandle {
        let rom = rom.to_vec();
        let cart = Cartridge::try_from(rom).unwrap();

        let mut gb = GameBoy::new(Manifest { cart, sample_rate });

        let canvas_context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<OffscreenCanvasRenderingContext2d>()
            .unwrap();

        gb.set_handles(
            Some(Box::new(move |data, #[cfg(debug_assertions)] _dbg_data| {
                data.iter().enumerate().for_each(|(y, pixel)| {
                    pixel.iter().enumerate().for_each(|(x, color_id)| {
                        let color = COLOR_PALETTES[*color_id as usize];
                        canvas_context.set_fill_style(&color.into());
                        canvas_context.fill_rect(x as f64, y as f64, 1.0, 1.0);
                    });
                });
            })),
            Some(Box::new(move |data| {
                if let Some(audio_port) = audio_port.as_ref() {
                    let f32_float_array =
                        web_sys::js_sys::Float32Array::new(&(data.len() as u32 * 2).into());
                    for (i, (left, right)) in data.iter().enumerate() {
                        f32_float_array.set_index(i as u32 * 2, *left);
                        f32_float_array.set_index(i as u32 * 2 + 1, *right);
                    }

                    audio_port
                        .post_message_with_transferable(
                            &f32_float_array,
                            &web_sys::js_sys::Array::of1(&f32_float_array.buffer()),
                        )
                        .unwrap();
                }
            })),
        );

        GameBoyHandle { gb, frame_id: 0, _audio_stream: None }
    }

    #[wasm_bindgen(js_name = fromUint8ClampedArray)]
    pub fn from_uint8_clamped_array(rom: Uint8ClampedArray) -> GameBoyHandle {
        let rom = rom.to_vec();
        let cart = Cartridge::try_from(rom).expect("TODO:");

        // TODO: what about default device changes?
        // let (stream, samples_buf, sample_rate) = init_audio()
        //     .map(|(stream, buf, sample_rate)| (Some(stream), Some(buf), Some(sample_rate)))
        //     .unwrap_or_default();

        let sample_rate = None;
        let stream = None;

        let mut gb = GameBoy::new(Manifest { cart, sample_rate });

        // if let Some(samples_buf) = samples_buf {
        //     gb.set_handles(
        //         None,
        //         Some(Box::new(move |sample_data| {
        //             samples_buf.lock().unwrap().extend_from_slice(sample_data);
        //         })),
        //     );
        // }

        // if let Some(stream) = &stream {
        //     let _ = stream.play();
        // }

        GameBoyHandle { gb, frame_id: 0, _audio_stream: stream }
    }

    #[wasm_bindgen(js_name = continue)]
    pub fn r#continue(&mut self) {
        self.gb.continue_clocks(70224); // 70224 clocks per frame
    }

    #[wasm_bindgen(js_name = changeKeyState)]
    pub fn change_key_state(&mut self, state: u8) {
        self.gb.exec_command(Command::Joypad(JoypadCommand::State(state)));
    }

    #[wasm_bindgen]
    pub fn draw(&mut self, context: OffscreenCanvasRenderingContext2d) -> bool {
        let (data, frame_id) = self.gb.pull_frame();
        if frame_id != self.frame_id {
            self.frame_id = frame_id;

            data.iter().enumerate().for_each(|(y, pixel)| {
                pixel.iter().enumerate().for_each(|(x, color_id)| {
                    let color = COLOR_PALETTES[*color_id as usize];
                    let r = (color >> 16) as u8;
                    let g = (color >> 8) as u8;
                    let b = color as u8;
                    context.set_fill_style(&format!("rgb({}, {}, {})", r, g, b).into());
                    context.fill_rect(x as f64, y as f64, 1.0, 1.0);
                });
            });
            return true;
        }

        false
    }
}
