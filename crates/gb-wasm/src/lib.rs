mod utils;

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
                    // TODO: Improve communication performance
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

        GameBoyHandle { gb }
    }

    #[wasm_bindgen(js_name = continue)]
    pub fn r#continue(&mut self) {
        self.gb.continue_clocks(70224); // 70224 clocks per frame
    }

    #[wasm_bindgen(js_name = changeKeyState)]
    pub fn change_key_state(&mut self, state: u8) {
        self.gb.exec_command(Command::Joypad(JoypadCommand::State(state)));
    }
}
