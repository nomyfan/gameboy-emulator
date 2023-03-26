use shared::boxed_array;

pub struct PPU {
    /// - Block 0: \[0x8000, 0x87FF]
    /// - Block 1: \[0x8800, 0x8FFF]
    /// - Block 2: \[0x9000, 0x97FF]
    ///
    /// There're two addressing modes. Mode A indexes OBJ
    /// 0-127 in block 0 and indexes OBJ 128-255 in block 1.
    /// Mode B indexes OBJ 128-255 in block 1 and indexes
    /// OBJ 0-127 in block 2.
    ///
    /// For BG and Window, if LCDC.4 is 1, then mode
    /// A is used, and if LCDC.4 is 0 then mode B is used.
    /// For sprites, the mode is always A.
    vram: Box<[u8; 0x2000]>,
    /// - Tile map 0: \[0x9800, 0x9BFF\]
    /// - Tile map 1: \[0x9C00, 0x9FFF\]
    tile_map: Box<[u8; 0x200]>,
    /// \[0xFE00, 0xFE9F\]
    /// There're up to 40 sprites. Each entry consists of
    /// 4 bytes.
    /// - Byte 0: Y position.
    /// - Byte 1: X position.
    /// - Byte 2: tile index.
    /// - Byte 3: attributes.
    ///
    /// Sprite attributes
    /// - Bit 0-2: palette number. CGB only.
    /// - Bit 3: tile VRAM bank. CGB only.
    /// - Bit 4: palette number. Non CGB only.
    /// - Bit 5: X flip(0=normal, 1=horizontally mirrored).
    /// - Bit 6: Y flip(0=normal, 1=vertically mirrored).
    /// - Bit 7: BG and Window over OBJ(0=No, 1=BG and Window colors 1-3 over the OBJ)
    oam: Box<[u8; 0xA0]>,
    /// LCD control
    /// - Bit 0: BG and Window enable/priority, 0=off, 1=on.
    /// - Bit 1: OBJ enable, 0=off, 1=on.
    /// - Bit 2: OBJ size, 0=8x8, 1=8x16.
    /// - Bit 3: BG tile map area, 0=0x9800-0x9BFF, 1=0x9C00-0x9FFF.
    /// - Bit 4: BG and Window tile data area(VRAM), 0=0x8800-0x97FF, 1=0x8000-0x8FFF.
    /// - Bit 5: Window enable, 0=off, 1=on
    /// - Bit 6: Window tile map area, 0=0x9800-0x9BFF, 1=0x9C00-0x9FFF.
    /// - Bit 7: LCD and PPU enable 0=off, 1=on.
    lcdc: u8,
}

impl PPU {
    pub fn new() -> Self {
        PPU { vram: boxed_array(0), tile_map: boxed_array(0), oam: boxed_array(0), lcdc: 0 }
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
