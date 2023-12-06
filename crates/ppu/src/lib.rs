mod config;
mod lcd;
mod pixel;
mod sprite;
mod tile;

use config::{DOTS_PER_SCANLINE, RESOLUTION_X, RESOLUTION_Y, SCANLINES_PER_FRAME};
use gb_shared::{boxed_array, is_bit_set};
use lcd::{LCDMode, LCD};
use log::debug;
use sprite::Sprite;
use tile::{BackgroundTileDataBuilder, SpriteTileDataBuilder, TileDataBuilder};

/// The first fourth steps takes 2 dots each.
/// The fifth step is attempted every dot until it succeeds.
#[derive(Debug, Default)]
pub(crate) enum RenderStatus {
    #[default]
    GetTileIndex,
    GetTileDataLow,
    GetTileDataHigh,
    Sleep,
    Push,
}

#[derive(Debug, Default)]
pub(crate) struct PPUWorkState {
    render_status: RenderStatus,
    scanline_x: u8,
    /// X coordination of current pixel.
    /// scx + scanline_x
    map_x: u8,
    /// Y coordination of current pixel.
    /// scy + ly
    map_y: u8,
    bgw_tile_builder: BackgroundTileDataBuilder,
    /// Sprite tile data
    sprite_tile_builder: Option<SpriteTileDataBuilder>,
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
            lcd: LCD::default(),
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
        self.vram[vram_offset as usize]
    }

    fn read_tile_data(&self, index: u8, for_sprite: bool, is_high: bool) -> [u8; 8] {
        let index = if for_sprite || is_bit_set!(self.lcd.lcdc, 4) {
            index as usize
        } else {
            let index = index as i8; // Make it able to be negative.
            (256i16 + index as i16) as usize // It must be positive now before casting to usize.
        };
        let addr = index * 16 + (if is_high { 8 } else { 0 });
        self.vram[addr..(addr + 8)].try_into().unwrap()
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

    /// 持续80dots，结束后进入Drawing状态。
    fn step_oam_scan(&mut self) {
        if self.scanline_dots == 1 {
            let obj_size = self.lcd.object_size();
            for sprite_idx in 0..40usize {
                let sprite = unsafe {
                    let base_addr = sprite_idx * 4;
                    std::mem::transmute_copy::<[u8; 4], Sprite>(
                        &self.oam[base_addr..(base_addr + 4)].try_into().unwrap(),
                    )
                };
                // https://gbdev.io/pandocs/OAM.html#:~:text=since%20the%20gb_ppu::%20only%20checks%20the%20y%20coordinate%20to%20select%20objects
                // The sprite intersects with current line.
                if (self.lcd.ly + 16 >= sprite.y) && (self.lcd.ly + 16 < sprite.y + obj_size) {
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
                let row_index = self.work_state.map_y % 8;
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
                            self.work_state.bgw_tile_builder =
                                BackgroundTileDataBuilder::new(index, row_index);
                        }
                    } else {
                        let index = self.get_tile_index(
                            self.work_state.map_x,
                            self.work_state.map_y,
                            false,
                        );
                        self.work_state.bgw_tile_builder =
                            BackgroundTileDataBuilder::new(index, row_index);
                    }
                }
                // Sprite is enabled.
                if is_bit_set!(self.lcd.lcdc, 1) {
                    if let Some(sprite) = self.scanline_sprites.iter().find(|sprite| {
                        let x = sprite.x as i16 - 8 + self.lcd.scx as i16;
                        self.work_state.scanline_x as i16 >= x
                            && (self.work_state.scanline_x as i16) < x + 8
                    }) {
                        self.work_state.sprite_tile_builder = Some(SpriteTileDataBuilder::new(
                            *sprite,
                            self.lcd.object_size(),
                            row_index,
                        ));
                    }
                }

                self.work_state.scanline_x += 8;
                self.work_state.render_status = RenderStatus::GetTileDataLow;
            }
            RenderStatus::GetTileDataLow => {
                self.work_state.bgw_tile_builder.low(self.read_tile_data(
                    self.work_state.bgw_tile_builder.index,
                    false,
                    false,
                ));

                if let Some(mut builder) = self.work_state.sprite_tile_builder.take() {
                    builder.low(self.read_tile_data(builder.tile_index(), true, false));
                    self.work_state.sprite_tile_builder = Some(builder);
                }

                self.work_state.render_status = RenderStatus::GetTileDataHigh;
            }
            RenderStatus::GetTileDataHigh => {
                self.work_state.bgw_tile_builder.high(self.read_tile_data(
                    self.work_state.bgw_tile_builder.index,
                    false,
                    true,
                ));

                if let Some(mut builder) = self.work_state.sprite_tile_builder.take() {
                    builder.high(self.read_tile_data(builder.tile_index(), true, true));
                    self.work_state.sprite_tile_builder = Some(builder);
                }

                self.work_state.render_status = RenderStatus::Sleep;
            }
            RenderStatus::Sleep => {
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
