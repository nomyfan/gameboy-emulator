use gb_shared::{boxed_array, is_bit_set};
use log::debug;

const DOTS_PER_SCANLINE: u16 = 456;
const SCANLINES_PER_FRAME: u8 = 154;
const RESOLUTION_Y: u8 = 144;
const RESOLUTION_X: u8 = 160;

/// https://gbdev.io/pandocs/OAM.html
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Sprite {
    /// Sprite's Y position on the screen + 16.
    y: u8,
    /// Sprite's X position on the screen + 8.
    x: u8,
    tile_index: u8,
    attrs: u8,
}

/// The first fourth steps takes 2 dots each.
/// The fifth step is attempted every dot until it succeeds.
#[derive(Debug, Default)]
enum RenderStatus {
    #[default]
    GetTileIndex,
    GetTileDataLow,
    GetTileDataHigh,
    Sleep,
    Push,
}

#[derive(Debug, Default)]
struct TileData {
    index: u8,
    low: [u8; 16],
    high: [u8; 16],
}

#[derive(Debug, Default)]
struct TileDataBuilder {
    index: u8,
    _low: Option<[u8; 16]>,
    _high: Option<[u8; 16]>,
}

impl TileDataBuilder {
    fn new(index: u8) -> Self {
        TileDataBuilder { index, _low: None, _high: None }
    }

    fn low(mut self, data: [u8; 16]) -> Self {
        self._low = Some(data);
        self
    }

    fn high(mut self, data: [u8; 16]) -> Self {
        self._high = Some(data);
        self
    }

    fn build(mut self) -> TileData {
        let Some(low) = self._low else { panic!("low data is not set") };
        let Some(high) = self._high else { panic!("high data is not set") };
        TileData { index: self.index, low, high }
    }
}

#[derive(Debug, Default)]
struct PPUWorkState {
    render_status: RenderStatus,
    scanline_x: u8,
    /// X coordination of current pixel.
    /// scx + scanline_x
    map_x: u8,
    /// Y coordination of current pixel.
    /// scy + ly
    map_y: u8,
    wip_bgw_tile: TileDataBuilder,
    /// Sprite for current tile.
    wip_sprite: Option<Sprite>,
    /// Sprite tile data
    wip_sprite_tile: Option<TileDataBuilder>,
}

#[repr(u8)]
enum LCDMode {
    /// OAM is inaccessible(except DMA) during this period.
    Scan = 2,
    /// VRAM is inaccessible during this period.
    Render = 3,
    HBlank = 0,
    /// Everything is accessible during this period.
    VBlank = 1,
}

impl From<&LCD> for LCDMode {
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
    /// Tile data area(in size of 0x1800).
    /// There are total 384 tiles, each tile has 16 bytes.
    /// Thus, the size of this area is 6KB.
    /// - Block 0: \[0x8000, 0x87FF]
    /// - Block 1: \[0x8800, 0x8FFF]
    /// - Block 2: \[0x9000, 0x97FF]
    ///
    /// There're two addressing modes. Mode A indexes OBJ
    /// 0-127 in block 0 and indexes OBJ 128-255 in block 1.
    /// Mode B indexes OBJ 128-255 in block 1 and indexes
    /// OBJ 0-127 in block 2.
    ///
    // For BG and Window, if LCDC.4 is 1, then mode
    /// A is used, and if LCDC.4 is 0 then mode B is used.
    /// For sprites, the mode is always A.
    ///
    /// Tile map area(in size of 0x800).
    /// - Tile map 0: \[0x9800, 0x9BFF]
    /// - Tile map 1: \[0x9C00, 0x9FFF]
    vram: Box<[u8; 0x2000]>,
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
    scanline_sprites: Vec<Sprite>,
    /// One dot equals 4 CPU cycles. It's 2 cycles when it's CGB
    /// and CPU is in double speed mode.
    ///
    /// There are 456 dots per scanline, so there are 70224(456 * 154)
    /// dots per frame.
    /// TODO: reset this to zero when enter new scanline.
    scanline_dots: u16,
    /// PPU work state.
    work_state: PPUWorkState,
}

impl PPU {
    // TODO: add utils to read tile data and tile map from vram.
    pub fn new() -> Self {
        // TODO: check init
        PPU {
            vram: boxed_array(0),
            oam: boxed_array(0),
            lcd: LCD { lcdc: 0b10010001, stat: 0b10, ly: 0, lyc: 0, wy: 0, wx: 0, scy: 0, scx: 0 },
            scanline_sprites: Vec::with_capacity(10), // There are up to 10 sprites.
            scanline_dots: 0,
            work_state: PPUWorkState::default(),
        }
    }

    fn mode(&self) -> LCDMode {
        LCDMode::from(&self.lcd)
    }

    fn set_mode(&mut self, mode: LCDMode) {
        // Unset bit 0 and bit 1
        let mut lcdc = self.lcd.lcdc & (!0b11);
        // Set bit 0 and bit 1
        lcdc |= mode as u8 & 0b11;
        self.lcd.lcdc = lcdc;
    }

    /// For BGW only. Tile index for sprite is stored
    /// in the sprite object.
    fn get_tile_index(&self, map_x: u8, map_y: u8, is_window: bool) -> u8 {
        let vram_addr: u16 =
            if is_bit_set!(self.lcd.lcdc, if is_window { 6 } else { 3 }) { 0x9C00 } else { 0x9800 };
        let vram_addr = vram_addr + ((map_y as u16 / 8) * 32 + (map_x as u16) / 8);

        let vram_offset = vram_addr - 0x8000;
        // TODO: read from bus
        self.vram[vram_offset as usize]
    }

    /// Get tile data from VRAM.
    fn get_tile_data(&self, index: u8, for_sprite: bool) -> [u8; 16] {
        let index = if for_sprite || is_bit_set!(self.lcd.lcdc, 4) {
            index as usize
        } else {
            let index = index as i8; // Make it able to be negative.
            (256i16 + index as i16) as usize // It must be positive now before casting to usize.
        };
        let addr = index * 16;
        // TODO: read from bus
        self.vram[addr..(addr + 16)].try_into().unwrap()
    }

    pub fn step(&mut self) {
        self.scanline_dots += 1;
        match self.mode() {
            LCDMode::Scan => self.step_oam_scan(),
            LCDMode::Render => self.step_drawing(),
            LCDMode::HBlank => self.step_hblank(),
            LCDMode::VBlank => self.step_vblank(),
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
        if self.scanline_dots == 1 {
            let obj_size = self.obj_size();
            for sprite_idx in 0..40usize {
                let sprite = unsafe {
                    let base_addr = sprite_idx * 4;
                    std::mem::transmute_copy::<[u8; 4], Sprite>(
                        &self.oam[base_addr..(base_addr + 4)].try_into().unwrap(),
                    )
                };
                // https://gbdev.io/pandocs/OAM.html#:~:text=since%20the%20gb_ppu::%20only%20checks%20the%20y%20coordinate%20to%20select%20objects
                // The sprite intersects with current line.
                if (self.lcd.ly + 16 >= sprite.y) && (self.lcd.ly + 16 < sprite.y + (8 * obj_size))
                {
                    self.scanline_sprites.push(sprite);
                }
                // https://gbdev.io/pandocs/OAM.html?highlight=10#selection-priority
                if self.scanline_sprites.len() >= 10 {
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
            self.scanline_sprites.sort_by(|a, b| a.x.cmp(&b.x));
            debug!("{:?}", &self.scanline_sprites);
        } else if self.scanline_dots == 80 {
            self.set_mode(LCDMode::Render);
        }
    }

    /// 持续172-289dots，加载Win/BG的tile，和sprite做像素合成。
    /// 结束后进入HBlank状态。
    fn step_drawing(&mut self) {
        // TODO: penalty for canceling
        // https://gbdev.io/pandocs/pixel_fifo.html#object-fetch-canceling
        // https://gbdev.io/pandocs/Rendering.html#mode-3-length
        self.work_state.map_x = self.work_state.scanline_x.wrapping_add(self.lcd.scx);
        self.work_state.map_y = self.lcd.ly.wrapping_add(self.lcd.scy);

        // Run once every 2 dots.
        if self.scanline_dots % 2 != 0 {
            return;
        }

        match self.work_state.render_status {
            RenderStatus::GetTileIndex => {
                // BGW is enabled.
                if is_bit_set!(self.lcd.lcdc, 0) {
                    // Window is enabled.
                    if is_bit_set!(self.lcd.lcdc, 5) {
                        if self.work_state.map_x as i16 + 7 >= self.lcd.wx as i16
                            && self.work_state.map_x as i16 + 7
                                < self.lcd.wx as i16 + RESOLUTION_X as i16
                            && self.work_state.map_y as i16 >= self.lcd.wy as i16
                            && (self.work_state.map_y as i16)
                                < self.lcd.wy as i16 + RESOLUTION_Y as i16
                        {
                            let index = self.get_tile_index(
                                self.work_state.map_x,
                                self.work_state.map_y,
                                true,
                            );
                            self.work_state.wip_bgw_tile = TileDataBuilder::new(index);
                        }
                    } else {
                        let index = self.get_tile_index(
                            self.work_state.map_x,
                            self.work_state.map_y,
                            false,
                        );
                        self.work_state.wip_bgw_tile = TileDataBuilder::new(index);
                    }
                }
                // Sprite is enabled.
                if is_bit_set!(self.lcd.lcdc, 1) {
                    if let Some(sprite) = self.scanline_sprites.iter().find(|sprite| {
                        let x = sprite.x as i16 - 8 + self.lcd.scx as i16;
                        self.work_state.scanline_x as i16 >= x
                            && (self.work_state.scanline_x as i16) < x + 8
                    }) {
                        self.work_state.wip_sprite = Some(*sprite);
                    }
                }

                self.work_state.scanline_x += 8;
                self.work_state.render_status = RenderStatus::GetTileDataLow;
            }
            RenderStatus::GetTileDataLow => {
                todo!("get tile data low");
                self.work_state.render_status = RenderStatus::GetTileDataHigh;
            }
            RenderStatus::GetTileDataHigh => {
                todo!("get tile data high");
                self.work_state.render_status = RenderStatus::Sleep;
            }
            RenderStatus::Sleep => {
                todo!("sleep");
                self.work_state.render_status = RenderStatus::Push;
            }
            RenderStatus::Push => {
                todo!("push");
                self.work_state.render_status = RenderStatus::GetTileIndex;
            }
        }

        todo!("only set mode to HBlank when all pixels pushed");
        self.set_mode(LCDMode::HBlank);
    }

    /// 持续到scanline结束（456dots），结束后如果当前scanline为153，
    /// 则进入VBlank状态。
    fn step_hblank(&mut self) {
        if self.scanline_dots < DOTS_PER_SCANLINE {
            return;
        }
        self.scanline_dots = 0;
        // Enter VBlank state.
        if self.lcd.ly >= RESOLUTION_Y {
            self.set_mode(LCDMode::VBlank);
        }
        todo!()
    }

    /// 持续10scanlines，结束后进入OamScan状态。
    fn step_vblank(&mut self) {
        if self.scanline_dots >= DOTS_PER_SCANLINE {
            self.scanline_dots = 0;
            todo!("enter new scanline");

            if self.lcd.ly >= SCANLINES_PER_FRAME {
                self.lcd.ly = 0;
                self.set_mode(LCDMode::Scan);
                todo!("finish one frame");
            }
        }
    }
}

impl gb_shared::Memory for PPU {
    fn write(&mut self, addr: u16, value: u8) {
        // TODO: block some writes while PPU operating on it.
        match addr {
            0x8000..=0x9FFF => self.vram[addr as usize - 8000] = value,
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
            0x8000..=0x9FFF => self.vram[addr as usize - 8000],
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

#[cfg(test)]
mod tests {
    use super::Sprite;

    #[test]
    fn sprite_size() {
        assert_eq!(4, std::mem::size_of::<Sprite>())
    }
}
