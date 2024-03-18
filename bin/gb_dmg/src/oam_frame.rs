#![cfg(debug_assertions)]

use gb_shared::boxed::BoxedArray;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::{LogicalSize, Position},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::config::SCALE;

const COLOR_PALETTES: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000];

type Buffer = BoxedArray<u8, 0x1800>;

#[derive(Debug, Default)]
pub(crate) struct OamFrame {
    buffer: Buffer,
}

pub(super) fn get_color_id(data: &[u8], x: u8, y: u8) -> u8 {
    let nth = (y << 1) as usize;
    let offset = (7 - x) as usize;

    let low = (data[nth] >> offset) & 1;
    let high = (data[nth + 1] >> offset) & 1;

    (high << 1) | low
}

pub(super) fn read_pixel(buffer: &[u8], i: usize, width: usize) -> [u8; 4] {
    let tile_y = i / (width * 8 * 8);
    let tile_x = i % (width * 8) / 8;

    let nth = tile_y * width + tile_x;
    let offset = nth * 16;
    let tile = &buffer[offset..(offset + 16)];
    let y = i / (width * 8) % 8;
    let x = i % 8;
    let color_id = get_color_id(tile, x as u8, y as u8);
    let color = COLOR_PALETTES[color_id as usize];

    [(color >> 16) as u8, (color >> 8) as u8, color as u8, 0xFF]
}

impl OamFrame {
    pub(crate) fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let rgba = read_pixel(&self.buffer[..], i, 16);
            pixel.copy_from_slice(&rgba);
        }
    }

    pub(crate) fn update(&mut self, buffer: &[u8]) {
        self.buffer.copy_from_slice(buffer);
    }
}

pub fn new_window(
    event_loop: &EventLoop<()>,
    position: Position,
) -> anyhow::Result<(Window, Pixels)> {
    let window = {
        let size = LogicalSize::new(128.0 * SCALE, 192.0 * SCALE);
        WindowBuilder::new()
            .with_title("OAM")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_position(position)
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
