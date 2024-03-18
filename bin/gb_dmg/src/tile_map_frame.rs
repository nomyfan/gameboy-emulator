#![cfg(debug_assertions)]

use super::oam_frame::get_color_id;
use crate::config::SCALE;
use gb_shared::boxed::BoxedArray;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::{LogicalSize, Position};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

const COLOR_PALETTES: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000];

const BUFFER_SIZE: usize = 32 * 32 * 16; // 32x32 tiles
type Buffer = BoxedArray<u8, BUFFER_SIZE>;

#[derive(Debug, Default)]
pub(crate) struct TileMapFrame {
    buffer: Buffer,
}

impl TileMapFrame {
    pub(crate) fn draw(&self, frame: &mut [u8]) {
        if self.buffer.is_empty() {
            return;
        }
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            debug_assert!(i < 65536);
            let tile_y = i / (32 * 8 * 8);
            let tile_x = i % (32 * 8) / 8;

            let nth = tile_y * 32 + tile_x;
            let offset = nth * 16;
            let tile = self.buffer[offset..(offset + 16)].try_into().unwrap();
            let y = i / (32 * 8) % 8;
            let x = i % 8;
            let color_id = get_color_id(&tile, x as u8, y as u8);
            let color = COLOR_PALETTES[color_id as usize];
            let rgba = [(color >> 16) as u8, (color >> 8) as u8, color as u8, 0xFF];
            pixel.copy_from_slice(&rgba);
        }
    }

    pub(crate) fn update(&mut self, vram: &[u8], base_addr: usize, lcdc4: bool) {
        for (i, tile_index) in vram[base_addr..(base_addr + 0x400)].iter().enumerate() {
            let tile_index =
                if lcdc4 { *tile_index } else { ((*tile_index) as i8 as i16 + 256) as u8 } as usize;
            let offset = tile_index * 16;
            let tile = &vram[offset..(offset + 16)];
            self.buffer[i * 16..(i * 16 + 16)].copy_from_slice(tile);
        }
    }
}

pub fn new_window(
    name: &str,
    event_loop: &EventLoop<()>,
    position: Position,
) -> anyhow::Result<(Window, Pixels)> {
    let window = {
        let size = LogicalSize::new(256.0 * SCALE, 256.0 * SCALE);
        WindowBuilder::new()
            .with_title(name)
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_position(position)
            .build(event_loop)
            .unwrap()
    };
    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(256, 256, surface_texture)?
    };

    Ok((window, pixels))
}
