#![cfg(debug_assertions)]

use left_right::{Absorb, ReadGuard, ReadHandle, WriteHandle};

const COLOR_PALETTES: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000];

type Tile = [[u8; 8]; 8];

// 32x32 tiles
type Buffer = Vec<Tile>;

#[derive(Debug, Default)]
pub(crate) struct TileMapFrame {
    buffer: Buffer,
}

impl Absorb<Buffer> for TileMapFrame {
    fn absorb_first(&mut self, operation: &mut Buffer, _other: &Self) {
        self.buffer = operation.clone();
    }

    fn sync_with(&mut self, first: &Self) {
        self.buffer = first.buffer.clone();
    }
}

pub struct TileMapFrameWriter(WriteHandle<TileMapFrame, Buffer>);
impl TileMapFrameWriter {
    pub fn write(&mut self, buffer: Buffer) {
        self.0.append(buffer);
    }

    pub fn flush(&mut self) {
        self.0.publish();
    }
}

pub struct TileMapFrameReader(ReadHandle<TileMapFrame>);

impl TileMapFrameReader {
    pub fn read(&self) -> Option<ReadGuard<'_, TileMapFrame>> {
        self.0.enter()
    }
}

impl TileMapFrame {
    pub(crate) fn draw(&self, frame: &mut [u8]) {
        if self.buffer.is_empty() {
            return;
        }
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            debug_assert!(i < 65536);
            let ty = i / (32 * 8 * 8);
            let tx = i % (32 * 8) / 8;

            let nth = ty * 32 + tx;
            let tile = self.buffer.get(nth).unwrap();
            let y = i / (32 * 8) % 8;
            let x = i % 8;
            let palette = tile[y][x];
            let color = COLOR_PALETTES[palette as usize];
            let rgba = [(color >> 16) as u8, (color >> 8) as u8, color as u8, 0xFF];
            pixel.copy_from_slice(&rgba);
        }
    }
}

pub fn new() -> (TileMapFrameWriter, TileMapFrameReader) {
    let (write_handle, read_handle) = left_right::new();
    let writer = TileMapFrameWriter(write_handle);
    let reader = TileMapFrameReader(read_handle);
    (writer, reader)
}

use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;

pub fn new_window(name: &str, event_loop: &EventLoop<()>) -> anyhow::Result<(Window, Pixels)> {
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title(name)
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(event_loop)
            .unwrap()
    };
    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    Ok((window, pixels))
}
