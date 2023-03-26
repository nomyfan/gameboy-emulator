use shared::boxed_array;

pub struct PPU {
    /// \[0x8000, 0x9FFF\]
    vram: Box<[u8; 0x2000]>,
    /// \[0xFE00, 0xFE9F\]
    oam: Box<[u8; 0xA0]>,
}

impl PPU {
    pub fn new() -> Self {
        PPU { vram: boxed_array(0), oam: boxed_array(0) }
    }
}

impl io::IO for PPU {
    fn write(&mut self, addr: u16, value: u8) {
        debug_assert!((addr >= 0x8000 && addr <= 0x9FFF) || (addr >= 0xFE00 && addr <= 0xFE9F));

        if addr >= 0x8000 && addr <= 0x9FFF {
            self.vram[addr as usize] = value;
        } else {
            todo!("OAM")
        }
    }

    fn read(&self, addr: u16) -> u8 {
        debug_assert!((addr >= 0x8000 && addr <= 0x9FFF) || (addr >= 0xFE00 && addr <= 0xFE9F));

        if addr >= 0x8000 && addr <= 0x9FFF {
            self.vram[addr as usize]
        } else {
            todo!("OAM")
        }
    }
}
