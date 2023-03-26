use shared::boxed_array;

pub struct PPU {
    /// \[0x8000, 0x9FFF\]
    vram: Box<[u8; 0x2000]>,
}

impl PPU {
    pub fn new() -> Self {
        PPU { vram: boxed_array(0) }
    }
}

impl io::IO for PPU {
    fn write(&mut self, addr: u16, value: u8) {
        debug_assert!(addr >= 0x8000 && addr <= 0x9FFF);
        self.vram[addr as usize] = value;
    }

    fn read(&self, addr: u16) -> u8 {
        debug_assert!(addr >= 0x8000 && addr <= 0x9FFF);
        self.vram[addr as usize]
    }
}
