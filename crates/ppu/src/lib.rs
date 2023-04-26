use shared::boxed_array;

#[repr(C)]
struct Sprite {
    y: u8,
    x: u8,
    tile_index: u8,
    attrs: u8,
}

enum Mode {
    /// OAM is inaccessible(except DMA) during this period.
    OamScan,
    /// VRAM is inaccessible during this period.
    Drawing,
    HBlank,
    /// Everything is accessible during this period.
    VBlank,
}

struct LCD {
    /// LCD control, at 0xFF40.
    /// - Bit 0: BG and Window enable/priority, 0=off, 1=on.
    /// - Bit 1: OBJ enable, 0=off, 1=on.
    /// - Bit 2: OBJ size, 0=8x8, 1=8x16.
    /// - Bit 3: BG tile map area, 0=0x9800-0x9BFF, 1=0x9C00-0x9FFF.
    /// - Bit 4: BG and Window tile data area(VRAM), 0=0x8800-0x97FF, 1=0x8000-0x8FFF.
    /// - Bit 5: Window enable, 0=off, 1=on
    /// - Bit 6: Window tile map area, 0=0x9800-0x9BFF, 1=0x9C00-0x9FFF.
    /// - Bit 7: LCD and PPU enable 0=off, 1=on.
    lcdc: u8,
    /// LCD status, at 0xFF41.
    stat: u8,
    /// LCD Y coordinate, at 0xFF44.
    ly: u8,
    /// LCD Y compare, at 0xFF45.
    lyc: u8,
    /// Window Y position, at 0xFF4A.
    wy: u8,
    /// Window X position plus 7, at 0xFF4B.
    wx: u8,
    /// Scroll Y, at 0xFF42.
    scy: u8,
    /// Scroll X, at 0xFF43.
    scx: u8,
}

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
    vram: Box<[u8; 0x1800]>,
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
    lcd: LCD,
    mode: Mode,
    // Up to 10 sprites per scanline.
    sprites: Vec<Sprite>,
    // Up to 456 dots per scanline.
    dots: u16,
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            vram: boxed_array(0),
            tile_map: boxed_array(0),
            oam: boxed_array(0),
            lcd: LCD { lcdc: 0, stat: 0, ly: 0, lyc: 0, wy: 0, wx: 0, scy: 0, scx: 0 },
            mode: Mode::OamScan,
            sprites: vec![],
            dots: 0,
        }
    }

    pub fn step(&mut self) {
        match self.mode {
            Mode::OamScan => self.step_oam_scan(),
            Mode::Drawing => self.step_drawing(),
            Mode::HBlank => self.step_hblank(),
            Mode::VBlank => self.step_vblank(),
        }
    }

    fn obj_size(&self) -> u8 {
        if self.lcd.lcdc & 0b100 != 0 {
            2 // 8x16
        } else {
            1 // 8x8
        }
    }

    /// 持续80dots，结束后进入Drawing状态。
    fn step_oam_scan(&mut self) {
        self.dots += 1;
        if self.dots == 1 {
            let obj_size = self.obj_size();
            for sprite_idx in 0..40 {
                let sprite = unsafe {
                    let base_addr = sprite_idx as usize * 4;
                    std::mem::transmute_copy::<[u8; 4], Sprite>(
                        &self.oam[base_addr..base_addr + 4].try_into().unwrap(),
                    )
                };
                let scy_top = sprite.y - 16;
                let scy_bottom_exclusive = scy_top + (8 * obj_size);

                // https://gbdev.io/pandocs/OAM.html#:~:text=since%20the%20ppu%20only%20checks%20the%20y%20coordinate%20to%20select%20objects
                if (self.lcd.ly >= scy_top) && (self.lcd.ly < scy_bottom_exclusive) {
                    self.sprites.push(sprite);
                }
                if self.sprites.len() >= 10 {
                    break;
                }
            }
        } else if self.dots == 80 {
            self.mode = Mode::Drawing;
        }
    }

    /// 持续172-289dots，加载Win/BG的tile，和sprite做像素合成。
    /// 结束后进入HBlank状态。
    fn step_drawing(&mut self) {
        todo!()
    }

    /// 持续到scanline结束（456dots），结束后如果当前scanline为153，
    /// 则进入VBlank状态。
    fn step_hblank(&mut self) {
        todo!()
    }

    /// 持续10scanlines，结束后进入OamScan状态。
    fn step_vblank(&mut self) {
        todo!()
    }
}

impl io::IO for PPU {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x97FF => self.vram[addr as usize - 8000] = value,
            0x9800..=0x9FFF => self.tile_map[addr as usize - 0x9800] = value,
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00] = value,
            0xFF40 => self.lcd.lcdc = value,
            0xFF41 => self.lcd.stat = value,
            0xFF42 => self.lcd.scy = value,
            0xFF43 => self.lcd.scx = value,
            0xFF44 => self.lcd.ly = value,
            0xFF45 => self.lcd.lyc = value,
            0xFF4A => self.lcd.wy = value,
            0xFF4B => self.lcd.wx = value,
            _ => unreachable!("Invalid PPU address: {:#X}", addr),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x97FF => self.vram[addr as usize - 8000],
            0x9800..=0x9FFF => self.tile_map[addr as usize - 0x9800],
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00],
            0xFF40 => self.lcd.lcdc,
            0xFF41 => self.lcd.stat,
            0xFF42 => self.lcd.scy,
            0xFF43 => self.lcd.scx,
            0xFF44 => self.lcd.ly,
            0xFF45 => self.lcd.lyc,
            0xFF4A => self.lcd.wy,
            0xFF4B => self.lcd.wx,
            _ => unreachable!("Invalid PPU address: {:#X}", addr),
        }
    }
}
