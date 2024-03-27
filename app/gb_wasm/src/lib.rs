mod utils;

use gb::wasm::{Cartridge, GameBoy, Manifest};
use wasm_bindgen::prelude::*;
use web_sys::js_sys::Uint8ClampedArray;
use web_sys::CanvasRenderingContext2d;

const COLOR_PALETTES: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000];

#[wasm_bindgen(js_name = GameBoy)]
pub struct GameBoyHandle {
    gb: GameBoy,
    frame_id: u32,
}

#[wasm_bindgen(js_name = newGameBoy)]
pub fn new_gameboy(rom: Uint8ClampedArray) -> GameBoyHandle {
    let rom = rom.to_vec();
    let cart = Cartridge::try_from(rom).expect("TODO:");
    // TODO: audio
    let gb = GameBoy::new(Manifest { cart, sample_rate: None });

    GameBoyHandle { gb, frame_id: 0 }
}

#[wasm_bindgen(js_class = GameBoy)]
impl GameBoyHandle {
    #[wasm_bindgen]
    pub fn play_with_clocks(&mut self) {
        let _ = self.gb.play_with_clocks();
    }

    #[wasm_bindgen]
    pub fn draw(&mut self, context: CanvasRenderingContext2d) -> bool {
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
