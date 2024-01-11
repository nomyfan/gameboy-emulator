#![cfg(debug_assertions)]

use left_right::{Absorb, ReadGuard, ReadHandle, WriteHandle};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::{LogicalSize, Position},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

const COLOR_PALETTES: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000];

type Buffer = Vec<[[u8; 8]; 8]>;

#[derive(Debug, Default)]
pub(crate) struct OamFrame {
    buffer: Buffer,
}

impl Absorb<Buffer> for OamFrame {
    fn absorb_first(&mut self, operation: &mut Buffer, _other: &Self) {
        self.buffer = operation.clone();
    }

    fn sync_with(&mut self, first: &Self) {
        self.buffer = first.buffer.clone();
    }
}

pub struct OamFrameWriter(WriteHandle<OamFrame, Buffer>);
impl OamFrameWriter {
    pub fn write(&mut self, buffer: Buffer) {
        self.0.append(buffer);
    }

    pub fn flush(&mut self) {
        self.0.publish();
    }
}

pub struct OamFrameReader(ReadHandle<OamFrame>);

impl OamFrameReader {
    pub fn read(&self) -> Option<ReadGuard<'_, OamFrame>> {
        self.0.enter()
    }
}

impl OamFrame {
    pub(crate) fn draw(&self, frame: &mut [u8]) {
        if self.buffer.is_empty() {
            return;
        }
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let ty = i / (16 * 8 * 8);
            let tx = i % (16 * 8) / 8;

            let nth = ty * 16 + tx;
            let tile = self.buffer.get(nth).unwrap();
            let y = i / (16 * 8) % 8;
            let x = i % 8;
            let palette = tile[y][x];
            let color = COLOR_PALETTES[palette as usize];
            let rgba = [(color >> 16) as u8, (color >> 8) as u8, color as u8, 0xFF];
            pixel.copy_from_slice(&rgba);
        }
    }
}

pub fn new() -> (OamFrameWriter, OamFrameReader) {
    let (write_handle, read_handle) = left_right::new();
    let writer = OamFrameWriter(write_handle);
    let reader = OamFrameReader(read_handle);
    (writer, reader)
}

pub fn new_window(
    event_loop: &EventLoop<()>,
    position: Position,
) -> anyhow::Result<(Window, Pixels)> {
    let window = {
        let size = LogicalSize::new(128., 192.);
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
