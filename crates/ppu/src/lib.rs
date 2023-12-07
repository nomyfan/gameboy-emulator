mod config;
mod lcd;
mod pixel;
mod sprite;
mod tile;

use crate::config::{
    COLOR_PALETTES, DOTS_PER_SCANLINE, RESOLUTION_X, RESOLUTION_Y, SCANLINES_PER_FRAME,
};
use crate::lcd::{LCDMode, LCD};
use crate::sprite::Sprite;
use crate::tile::{BackgroundTileDataBuilder, SpriteTileDataBuilder, TileData, TileDataBuilder};
use gb_shared::{boxed_array, boxed_array_fn, is_bit_set, pick_bits};
use log::debug;

/// The first fourth steps takes 2 dots each.
/// The fifth step is attempted every dot until it succeeds.
#[derive(Debug, Default)]
pub(crate) enum RenderStage {
    #[default]
    GetTile,
    GetTileDataLow,
    GetTileDataHigh,
    Sleep,
    Push,
}

#[derive(Debug, Default)]
pub(crate) struct PPUWorkState {
    render_stage: RenderStage,
    /// X of current scanline.
    /// Reset when moving to next scanline.
    scanline_x: u8,
    /// One dot equals 4 CPU cycles. It's 2 cycles when it's CGB
    /// and CPU is in double speed mode.
    ///
    /// There are 456 dots per scanline, so there are 70224(456 * 154)
    /// dots per frame.
    /// Reset to 0 when enter to next scanline.
    scanline_dots: u16,
    /// Up to 10 sprites per scanline.
    /// Appended in mode 2(OAM scan).
    /// Reset when moving to next scanline.
    scanline_sprites: Vec<Sprite>,
    /// X coordination of current pixel.
    /// scx + scanline_x
    /// Updated in mode 3(render a pixel).
    map_x: u8,
    /// Y coordination of current pixel.
    /// scy + ly
    /// Updated in mode3(render a pixel).
    map_y: u8,
    /// Reset in FIFO push stage(by taking out the value).
    bgw_tile_builder: Option<BackgroundTileDataBuilder>,
    /// Reset in FIFO push stage(by taking out the value).
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
    oam: Box<[u8; 4 * 40]>,
    lcd: LCD,
    /// BG palette, at 0xFF47.
    bgp: u8,
    /// OBJ palette 0, at 0xFF48.
    obp0: u8,
    /// OBJ palette 1, at 0xFF49.
    obp1: u8,
    /// PPU work state.
    work_state: PPUWorkState,
    video_buffer: Box<[[u32; RESOLUTION_X as usize]; RESOLUTION_Y as usize]>,
}

impl PPU {
    // TODO: add utils to read tile data and tile map from vram.
    pub fn new() -> Self {
        // TODO: check init
        // TODO: impl Default trait
        PPU {
            vram: boxed_array(0),
            oam: boxed_array(0),
            lcd: LCD::default(),
            work_state: PPUWorkState::default(),
            video_buffer: boxed_array_fn(|_| [0; RESOLUTION_X as usize]),
            bgp: 0,
            obp0: 0,
            obp1: 0,
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
            (256 + index as i16) as usize // It must be positive now before casting to usize.
        };
        let addr = index * 16 + (if is_high { 8 } else { 0 });
        self.vram[addr..(addr + 8)].try_into().unwrap()
    }

    fn select_color(&self, bgw: &Option<TileData>, sprite: &Option<TileData>) -> u32 {
        let x = self.work_state.map_x % 8;
        let y = self.work_state.map_y % 8;
        // Priority definition
        // 1. If BGW' color ID is 0, then render the sprite.
        // 2. If LCDC.0 is 0, then render the sprite.
        // 3. If OAM attributes.7 is 0, then render the sprite.
        // 4. Otherwise, render the BGW.

        let mut color = 0;
        let mut color_id = 0;
        if let Some(tile) = bgw {
            if self.lcd.is_bgw_enabled() {
                color_id = tile.get_color_id(x, y);
                let offset = color_id * 2;
                let palette = (pick_bits!(self.bgp, offset, offset + 1)) >> offset;
                color = COLOR_PALETTES[palette as usize];
            }
        }

        if color_id == 0 {
            if let Some(tile) = sprite {
                let attrs = tile.sprite_attrs.as_ref().unwrap();
                if !attrs.bgw_over_object() {
                    color_id = tile.get_color_id(x, y);
                    let obp = if attrs.dmg_palette() == 0 { self.obp0 } else { self.obp1 };
                    let offset = color_id * 2;
                    let palette = pick_bits!(obp, offset, offset + 1) >> offset;
                    color = COLOR_PALETTES[palette as usize];
                }
            }
        }

        color
    }

    pub fn step(&mut self) {
        self.work_state.scanline_dots += 1;
        match self.mode() {
            LCDMode::OamScan => self.step_oam_scan(),
            LCDMode::RenderPixel => self.step_render_pixel(),
            LCDMode::HBlank => self.step_hblank(),
            LCDMode::VBlank => self.step_vblank(),
        }
    }

    /// 持续80dots，结束后进入Drawing状态。
    fn step_oam_scan(&mut self) {
        if self.work_state.scanline_dots == 1 {
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
                    self.work_state.scanline_sprites.push(sprite);
                }
                // https://gbdev.io/pandocs/OAM.html?highlight=10#selection-priority
                if self.work_state.scanline_sprites.len() >= 10 {
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
            self.work_state.scanline_sprites.sort_by(|a, b| a.x.cmp(&b.x));
            debug!("{:?}", &self.work_state.scanline_sprites);
        } else if self.work_state.scanline_dots == 80 {
            self.set_mode(LCDMode::RenderPixel);
        }
    }

    /// 持续172-289dots，加载Win/BG的tile，和sprite做像素合成。
    /// 结束后进入HBlank状态。
    fn step_render_pixel(&mut self) {
        // TODO: penalty for canceling
        // https://gbdev.io/pandocs/pixel_fifo.html#object-fetch-canceling
        // https://gbdev.io/pandocs/Rendering.html#mode-3-length
        self.work_state.map_x = self.work_state.scanline_x.wrapping_add(self.lcd.scx);
        self.work_state.map_y = self.lcd.ly.wrapping_add(self.lcd.scy);

        // Extra 12 dots are needed for fetching two tiles at the beginning of mode 3.
        // https://gbdev.io/pandocs/Rendering.html#:~:text=the%2012%20extra%20cycles%20come%20from%20two%20tile%20fetches%20at%20the%20beginning%20of%20mode%203
        if self.work_state.scanline_dots <= (80 + 12) {
            return;
        }

        match self.work_state.render_stage {
            RenderStage::GetTile => {
                if self.lcd.is_bgw_enabled() {
                    let index =
                        self.get_tile_index(self.work_state.map_x, self.work_state.map_y, false);
                    self.work_state.bgw_tile_builder.replace(BackgroundTileDataBuilder::new(index));

                    if self.lcd.is_window_enabled() {
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

                            self.work_state
                                .bgw_tile_builder
                                .replace(BackgroundTileDataBuilder::new(index));
                        }
                    }
                }

                if self.lcd.is_obj_enabled() {
                    let builder = self
                        .work_state
                        .scanline_sprites
                        .iter()
                        .find(|sprite| {
                            let x = sprite.x as i16 - 8 + self.lcd.scx as i16;
                            let scanline_x = self.work_state.scanline_x as i16;

                            // Two rectangles intersect.
                            x + 8 >= scanline_x && x < scanline_x + 8
                        })
                        .map(|sprite| SpriteTileDataBuilder::new(*sprite, self.lcd.object_size()));

                    self.work_state.sprite_tile_builder = builder;
                }

                self.work_state.render_stage = RenderStage::GetTileDataLow;
            }
            RenderStage::GetTileDataLow => {
                if let Some(mut builder) = self.work_state.bgw_tile_builder.take() {
                    builder.low(self.read_tile_data(builder.index, false, false));
                    self.work_state.bgw_tile_builder.replace(builder);
                }

                if let Some(mut builder) = self.work_state.sprite_tile_builder.take() {
                    builder.low(self.read_tile_data(builder.tile_index(), true, false));
                    self.work_state.sprite_tile_builder.replace(builder);
                }

                self.work_state.render_stage = RenderStage::GetTileDataHigh;
            }
            RenderStage::GetTileDataHigh => {
                if let Some(mut builder) = self.work_state.bgw_tile_builder.take() {
                    builder.high(self.read_tile_data(builder.index, false, true));
                    self.work_state.bgw_tile_builder.replace(builder);
                }

                if let Some(mut builder) = self.work_state.sprite_tile_builder.take() {
                    builder.high(self.read_tile_data(builder.tile_index(), true, true));
                    self.work_state.sprite_tile_builder = Some(builder);
                }

                self.work_state.render_stage = RenderStage::Sleep;
            }
            RenderStage::Sleep => {
                self.work_state.render_stage = RenderStage::Push;
            }
            RenderStage::Push => {
                let bgw_tile =
                    self.work_state.bgw_tile_builder.take().map(|builder| builder.build());
                let sprite_tile =
                    self.work_state.sprite_tile_builder.take().map(|builder| builder.build());

                let color = self.select_color(&bgw_tile, &sprite_tile);
                let viewport_x = self.work_state.scanline_x as usize;
                let viewport_y = self.lcd.ly as usize;
                self.video_buffer[viewport_y][viewport_x] = color;

                self.work_state.scanline_x += 1;
                self.work_state.render_stage = RenderStage::GetTile;
            }
        }

        // Pixels in current scanline are all rendered.
        if self.work_state.scanline_x >= RESOLUTION_X {
            // TODO: LCD interrupts
            self.work_state.render_stage = RenderStage::GetTile;
            self.set_mode(LCDMode::HBlank);
        }
    }

    /// 持续到scanline结束（456dots），结束后如果当前scanline为153，
    /// 则进入VBlank状态。
    fn step_hblank(&mut self) {
        if self.work_state.scanline_dots < DOTS_PER_SCANLINE {
            return;
        }
        // Enter new line
        self.work_state.scanline_dots = 0;
        self.work_state.scanline_x = 0;
        self.work_state.scanline_sprites.clear();
        self.lcd.ly += 1;
        // TODO: LCD interrupts

        if self.lcd.ly >= RESOLUTION_Y {
            self.set_mode(LCDMode::VBlank);
        } else {
            self.set_mode(LCDMode::OamScan);
        }
    }

    /// 持续10scanlines，结束后进入OamScan状态。
    fn step_vblank(&mut self) {
        if self.work_state.scanline_dots >= DOTS_PER_SCANLINE {
            self.work_state.scanline_dots = 0;
            self.lcd.ly += 1;

            if self.lcd.ly >= SCANLINES_PER_FRAME {
                self.lcd.ly = 0;
                self.set_mode(LCDMode::OamScan);
            }
        }
    }
}

impl gb_shared::Memory for PPU {
    fn write(&mut self, addr: u16, value: u8) {
        // TODO: block some writes while PPU operating on it.
        // https://gbdev.io/pandocs/Rendering.html#:~:text=accessible%20video%20memory
        match addr {
            0x8000..=0x9FFF => self.vram[addr as usize - 8000] = value,
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00] = value,
            0xFF40 => self.lcd.lcdc = value,
            0xFF41 => self.lcd.stat = value,
            0xFF42 => self.lcd.scy = value,
            0xFF43 => self.lcd.scx = value,
            0xFF44 => self.lcd.ly = value,
            0xFF45 => self.lcd.lyc = value,
            0xFF46 => self.bgp = value,
            0xFF47 => self.obp0 = value,
            0xFF48 => self.obp1 = value,
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
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.lcd.wy,
            0xFF4B => self.lcd.wx,
            _ => unreachable!("Invalid PPU address: {:#X}", addr),
        }
    }
}