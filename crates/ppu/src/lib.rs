mod config;
mod lcd;
mod object;
mod tile;

use crate::config::{DOTS_PER_SCANLINE, RESOLUTION_X, RESOLUTION_Y, SCANLINES_PER_FRAME};
use crate::lcd::{LCDMode, LCD};
use crate::object::Object;
use crate::tile::{BackgroundTileDataBuilder, ObjectTileDataBuilder, TileData, TileDataBuilder};
use gb_shared::boxed::{BoxedArray, BoxedMatrix};
use gb_shared::event::{Event, EventSender};
use gb_shared::{is_bit_set, set_bits, unset_bits, InterruptRequest, Memory};

#[derive(Debug, Default)]
pub(crate) struct PPUWorkState {
    /// X of current scanline.
    /// Reset when moving to next scanline.
    scanline_x: u8,
    /// PPU working frequency is the same as CPU. A dot equals to one CPU clock cycle.
    ///
    /// There are 456 dots per scanline, so there are 70224(456 * 154)
    /// dots per frame.
    /// Reset to 0 when enter to next scanline.
    scanline_dots: u16,
    /// Up to 10 objects per scanline.
    /// Appended in mode 2(OAM scan).
    /// Reset when moving to next scanline.
    scanline_objects: Vec<Object>,
    /// X coordination of current pixel.
    /// scx + scanline_x
    /// Updated in mode 3(render a pixel).
    map_x: u8,
    /// Y coordination of current pixel.
    /// scy + ly
    /// Updated in mode3(render a pixel).
    map_y: u8,
}

pub struct PPU<BUS: Memory> {
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
    /// For objects, the mode is always A.
    ///
    /// Tile map area(in size of 0x800).
    /// - Tile map 0: \[0x9800, 0x9BFF]
    /// - Tile map 1: \[0x9C00, 0x9FFF]
    vram: BoxedArray<u8, 0x2000>,
    /// \[0xFE00, 0xFE9F]
    /// OAM(Object Attribute Memory) is used to store objects.
    /// There're up to 40 objects. Each entry consists of 4 bytes.
    /// - Byte 0: Y position.
    /// - Byte 1: X position.
    /// - Byte 2: tile index.
    /// - Byte 3: attributes.
    oam: BoxedArray<u8, 160>,
    lcd: LCD,
    /// BG palette, at 0xFF47.
    bgp: u8,
    /// OBJ palette 0, at 0xFF48.
    obp0: u8,
    /// OBJ palette 1, at 0xFF49.
    obp1: u8,
    /// PPU work state.
    work_state: PPUWorkState,
    /// Storing palettes.
    video_buffer: BoxedMatrix<u8, RESOLUTION_X, RESOLUTION_Y>,

    bus: BUS,
    pub event_sender: Option<EventSender>,
}

impl<BUS: Memory + InterruptRequest> PPU<BUS> {
    pub fn new(bus: BUS) -> Self {
        Self {
            vram: BoxedArray::default(),
            oam: BoxedArray::default(),
            lcd: LCD::default(),
            bgp: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            work_state: PPUWorkState::default(),
            video_buffer: BoxedMatrix::default(),
            bus,
            event_sender: None,
        }
    }

    fn lcd_mode(&self) -> LCDMode {
        LCDMode::from(self.lcd.stat)
    }

    fn set_lcd_mode(&mut self, mode: LCDMode) {
        // Unset bit 0 and bit 1
        let mut stat = self.lcd.stat;
        stat &= !0b11;
        // Set bit 0 and bit 1
        stat |= mode as u8 & 0b11;
        self.lcd.stat = stat;
    }

    fn get_bgw_tile_index(&self, map_x: u8, map_y: u8, is_window: bool) -> u8 {
        let vram_addr: u16 =
            if is_bit_set!(self.lcd.lcdc, if is_window { 6 } else { 3 }) { 0x9C00 } else { 0x9800 };
        let vram_addr = vram_addr + ((map_y as u16 / 8) * 32 + (map_x as u16) / 8);

        let vram_offset = vram_addr - 0x8000;
        self.vram[vram_offset as usize]
    }

    fn read_tile_data(&self, index: u8, for_object: bool, is_high: bool) -> [u8; 8] {
        let index = if for_object || is_bit_set!(self.lcd.lcdc, 4) {
            index as usize
        } else {
            let index = index as i8; // Make it able to be negative.
            (256 + index as i16) as usize // It must be positive now before casting to usize.
        };
        let vram_offset = index * 16 + (if is_high { 8 } else { 0 });
        self.vram[vram_offset..(vram_offset + 8)].try_into().unwrap()
    }

    fn select_palette(&self, bgw: &Option<TileData>, object: &Option<TileData>) -> u8 {
        // Priority definition
        // 1. If BGW' color ID is 0, then render the object.
        // 2. If LCDC.0 is 0, then render the object.
        // 3. If OAM attributes.7 is 0, then render the object.
        // 4. Otherwise, render the BGW.

        let mut color_id = 0;
        let mut palette = 0;
        if let Some(tile) = bgw {
            if self.lcd.is_bgw_enabled() {
                let x = self.work_state.map_x % 8;
                let y = self.work_state.map_y % 8;

                color_id = tile.get_color_id(x, y);
                let offset = color_id * 2;
                palette = (self.bgp >> offset) & 0b11;
            }
        }

        if color_id == 0 {
            if let Some(tile) = object {
                let object = tile.object.as_ref().unwrap();
                if !object.attrs.bgw_over_object() {
                    let x = (self.work_state.scanline_x + 8) - object.x;
                    let y = (self.lcd.ly + 16) - object.y;

                    color_id = tile.get_color_id(x, y);
                    let obp = if object.attrs.dmg_palette() == 0 { self.obp0 } else { self.obp1 };
                    let offset = color_id * 2;
                    palette = (obp >> offset) & 0b11;
                }
            }
        }

        palette
    }

    fn move_to_next_scanline(&mut self) {
        self.work_state.scanline_dots = 0;
        self.work_state.scanline_x = 0;
        self.work_state.scanline_objects.clear();
        self.lcd.ly += 1;

        if self.lcd.ly == self.lcd.lyc {
            self.lcd.stat = set_bits!(self.lcd.stat, 2);

            // LY == LYC stat interrupt
            if is_bit_set!(self.lcd.stat, 6) {
                self.bus.request_lcd_stat();
            }
        } else {
            self.lcd.stat = unset_bits!(self.lcd.stat, 2);
        }
    }

    pub fn step(&mut self) {
        self.work_state.scanline_dots += 1;
        match self.lcd_mode() {
            LCDMode::OamScan => self.step_oam_scan(),
            LCDMode::RenderPixel => self.step_render_pixel(),
            LCDMode::HBlank => self.step_hblank(),
            LCDMode::VBlank => self.step_vblank(),
        }
    }

    /// 持续80dots，结束后进入Drawing状态。
    fn step_oam_scan(&mut self) {
        if self.work_state.scanline_dots == 1 {
            // Mode 2(OAM scan) stat interrupt
            if is_bit_set!(self.lcd.stat, 5) {
                self.bus.request_lcd_stat();
            }

            let obj_size = self.lcd.object_size();
            for object_index in 0..40usize {
                let object = unsafe {
                    let base_addr = object_index * 4;
                    std::mem::transmute_copy::<[u8; 4], Object>(
                        &self.oam[base_addr..(base_addr + 4)].try_into().unwrap(),
                    )
                };
                // https://gbdev.io/pandocs/OAM.html#:~:text=since%20the%20gb_ppu::%20only%20checks%20the%20y%20coordinate%20to%20select%20objects
                // The object intersects with current line.
                if (object.y..(object.y + obj_size)).contains(&(self.lcd.ly + 16)) {
                    self.work_state.scanline_objects.push(object);
                }
                // https://gbdev.io/pandocs/OAM.html?highlight=10#selection-priority
                if self.work_state.scanline_objects.len() >= 10 {
                    break;
                }
            }
            // https://gbdev.io/pandocs/OAM.html#drawing-priority
            //
            // For Non-CGB, the smaller X, the higher priority.
            // If the X is same, object located first has higher priority.
            //
            // For CGB, the priority is determined by the location in OAM.
            // The earlier the object, the higher its priority.
            //
            // It's worth mentioning that `sort_by` is stable.
            self.work_state.scanline_objects.sort_by(|a, b| a.x.cmp(&b.x));
        } else if self.work_state.scanline_dots == 80 {
            self.set_lcd_mode(LCDMode::RenderPixel);
        }
    }

    /// 持续172-289dots，加载Win/BG的tile，和object做像素合成。
    /// 结束后进入HBlank状态。
    fn step_render_pixel(&mut self) {
        // TODO: penalty for canceling?
        // https://gbdev.io/pandocs/pixel_fifo.html#object-fetch-canceling
        // https://gbdev.io/pandocs/Rendering.html#mode-3-length
        self.work_state.map_x = self.work_state.scanline_x.wrapping_add(self.lcd.scx);
        self.work_state.map_y = self.lcd.ly.wrapping_add(self.lcd.scy);

        // Extra 12 dots are needed for fetching two tiles at the beginning of mode 3.
        // https://gbdev.io/pandocs/Rendering.html#:~:text=the%2012%20extra%20cycles%20come%20from%20two%20tile%20fetches%20at%20the%20beginning%20of%20mode%203
        if self.work_state.scanline_dots <= (80 + 12) {
            return;
        }

        let mut bgw_tile_builder = None;
        let mut object_tile_builder = None;

        {
            if self.lcd.is_bgw_enabled() {
                let index =
                    self.get_bgw_tile_index(self.work_state.map_x, self.work_state.map_y, false);
                bgw_tile_builder.replace(BackgroundTileDataBuilder::new(index));

                if self.lcd.is_window_enabled()
                    && ((self.lcd.wx as u16)..(self.lcd.wx as u16 + RESOLUTION_X as u16))
                        .contains(&(self.work_state.map_x as u16 + 7))
                    && ((self.lcd.wy as u16)..(self.lcd.wy as u16 + RESOLUTION_Y as u16))
                        .contains(&(self.work_state.map_y as u16))
                {
                    let index =
                        self.get_bgw_tile_index(self.work_state.map_x, self.work_state.map_y, true);

                    bgw_tile_builder.replace(BackgroundTileDataBuilder::new(index));
                }
            }

            if self.lcd.is_obj_enabled() {
                let builder = self
                    .work_state
                    .scanline_objects
                    .iter()
                    .find(|object| {
                        let sx = self.work_state.scanline_x + 8;
                        sx >= object.x && sx < object.x + 8
                    })
                    .map(|object| ObjectTileDataBuilder::new(*object, self.lcd.object_size()));

                object_tile_builder = builder;
            }
        }

        {
            if let Some(mut builder) = bgw_tile_builder.take() {
                builder.low(self.read_tile_data(builder.index, false, false));
                bgw_tile_builder.replace(builder);
            }

            if let Some(mut builder) = object_tile_builder.take() {
                builder.low(self.read_tile_data(builder.tile_index(), true, false));
                object_tile_builder.replace(builder);
            }
        }

        {
            if let Some(mut builder) = bgw_tile_builder.take() {
                builder.high(self.read_tile_data(builder.index, false, true));
                bgw_tile_builder.replace(builder);
            }

            if let Some(mut builder) = object_tile_builder.take() {
                builder.high(self.read_tile_data(builder.tile_index(), true, true));
                object_tile_builder = Some(builder);
            }
        }

        {
            let bgw_tile = bgw_tile_builder.take().map(|builder| builder.build());
            let object_tile = object_tile_builder.take().map(|builder| builder.build());

            let palette = self.select_palette(&bgw_tile, &object_tile);
            let viewport_x = self.work_state.scanline_x as usize;
            let viewport_y = self.lcd.ly as usize;
            self.video_buffer[viewport_y][viewport_x] = palette;

            self.work_state.scanline_x += 1;
        }

        // Pixels in current scanline are all rendered.
        if self.work_state.scanline_x >= RESOLUTION_X as u8 {
            self.set_lcd_mode(LCDMode::HBlank);

            // Mode 0(HBlank) stat interrupt
            if is_bit_set!(self.lcd.stat, 3) {
                self.bus.request_lcd_stat();
            }
        }
    }

    /// 持续到scanline结束（456dots），结束后如果当前scanline为153，
    /// 则进入VBlank状态。
    fn step_hblank(&mut self) {
        if self.work_state.scanline_dots < DOTS_PER_SCANLINE {
            return;
        }

        self.move_to_next_scanline();

        if self.lcd.ly >= RESOLUTION_Y as u8 {
            self.set_lcd_mode(LCDMode::VBlank);

            // VBlank interrupt
            self.bus.request_vblank();
            // Mode 1(VBlank) stat interrupt
            if is_bit_set!(self.lcd.stat, 4) {
                self.bus.request_lcd_stat();
            }

            // Notify that a frame is rendered.
            if let Some(event_sender) = self.event_sender.as_ref() {
                event_sender.send(Event::OnFrame(self.video_buffer.clone())).unwrap();

                #[cfg(debug_assertions)]
                {
                    use crate::tile::mix_colors;

                    // log::info!("write to vram {:#X} = {:#X}", addr, value);
                    let data = self
                        .vram
                        .chunks(16)
                        .map(|chunk| {
                            mix_colors(
                                chunk[0..8].try_into().unwrap(),
                                chunk[8..16].try_into().unwrap(),
                            )
                        })
                        .map(|ti| {
                            let mut matrix_8_by_8: [[u8; 8]; 8] = Default::default();
                            for (y, yd) in ti.into_iter().enumerate() {
                                for x in (0..16u8).step_by(2) {
                                    let offset = 14 - x;
                                    let value = (yd >> offset) & 0b11;
                                    matrix_8_by_8[y][x as usize / 2] = value as u8;
                                }
                            }
                            matrix_8_by_8
                        })
                        .collect::<Vec<_>>();

                    if let Some(sender) = self.event_sender.as_ref() {
                        sender.send(Event::OnDebugFrame(data)).unwrap();
                    }
                }
            }
        } else {
            self.set_lcd_mode(LCDMode::OamScan);
        }
    }

    /// 持续10scanlines，结束后进入OamScan状态。
    fn step_vblank(&mut self) {
        if self.work_state.scanline_dots >= DOTS_PER_SCANLINE {
            self.move_to_next_scanline();

            if self.lcd.ly >= SCANLINES_PER_FRAME {
                self.lcd.ly = 0;
                self.set_lcd_mode(LCDMode::OamScan);
            }
        }
    }
}

impl<BUS: Memory + InterruptRequest> PPU<BUS> {
    /// https://gbdev.io/pandocs/Accessing_VRAM_and_OAM.html#accessing-vram-and-oam
    fn block_vram(&self, addr: u16) -> bool {
        (0x8000..=0x9FFF).contains(&addr) && self.lcd_mode() == LCDMode::RenderPixel
    }

    /// https://gbdev.io/pandocs/Accessing_VRAM_and_OAM.html#accessing-vram-and-oam
    fn block_oam(&self, addr: u16) -> bool {
        let lcd_mode = self.lcd_mode();
        (0xFE00..=0xFE9F).contains(&addr)
            && (lcd_mode == LCDMode::OamScan || lcd_mode == LCDMode::RenderPixel)
    }
}

impl<BUS: Memory + InterruptRequest> Memory for PPU<BUS> {
    fn write(&mut self, addr: u16, value: u8) {
        // if self.block_vram(addr) || self.block_oam(addr) {
        //     return;
        // }

        match addr {
            0x8000..=0x9FFF => {
                self.vram[addr as usize - 0x8000] = value;
            }
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00] = value,
            0xFF40 => self.lcd.lcdc = value,
            0xFF41 => {
                // https://gbdev.io/pandocs/Interrupt_Sources.html#int-48--stat-interrupt
                // https://gbdev.io/pandocs/STAT.html#ff41--stat-lcd-status
                // Since bit 0..=2 is readonly, writes on them are ignored.
                self.lcd.stat = (value & !(0b111)) | (self.lcd.stat & 0b111);
            }
            0xFF42 => self.lcd.scy = value,
            0xFF43 => self.lcd.scx = value,
            0xFF44 => {
                // readonly
            }
            0xFF45 => self.lcd.lyc = value,
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.lcd.wy = value,
            0xFF4B => self.lcd.wx = value,
            _ => unreachable!("Invalid PPU address: {:#X}", addr),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        // if self.block_oam(addr) || self.block_vram(addr) {
        //     return 0xFF;
        // }

        match addr {
            0x8000..=0x9FFF => self.vram[addr as usize - 0x8000],
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

#[cfg(test)]
use gb_shared::InterruptType;
#[cfg(test)]
use mockall::mock;

#[cfg(test)]
mock! {
    pub Bus {}

    impl Memory for Bus {
        fn write(&mut self, addr: u16, value: u8);
        fn read(&self, addr: u16) -> u8;
    }

    impl InterruptRequest for Bus {
        fn request(&mut self, interrupt_type: InterruptType);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_only_stat_bits() {
        let mut ppu = PPU::new(MockBus::new());
        ppu.lcd.stat = 0b0000_0101;

        ppu.write(0xFF41, 0b1000_0010);
        assert_eq!(ppu.read(0xFF41), 0b1000_0101);
    }

    #[test]
    fn read_only_ly() {
        let mut ppu = PPU::new(MockBus::new());
        ppu.lcd.ly = 0x12;

        ppu.write(0xFF44, 0x34);
        assert_eq!(ppu.read(0xFF44), 0x12);
    }
}
