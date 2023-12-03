use gb_shared::boxed_array;
use log::debug;

#[repr(C)]
#[derive(Debug)]
struct Sprite {
    y: u8,
    x: u8,
    tile_index: u8,
    attrs: u8,
}

#[repr(u8)]
enum Mode {
    /// OAM is inaccessible(except DMA) during this period.
    OamScan = 2,
    /// VRAM is inaccessible during this period.
    Drawing = 3,
    HBlank = 0,
    /// Everything is accessible during this period.
    VBlank = 1,
}

impl From<&LCD> for Mode {
    fn from(lcd: &LCD) -> Self {
        let value = lcd.stat & 0b11;
        unsafe { std::mem::transmute::<[u8; 1], Self>([value]) }
    }
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
    /// TODO: since bit 0-2 is read only, we need to ignore write operations on these bits.
    /// - Bit 6 - LYC=LY STAT Interrupt source         (1=Enable) (Read/Write)
    /// - Bit 5 - Mode 2 OAM STAT Interrupt source     (1=Enable) (Read/Write)
    /// - Bit 4 - Mode 1 VBlank STAT Interrupt source  (1=Enable) (Read/Write)
    /// - Bit 3 - Mode 0 HBlank STAT Interrupt source  (1=Enable) (Read/Write)
    /// - Bit 2 - LYC=LY Flag                          (0=Different, 1=Equal) (Read Only)
    /// - Bit 1-0 - Mode Flag                          (Mode 0-3, see below) (Read Only)
    ///           0: HBlank
    ///           1: VBlank
    ///           2: Searching OAM
    ///           3: Transferring Data to LCD Controller
    stat: u8,
    /// LCD Y coordinate, at 0xFF44.
    /// Read only, it represents current scanline.
    ly: u8,
    /// LCD Y compare, at 0xFF45.
    /// When LYC == LY, LYC=LY flag is set, and (if enabled) a STAT interrupt is requested.
    lyc: u8,
    /// Window Y position, at 0xFF4A.
    wy: u8,
    /// Window X position plus 7, at 0xFF4B.
    wx: u8,
    /// Viewport Y position, at 0xFF42.
    scy: u8,
    /// Viewport X position, at 0xFF43.
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
    /// - Tile map 0: \[0x9800, 0x9BFF]
    /// - Tile map 1: \[0x9C00, 0x9FFF]
    tile_map: Box<[u8; 0x200]>,
    /// \[0xFE00, 0xFE9F]
    /// OAM(Object Attribute Memory) is used to store sprites(or objects).
    /// There're up to 40 sprites. Each entry consists of 4 bytes.
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
    /// - Bit 7: BG and Window over OBJ(0=No, 1=BG and Window colors 1-3 are drawn over the OBJ)
    oam: Box<[u8; 4 * 40]>,
    lcd: LCD,
    // Up to 10 sprites per scanline.
    sprites: Vec<Sprite>,
    // Up to 456 dots per scanline.
    dots: u16,
}

impl PPU {
    pub fn new() -> Self {
        // TODO: check init
        PPU {
            vram: boxed_array(0),
            tile_map: boxed_array(0),
            oam: boxed_array(0),
            lcd: LCD { lcdc: 0b10010001, stat: 0b10, ly: 0, lyc: 0, wy: 0, wx: 0, scy: 0, scx: 0 },
            sprites: Vec::with_capacity(10), // There are up to 10 sprites.
            dots: 0,
        }
    }

    fn mode(&self) -> Mode {
        Mode::from(&self.lcd)
    }

    fn set_mode(&mut self, mode: Mode) {
        // Unset bit 0 and bit 1
        let mut lcdc = self.lcd.lcdc & (!0b11);
        // Set bit 0 and bit 1
        lcdc |= mode as u8 & 0b11;
        self.lcd.lcdc = lcdc;
    }

    pub fn step(&mut self) {
        match self.mode() {
            Mode::OamScan => self.step_oam_scan(),
            Mode::Drawing => self.step_drawing(),
            Mode::HBlank => self.step_hblank(),
            Mode::VBlank => self.step_vblank(),
        }
    }

    #[inline]
    fn obj_size(&self) -> u8 {
        if self.lcd.lcdc & 0b100 != 0 {
            2 // 8x16
        } else {
            1 // 8x8
        }
    }

    /// 持续80dots，结束后进入Drawing状态。
    fn step_oam_scan(&mut self) {
        self.dots += 1; // TODO cycles
        if self.dots == 1 {
            let obj_size = self.obj_size();
            for sprite_idx in 0..40usize {
                let sprite = unsafe {
                    let base_addr = sprite_idx * 4;
                    std::mem::transmute_copy::<[u8; 4], Sprite>(
                        &self.oam[base_addr..base_addr + 4].try_into().unwrap(),
                    )
                };
                // https://gbdev.io/pandocs/OAM.html#:~:text=since%20the%20gb_ppu::%20only%20checks%20the%20y%20coordinate%20to%20select%20objects
                // The sprite intersects with current line.
                if (self.lcd.ly + 16 >= sprite.y) && (self.lcd.ly + 16 < sprite.y + (8 * obj_size))
                {
                    self.sprites.push(sprite);
                }
                if self.sprites.len() >= 10 {
                    break;
                }
            }
            // https://gbdev.io/pandocs/OAM.html#drawing-priority
            //
            // For Non-CGB, the smaller X, the higher priority.
            // If the X is same, sprite located first has higher priority.
            //
            // For CGB, the priority is determined by the location in OAM.
            // The earlier the sprite, the higher its priority.
            //
            // It's worth to mention that `sort_by` is stable.
            self.sprites.sort_by(|a, b| a.x.cmp(&b.x));
            debug!("{:?}", &self.sprites);
        } else if self.dots == 80 {
            self.set_mode(Mode::Drawing);
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

impl gb_io::IO for PPU {
    fn write(&mut self, addr: u16, value: u8) {
        // TODO: block some writes while PPU operating on it.
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
        // TODO: block some reads(return 0xFF) while PPU operating on it.
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
