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

struct ScaleImageData(Vec<u8>, u8);

impl ScaleImageData {
    fn new(width: usize, height: usize, scale: u8) -> ScaleImageData {
        let scaled_size = width * scale as usize * height * scale as usize * 4;
        ScaleImageData(vec![0; scaled_size], scale)
    }

    fn set(&mut self, x: usize, y: usize, color: &[u8; 4]) {
        let scale = self.1 as usize;
        let x_begin = x * scale;
        let x_end = x_begin + scale;
        let y_begin = y * scale;
        let y_end = y_begin + scale;

        for y in y_begin..y_end {
            for x in x_begin..x_end {
                let offset = (y * 160 * scale + x) * 4;
                self.0[offset..(offset + 4)].copy_from_slice(color);
            }
        }
    }

    fn as_image_data(&self) -> ImageData {
        ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.0),
            160 * self.1 as u32,
            144 * self.1 as u32,
        )
        .unwrap()
    }
}

#[wasm_bindgen(js_class = GameBoy)]
impl GameBoyHandle {
    #[wasm_bindgen]
    pub fn __for_emitting_types_only__(_: JoypadKey) {}

    #[wasm_bindgen]
    pub fn create(
        rom: Uint8ClampedArray,
        canvas: OffscreenCanvas,
        scale: Option<u8>,
        sample_rate: Option<u32>,
        audio_stream: Option<WritableStream>,
    ) -> GameBoyHandle {
        let rom = rom.to_vec();
        let cart = Cartridge::try_from(rom).unwrap();

        let mut gb = GameBoy::new(Manifest { cart, sample_rate });

        let scale = scale.unwrap_or(1);
        let canvas_context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<OffscreenCanvasRenderingContext2d>()
            .unwrap();
        canvas_context.set_transform(scale as f64, 0.0, 0.0, scale as f64, 0.0, 0.0).unwrap();

        // TODO: how to dynamically change the scale?
        let mut scale_image = ScaleImageData::new(160, 144, scale);
        let frame_handle =
            Box::new(
                move |data: &BoxedArray<u8, 23040>,
                      #[cfg(debug_assertions)] _dbg_data: Option<(
                    &BoxedArray<u8, 0x2000>,
                    bool,
                )>| {
                    data.iter().enumerate().for_each(|(n, color_id)| {
                        let color = &COLORS[*color_id as usize];

                        let y = n / 160;
                        let x = n % 160;
                        scale_image.set(x, y, color);
                    });

                    let image_data = scale_image.as_image_data();
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
