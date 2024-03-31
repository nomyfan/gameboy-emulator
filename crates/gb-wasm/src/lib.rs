mod utils;

use gb::wasm::{Cartridge, GameBoy, Manifest};
use gb_shared::boxed::BoxedArray;
use gb_shared::command::{Command, JoypadCommand, JoypadKey};
use js_sys::Uint8ClampedArray;
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::{
    js_sys, ImageData, OffscreenCanvas, OffscreenCanvasRenderingContext2d, WritableStream,
};

const COLORS: [[u8; 4]; 4] = [
    [0xFF, 0xFF, 0xFF, 0xFF],
    [0xAA, 0xAA, 0xAA, 0xFF],
    [0x55, 0x55, 0x55, 0xFF],
    [0x00, 0x00, 0x00, 0xFF],
];

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
        audio_stream: Option<WritableStream>,
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

        // FIXME: scale
        let mut raw_pixels = vec![0; 160 * 144 * 4];

        let frame_handle =
            Box::new(
                move |data: &BoxedArray<u8, 23040>,
                      #[cfg(debug_assertions)] _dbg_data: Option<(
                    &BoxedArray<u8, 0x2000>,
                    bool,
                )>| {
                    data.iter().enumerate().for_each(|(n, color_id)| {
                        let color = &COLORS[*color_id as usize];
                        let offset = n * 4;
                        raw_pixels[offset..=(offset + 3)].copy_from_slice(color);
                    });

                    let image_data =
                        ImageData::new_with_u8_clamped_array_and_sh(Clamped(&raw_pixels), 160, 144)
                            .unwrap();
                    canvas_context.put_image_data(&image_data, 0.0, 0.0).unwrap();
                },
            );

        match audio_stream {
            Some(stream) => {
                let stream_writer = stream.get_writer().unwrap();
                let sample_rate = sample_rate.unwrap();
                let sample_count = sample_rate.div_ceil(64); // TODO: align to APU
                let audio_buffer = js_sys::Float32Array::new_with_length(sample_count * 2);

                gb.set_handles(
                    Some(frame_handle),
                    Some(Box::new(move |data| {
                        let len = data.len().min(sample_count as usize);
                        for (i, (left, right)) in data.iter().take(len).enumerate() {
                            audio_buffer.set_index(i as u32 * 2, *left);
                            audio_buffer.set_index(i as u32 * 2 + 1, *right);
                        }

                        let slice = audio_buffer.slice(0, (len * 2) as u32);

                        // TODO: should we wait?
                        let _ = stream_writer.write_with_chunk(&slice.into());
                    })),
                )
            }
            None => {
                gb.set_handles(Some(frame_handle), None);
            }
        }

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
