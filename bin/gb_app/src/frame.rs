use crate::config::{HEIGHT, WIDTH};
use gb_shared::boxed::BoxedMatrix;
use left_right::{Absorb, ReadGuard, ReadHandle, WriteHandle};

const COLOR_PALETTES: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000];

#[derive(Debug, Default)]
pub(crate) struct Frame {
    buffer: BoxedMatrix<u8, WIDTH, HEIGHT>,
}

impl Absorb<BoxedMatrix<u8, WIDTH, HEIGHT>> for Frame {
    fn absorb_first(&mut self, operation: &mut BoxedMatrix<u8, WIDTH, HEIGHT>, _other: &Self) {
        self.buffer = operation.clone();
    }

    fn sync_with(&mut self, first: &Self) {
        self.buffer = first.buffer.clone();
    }
}

pub struct FrameWriter(WriteHandle<Frame, BoxedMatrix<u8, WIDTH, HEIGHT>>);
impl FrameWriter {
    pub fn write(&mut self, buffer: BoxedMatrix<u8, WIDTH, HEIGHT>) {
        self.0.append(buffer);
    }

    pub fn flush(&mut self) {
        self.0.publish();
    }
}

pub struct FrameReader(ReadHandle<Frame>);

impl FrameReader {
    pub fn read(&self) -> Option<ReadGuard<'_, Frame>> {
        self.0.enter()
    }
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
}

pub fn new() -> (FrameWriter, FrameReader) {
    let (write_handle, read_handle) = left_right::new();
    let writer = FrameWriter(write_handle);
    let reader = FrameReader(read_handle);
    (writer, reader)
}
