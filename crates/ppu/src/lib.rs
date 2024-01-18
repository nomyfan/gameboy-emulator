mod config;
mod lcd;
mod object;
mod tile;

use crate::config::{DOTS_PER_SCANLINE, RESOLUTION_X, RESOLUTION_Y, SCANLINES_PER_FRAME};
use crate::lcd::{LCDMode, LCD};
use crate::object::Object;
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
    /// Window line counter.
    /// It gets increased alongside with LY when window is visible.
    window_line: u8,
    /// Whether window is used in current scanline.
    /// Used for incrementing window_line.
    window_used: bool,
}

pub struct PPU<IRQ: InterruptRequest> {
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

    irq: IRQ,
    event_sender: Option<EventSender>,
}

impl<IRQ: InterruptRequest> PPU<IRQ> {
    pub fn new(irq: IRQ) -> Self {
        Self {
            vram: BoxedArray::default(),
            oam: BoxedArray::default(),
            lcd: LCD::default(),
            bgp: 0,
            obp0: 0,
            obp1: 0,
            work_state: PPUWorkState::default(),
            video_buffer: BoxedMatrix::default(),
            irq,
            event_sender: None,
        }
    }

    pub fn set_event_sender(&mut self, event_sender: EventSender) {
        self.event_sender.replace(event_sender);
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

    fn get_tile_map_value(&self, x: u8, y: u8, is_window: bool) -> u8 {
        let vram_addr = if is_window {
            self.lcd.window_tile_map_area()
        } else {
            self.lcd.background_tile_map_area()
        };
        let vram_addr = vram_addr + ((y as u16 / 8) * 32 + (x as u16) / 8);

        let vram_offset = vram_addr - 0x8000;
        self.vram[vram_offset as usize]
    }

    fn read_tile_data(&self, index: u8, for_object: bool) -> [u8; 16] {
        let index = if for_object || is_bit_set!(self.lcd.lcdc, 4) {
            index as usize
        } else {
            let index = index as i8; // Make it able to be negative.
            (256 + index as i16) as usize // It must be positive now before casting to usize.
        };
        let vram_offset = index * 16;
        self.vram[vram_offset..(vram_offset + 16)].try_into().unwrap()
    }

    fn move_to_next_scanline(&mut self) {
        self.work_state.scanline_dots = 0;
        self.work_state.scanline_x = 0;
        self.work_state.scanline_objects.clear();
        self.lcd.ly += 1;

        if self.work_state.window_used {
            self.work_state.window_line += 1;
        }
        self.work_state.window_used = false;

        if self.lcd.ly == self.lcd.lyc {
            self.lcd.stat = set_bits!(self.lcd.stat, 2);

            // LY == LYC stat interrupt
            if is_bit_set!(self.lcd.stat, 6) {
                self.irq.request_lcd_stat();
            }
        } else {
            self.lcd.stat = unset_bits!(self.lcd.stat, 2);
        }
    }

    fn is_window_visible(&self) -> bool {
        self.lcd.is_window_enabled() && self.lcd.wy <= 143 && self.lcd.wx <= 166
    }

    fn push_frame(&mut self) {
        if let Some(event_sender) = self.event_sender.as_ref() {
            event_sender.send(Event::OnFrame(self.video_buffer.clone())).unwrap();

            #[cfg(debug_assertions)]
            {
                let dbg_windows_flag = std::env::var("GB_DBG_WIN").unwrap_or_default();
                let dbg_windows_flag = dbg_windows_flag.split(',').collect::<Vec<_>>();

                // OAM frame.
                if dbg_windows_flag.contains(&"oam") {
                    let data = self
                        .vram
                        .chunks(16)
                        .map(|chunk| tile::mix_colors_16(chunk.try_into().unwrap()))
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

                    event_sender.send(Event::OnDebugFrame(0, data)).unwrap();
                }

                // Two tile map frames.
                if dbg_windows_flag.contains(&"map") {
                    let get_map = |indexes: &[u8], index_fn: fn(u8) -> u8| {
                        let mut map_data = Vec::with_capacity(32 * 32);
                        for index in indexes {
                            let index = index_fn(*index) as usize;

                            let tile = {
                                let tile: [u8; 16] =
                                    self.vram[(index * 16)..(index * 16 + 16)].try_into().unwrap();

                                let data = tile::mix_colors_16(&tile);

                                let mut matrix_8_by_8: [[u8; 8]; 8] = Default::default();
                                for (y, yd) in data.into_iter().enumerate() {
                                    for x in (0..16u8).step_by(2) {
                                        let offset = 14 - x;
                                        let value = (yd >> offset) & 0b11;
                                        matrix_8_by_8[y][x as usize / 2] = value as u8;
                                    }
                                }
                                matrix_8_by_8
                            };

                            map_data.push(tile);
                        }

                        map_data
                    };
                    let index_fn = if is_bit_set!(self.lcd.lcdc, 4) {
                        |index: u8| index
                    } else {
                        |index: u8| (index as i8 as i16 + 256) as u8
                    };
                    let map1 = get_map(&self.vram[0x1800..(0x1800 + 1024)], index_fn);
                    let map2 = get_map(&self.vram[0x1C00..(0x1C00 + 1024)], index_fn);
                    debug_assert_eq!(map1.len(), 1024);
                    debug_assert_eq!(map2.len(), 1024);

                    event_sender.send(Event::OnDebugFrame(1, map1)).unwrap();
                    event_sender.send(Event::OnDebugFrame(2, map2)).unwrap();
                }
            }
        }
    }

    fn power(&mut self, on: bool) {
        if on {
            log::debug!("ppu power on");
            self.work_state = Default::default();
            self.lcd.ly = 0;
            self.set_lcd_mode(LCDMode::OamScan);
        } else {
            log::debug!("ppu power off");
            self.video_buffer.iter_mut().for_each(|row| {
                row.iter_mut().for_each(|color_palette| {
                    *color_palette = 0;
                })
            });
            self.push_frame();
        }
    }

    pub fn step(&mut self) {
        if !self.lcd.is_lcd_enabled() {
            return;
        }

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
        if self.work_state.scanline_dots == 1 && is_bit_set!(self.lcd.stat, 5) {
            // Mode 2(OAM scan) stat interrupt
            self.irq.request_lcd_stat();
        }

        // https://gbdev.io/pandocs/OAM.html#:~:text=up%20to%2010%20objects%20to%20be%20drawn%20on%20that%20line
        if self.work_state.scanline_dots % 2 == 0 && self.work_state.scanline_objects.len() < 10 {
            let obj_size = self.lcd.object_size();
            let object_index = (self.work_state.scanline_dots as usize - 1) / 2;
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
        }

        if self.work_state.scanline_dots == 80 {
            // https://gbdev.io/pandocs/OAM.html#drawing-priority
            //
            // For Non-CGB, the smaller X, the higher priority.
            // If the X is same, object located first has higher priority.
            //
            // For CGB, the priority is determined by the location in OAM.
            // The earlier the object, the higher its priority.
            //
            // It's notable that `sort_by` is stable.
            self.work_state.scanline_objects.sort_by(|a, b| a.x.cmp(&b.x));
            self.set_lcd_mode(LCDMode::RenderPixel);
        }
    }

    /// 持续172-289dots，加载Win/BG的tile，和object做像素合成。
    /// 结束后进入HBlank状态。
    fn step_render_pixel(&mut self) {
        // TODO: penalty for canceling?
        // https://gbdev.io/pandocs/pixel_fifo.html#object-fetch-canceling
        // https://gbdev.io/pandocs/Rendering.html#mode-3-length

        // Extra 12 dots are needed for fetching two tiles at the beginning of mode 3.
        // https://gbdev.io/pandocs/Rendering.html#:~:text=the%2012%20extra%20cycles%20come%20from%20two%20tile%20fetches%20at%20the%20beginning%20of%20mode%203
        if self.work_state.scanline_dots <= (80 + 12) {
            return;
        }

        let mut color_id = 0;
        let mut color_palette = self.bgp & 0b11;

        if self.lcd.is_bgw_enabled() {
            let (mut index, mut ty, mut tx) = {
                let map_y = self.lcd.ly.wrapping_add(self.lcd.scy);
                let map_x = self.work_state.scanline_x.wrapping_add(self.lcd.scx);

                let index = self.get_tile_map_value(map_x, map_y, false);
                let ty = map_y % 8;
                let tx = map_x % 8;

                (index, ty, tx)
            };

            if self.is_window_visible()
                && ((self.lcd.wy as u16)..(self.lcd.wy as u16 + RESOLUTION_Y as u16))
                    .contains(&(self.lcd.ly as u16))
                && ((self.lcd.wx as u16)..(self.lcd.wx as u16 + RESOLUTION_X as u16))
                    .contains(&(self.work_state.scanline_x as u16 + 7))
            {
                self.work_state.window_used = true;
                let y = self.work_state.window_line;
                let x = self.work_state.scanline_x + 7 - self.lcd.wx;

                index = self.get_tile_map_value(x, y, true);
                ty = y % 8;
                tx = x % 8;
            }

            let tile_data = self.read_tile_data(index, false);
            let tile_data = tile::mix_colors_16(&tile_data);

            color_id = tile::get_color_id(&tile_data, tx, ty);
            color_palette = (self.bgp >> (color_id * 2)) & 0b11;
        }

        if self.lcd.is_object_enabled() {
            let obj_size = self.lcd.object_size();
            let objects = self
                .work_state
                .scanline_objects
                .iter()
                .filter(|object| {
                    // overlap
                    let sx = self.work_state.scanline_x + 8;
                    sx >= object.x && sx < object.x + 8
                })
                .map(|object| {
                    let object = *object;

                    let mut ty = (self.lcd.ly + 16) - object.y;
                    let tx = (self.work_state.scanline_x + 8) - object.x;

                    if obj_size == 16 && object.attrs.y_flip() {
                        ty = 15 - ty;
                    }

                    let index = if obj_size == 16 {
                        if ty >= 8 {
                            // bottom tile
                            object.tile_index | 0x01
                        } else {
                            // top tile
                            object.tile_index & 0xFE
                        }
                    } else {
                        object.tile_index
                    };

                    ty %= 8;

                    let tile_data = self.read_tile_data(index, true);
                    let mut tile_data = tile::mix_colors_16(&tile_data);
                    tile::apply_object_attrs(&mut tile_data, &object.attrs);
                    let color_id = tile::get_color_id(&tile_data, tx, ty);

                    (color_id, object)
                })
                .collect::<Vec<_>>();
            let opaque_object = objects.into_iter().find(|(obj_color_id, _)| obj_color_id != &0);
            if let Some((obj_color_id, object)) = opaque_object {
                // Priority definition(the object below is opaque)
                // 1. If BGW' color ID is 0, then render the object.
                // 2. If LCDC.0 is 0, then render the object.
                // 3. If OAM attributes.7 is 0, then render the object.
                // 4. Otherwise, render the BGW.
                if self.lcd.ly >= 88 && self.lcd.ly < 114 {
                    log::debug!(
                        "Render object at ({}, {}), object len {}, object size {}",
                        self.work_state.scanline_x,
                        self.lcd.ly,
                        self.work_state.scanline_objects.len(),
                        obj_size,
                    );
                }
                if color_id == 0 || !object.attrs.bgw_over_object() {
                    let obp = if object.attrs.dmg_palette() == 0 { self.obp0 } else { self.obp1 };
                    let offset = obj_color_id * 2;
                    color_palette = (obp >> offset) & 0b11;
                }
            }
        }

        let viewport_x = self.work_state.scanline_x as usize;
        let viewport_y = self.lcd.ly as usize;
        self.video_buffer[viewport_y][viewport_x] = color_palette;
        self.work_state.scanline_x += 1;

        // Pixels in current scanline are all rendered.
        if self.work_state.scanline_x >= RESOLUTION_X as u8 {
            self.set_lcd_mode(LCDMode::HBlank);

            // Mode 0(HBlank) stat interrupt
            if is_bit_set!(self.lcd.stat, 3) {
                self.irq.request_lcd_stat();
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
            self.irq.request_vblank();
            // Mode 1(VBlank) stat interrupt
            if is_bit_set!(self.lcd.stat, 4) {
                self.irq.request_lcd_stat();
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
                self.work_state.window_line = 0;
                self.set_lcd_mode(LCDMode::OamScan);

                // https://gbdev.io/pandocs/Rendering.html#obj-penalty-algorithm:~:text=one%20frame%20takes%20~-,16.74,-ms%20instead%20of
                // (456 * 154) * (1/(2**22)) * 1000 = 16.74ms
                // Notify that a frame is rendered.
                self.push_frame();
            }
        }
    }
}

impl<IRQ: InterruptRequest> Memory for PPU<IRQ> {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => {
                self.vram[addr as usize - 0x8000] = value;
            }
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00] = value,
            0xFF40 => {
                let old_enabled = self.lcd.is_lcd_enabled();
                let old_object_size = self.lcd.object_size();
                self.lcd.lcdc = value;
                let new_enabled = self.lcd.is_lcd_enabled();
                let new_object_size = self.lcd.object_size();

                if old_enabled != new_enabled {
                    // https://gbdev.io/pandocs/LCDC.html#lcdc7--lcd-enable:~:text=be%20performed%0Aduring-,vblank%20only%2C,-disabling%20the%20display
                    assert_eq!(self.lcd_mode(), LCDMode::VBlank);
                    self.power(new_enabled);
                }

                if old_object_size != new_object_size && self.work_state.scanline_objects.len() < 10
                {
                    log::debug!(
                        "Object size changes from {} to {} at scanline dots {}, ly {}. Scanned object count {}",
                        old_object_size, new_object_size,
                        self.work_state.scanline_dots,
                        self.lcd.ly,
                        self.work_state.scanline_objects.len()
                    );
                }
            }
            0xFF41 => {
                // https://gbdev.io/pandocs/Interrupt_Sources.html#int-48--stat-interrupt
                // https://gbdev.io/pandocs/STAT.html#ff41--stat-lcd-status
                // Since bit 0..=2 is readonly, writes on them are ignored.
                self.lcd.stat = 0x80 | (value & !(0b111)) | (self.lcd.stat & 0b111);
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
