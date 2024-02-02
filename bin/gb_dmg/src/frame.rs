use crate::config::{HEIGHT, WIDTH};
use gb_shared::boxed::BoxedMatrix;

const COLOR_PALETTES: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000];

#[derive(Debug, Default)]
pub(crate) struct Frame {
    buffer: BoxedMatrix<u8, WIDTH, HEIGHT>,
}

impl Frame {
    pub(crate) fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = i % WIDTH;
            let y = i / WIDTH;

            let palette = self.buffer[y][x];
            let color = COLOR_PALETTES[palette as usize];
            // info!("color {:#08X}", color);
            let rgba = [(color >> 16) as u8, (color >> 8) as u8, color as u8, 0xFF];
            pixel.copy_from_slice(&rgba);
        }
    }

    pub(crate) fn update(&mut self, buffer: &BoxedMatrix<u8, WIDTH, HEIGHT>) {
        if self.buffer.is_empty() {
            self.buffer = buffer.clone();
        } else {
            for (i, row) in buffer.iter().enumerate() {
                self.buffer[i].copy_from_slice(row.as_ref());
            }
        }
    }
}
