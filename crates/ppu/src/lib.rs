mod config;
mod lcd;
mod object;
mod palette;
mod tile;
mod vram;

use crate::config::{DOTS_PER_SCANLINE, RESOLUTION_X, RESOLUTION_Y, SCANLINES_PER_FRAME};
use crate::lcd::{LCDMode, LCD};
use crate::object::Object;
use gb_shared::boxed::BoxedArray;
use gb_shared::{
    is_bit_set, set_bits, unset_bits, Interrupt, InterruptRequest, MachineModel, Memory, Snapshot,
};
use object::ObjectSnapshot;
use palette::{Monochrome, Polychrome};
use vram::{BackgroundAttrs, VideoRam, VideoRamSnapshot};

pub type VideoFrame = BoxedArray<u8, 69120>; // 160 * 144 * 3

pub type FrameOutHandle = dyn FnMut(&VideoFrame, &[u8]); // 256 * 256 * 2 * 3, 393216

#[derive(Debug, Default)]
pub(crate) struct PpuWorkState {
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

trait GraphicPalette: Memory + Snapshot + crate::palette::Palette {}
impl<T: Memory + Snapshot + crate::palette::Palette> GraphicPalette for T {}

pub struct Ppu {
    vram: VideoRam,
    /// \[0xFE00, 0xFE9F]
    /// OAM(Object Attribute Memory) is used to store objects.
    /// There're up to 40 objects. Each entry consists of 4 bytes.
    /// - Byte 0: Y position.
    /// - Byte 1: X position.
    /// - Byte 2: tile index.
    /// - Byte 3: attributes.
    oam: BoxedArray<u8, 160>,
    lcd: LCD,
    palette: Box<dyn GraphicPalette<Snapshot = Vec<u8>>>,
    /// PPU work state.
    work_state: PpuWorkState,
    /// Storing palettes.
    video_buffer: VideoFrame,
    dbg_video_buffer: Vec<u8>,

    irq: Interrupt,
    machine_model: MachineModel,
    frame_out_handle: Option<Box<FrameOutHandle>>,
}

impl Default for Ppu {
    fn default() -> Self {
        let mut video_buffer: VideoFrame = Default::default();
        video_buffer.fill(0xFF);

        let machine_model = MachineModel::DMG;

        Self {
            vram: VideoRam::new(machine_model),
            oam: Default::default(),
            lcd: Default::default(),
            work_state: Default::default(),
            video_buffer,
            irq: Default::default(),
            machine_model,
            frame_out_handle: None,
            palette: Box::new(Monochrome::new(0)),
            dbg_video_buffer: vec![0xFF; (256 * 256 * 3 * 2) + (3 * 12)],
        }
    }
}

impl Ppu {
    pub fn new(machine_model: MachineModel, compatibility_palette_id: u16) -> Self {
        Self {
            palette: match machine_model {
                MachineModel::DMG => Box::new(Monochrome::new(compatibility_palette_id)),
                MachineModel::CGB => Box::new(Polychrome::new()),
            },
            dbg_video_buffer: match machine_model {
                MachineModel::DMG => vec![0xFF; (256 * 256 * 3 * 2) + (3 * 12)],
                MachineModel::CGB => vec![0xFF; (256 * 256 * 3 * 2) + (16 * 12)],
            },
            vram: VideoRam::new(machine_model),
            machine_model,
            ..Self::default()
        }
    }

    pub fn set_frame_out_handle(&mut self, handle: Option<Box<FrameOutHandle>>) {
        self.frame_out_handle = handle;
    }

    pub fn take_irq(&mut self) -> u8 {
        self.irq.take()
    }

    pub fn ly(&self) -> u8 {
        self.lcd.ly
    }

    pub fn lcd_mode(&self) -> LCDMode {
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

    fn get_bgw_tile(&self, x: u8, y: u8, is_window: bool) -> (u8, Option<BackgroundAttrs>) {
        let addr_base = if is_window {
            self.lcd.window_tile_map_area()
        } else {
            self.lcd.background_tile_map_area()
        };
        let nth = (y as usize / 8) * 32 + (x as usize) / 8;
        self.vram.bgw_tile_info(addr_base as usize - 0x9800 + nth)
    }

    // fn get_tile_map_value(&self, x: u8, y: u8, is_window: bool) -> u8 {
    //     let vram_addr = if is_window {
    //         self.lcd.window_tile_map_area()
    //     } else {
    //         self.lcd.background_tile_map_area()
    //     };
    //     let vram_addr = vram_addr + ((y as u16 / 8) * 32 + (x as u16) / 8);

    //     let vram_offset = vram_addr - 0x8000;
    //     self.vram[vram_offset as usize]
    // }

    fn read_tile_data(&self, bank_num: u8, index: u8, for_object: bool) -> &[u8; 16] {
        let index = if for_object || is_bit_set!(self.lcd.lcdc, 4) {
            index as usize
        } else {
            let index = index as i8; // Make it able to be negative.
            (256 + index as i16) as usize // It must be positive now before casting to usize.
        };
        self.vram.tile(bank_num, index)
    }

    fn move_to_next_scanline(&mut self) {
        self.work_state.scanline_dots = 0;
        self.work_state.scanline_x = 0;
        self.work_state.scanline_objects.clear();
        self.lcd.ly = (self.lcd.ly + 1) % SCANLINES_PER_FRAME;

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
        self.lcd.window_enabled() && self.lcd.wy <= 143 && self.lcd.wx <= 166
    }

    fn push_frame(&mut self) {
        if self.frame_out_handle.is_some() {
            for (i, vram_offset) in [0, 0x400].iter().enumerate() {
                for y in 0..256 {
                    for x in 0..256 {
                        let nth = y / 8 * 32 + (x / 8);
                        let tile_index = self.vram.tile_index(vram_offset + nth);

                        let attrs = self.vram.bgw_tile_attrs(vram_offset + nth);
                        let bank_num = attrs.map(|x| x.bank_number()).unwrap_or_default();
                        let palette_id = attrs.map(|x| x.palette()).unwrap_or_default();

                        let tile_data = self.read_tile_data(bank_num, tile_index, false);
                        let color_id = tile::get_color_id(
                            tile_data,
                            (x as u8) % 8,
                            (y as u8) % 8,
                            false,
                            false,
                        );
                        let color = self.palette.background_color(palette_id, color_id);
                        let base_addr = (y as usize * 256 + x as usize) * 3 + (i * 256 * 256 * 3);
                        self.dbg_video_buffer[base_addr] = (color >> 16) as u8;
                        self.dbg_video_buffer[base_addr + 1] = (color >> 8) as u8;
                        self.dbg_video_buffer[base_addr + 2] = color as u8;
                    }
                }
            }
            // Palette colors
            for (i, color) in self.palette.colors().iter().enumerate() {
                const START_ADDR: usize = 256 * 256 * 3 * 2;
                let base_addr = START_ADDR + i * 12;
                self.dbg_video_buffer[base_addr] = (color[0] >> 16) as u8;
                self.dbg_video_buffer[base_addr + 1] = (color[0] >> 8) as u8;
                self.dbg_video_buffer[base_addr + 2] = color[0] as u8;
                self.dbg_video_buffer[base_addr + 3] = (color[1] >> 16) as u8;
                self.dbg_video_buffer[base_addr + 4] = (color[1] >> 8) as u8;
                self.dbg_video_buffer[base_addr + 5] = color[1] as u8;
                self.dbg_video_buffer[base_addr + 6] = (color[2] >> 16) as u8;
                self.dbg_video_buffer[base_addr + 7] = (color[2] >> 8) as u8;
                self.dbg_video_buffer[base_addr + 8] = color[2] as u8;
                self.dbg_video_buffer[base_addr + 9] = (color[3] >> 16) as u8;
                self.dbg_video_buffer[base_addr + 10] = (color[3] >> 8) as u8;
                self.dbg_video_buffer[base_addr + 11] = color[3] as u8;
            }
        }

        if let Some(handle) = self.frame_out_handle.as_mut() {
            handle(&self.video_buffer, &self.dbg_video_buffer);
        }
    }

    fn power(&mut self, on: bool) {
        if on {
            self.set_lcd_mode(LCDMode::OamScan);
        } else {
            // https://www.reddit.com/r/Gameboy/comments/a1c8h0/what_happens_when_a_gameboy_screen_is_disabled/
            self.work_state = Default::default();
            self.lcd.ly = 0;
            self.set_lcd_mode(LCDMode::HBlank);
            self.video_buffer.fill(0xFF);
            self.push_frame();
        }
    }

    pub fn step(&mut self) {
        if !self.lcd.lcd_enabled() {
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
            let object_y = object.y as u16;
            if (object_y..(object_y + obj_size as u16)).contains(&(self.lcd.ly as u16 + 16)) {
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
            if self.machine_model == MachineModel::DMG {
                // It's notable that `sort_by` is stable.
                self.work_state.scanline_objects.sort_by(|a, b| a.x.cmp(&b.x));
            }
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

        let mut bgw_color_id = 0;
        let mut color = 0xFFFFFF;
        let mut bgw_attrs: Option<BackgroundAttrs> = None;

        let bg_enabled = match self.machine_model {
            MachineModel::DMG => self.lcd.lcdc0(),
            MachineModel::CGB => true,
        };

        if bg_enabled {
            let (mut tile_index, mut ty, mut tx) = {
                let map_y = self.lcd.ly.wrapping_add(self.lcd.scy);
                let map_x = self.work_state.scanline_x.wrapping_add(self.lcd.scx);

                let (index, attrs) = self.get_bgw_tile(map_x, map_y, false);
                let ty = map_y % 8;
                let tx = map_x % 8;

                bgw_attrs = attrs;
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

                let (index, attrs) = self.get_bgw_tile(x, y, true);

                ty = y % 8;
                tx = x % 8;
                tile_index = index;
                bgw_attrs = attrs;
            }

            match self.machine_model {
                MachineModel::DMG => {
                    let tile_data = self.read_tile_data(0, tile_index, false);
                    bgw_color_id = tile::get_color_id(tile_data, tx, ty, false, false);
                    color = self.palette.background_color(0, bgw_color_id);
                }
                MachineModel::CGB => {
                    let attrs = bgw_attrs.unwrap();
                    let tile_data = self.read_tile_data(attrs.bank_number(), tile_index, false);
                    bgw_color_id =
                        tile::get_color_id(tile_data, tx, ty, attrs.x_flip(), attrs.y_flip());
                    color = self.palette.background_color(attrs.palette(), bgw_color_id);
                }
            }
        }

        let object_enabled = match self.machine_model {
            MachineModel::DMG => self.lcd.object_enabled(),
            MachineModel::CGB => !self.lcd.lcdc0() || self.lcd.object_enabled(),
        };

        if object_enabled {
            let obj_size = self.lcd.object_size();

            for object in &self.work_state.scanline_objects {
                let sx = self.work_state.scanline_x + 8;
                if sx < object.x || sx >= object.x + 8 {
                    continue;
                }

                let ty = (self.lcd.ly + 16) - object.y;
                let tx = (self.work_state.scanline_x + 8) - object.x;

                let index = if obj_size == 16 {
                    let mut top = object.tile_index & 0xFE;
                    let mut bottom = object.tile_index | 0x01;

                    if object.attrs.y_flip() {
                        std::mem::swap(&mut top, &mut bottom);
                    }

                    if ty < 8 {
                        top
                    } else {
                        bottom
                    }
                } else {
                    object.tile_index
                };

                let bank_number = match self.machine_model {
                    MachineModel::DMG => 0,
                    MachineModel::CGB => object.attrs.bank_num(),
                };
                let tile_data = self.read_tile_data(bank_number, index, true);
                let object_color_id = tile::get_color_id(
                    tile_data,
                    tx,
                    ty % 8,
                    object.attrs.x_flip(),
                    object.attrs.y_flip(),
                );
                if object_color_id != 0 {
                    let (render_object, palette_id) = match self.machine_model {
                        MachineModel::DMG => (
                            bgw_color_id == 0 || !object.attrs.bgw_over_object(),
                            object.attrs.dmg_palette(),
                        ),
                        MachineModel::CGB => (
                            bgw_color_id == 0
                                || !self.lcd.lcdc0()
                                || (!object.attrs.bgw_over_object()
                                    && !bgw_attrs.unwrap().bgw_over_object()),
                            object.attrs.cgb_palette(),
                        ),
                    };
                    if render_object {
                        color = self.palette.object_color(palette_id, object_color_id);
                    }

                    break;
                }
            }
        }

        let viewport_x = self.work_state.scanline_x as usize;
        let viewport_y = self.lcd.ly as usize;

        let buf_addr = (viewport_y * RESOLUTION_X + viewport_x) * 3;
        self.video_buffer[buf_addr] = (color >> 16) as u8;
        self.video_buffer[buf_addr + 1] = (color >> 8) as u8;
        self.video_buffer[buf_addr + 2] = color as u8;
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

            if self.lcd.ly == 0 {
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

impl Memory for Ppu {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => self.vram.write(addr, value),
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00] = value,
            0xFF40 => {
                let old_enabled = self.lcd.lcd_enabled();
                let old_object_size = self.lcd.object_size();
                self.lcd.lcdc = value;
                let new_enabled = self.lcd.lcd_enabled();
                let new_object_size = self.lcd.object_size();

                if old_enabled != new_enabled {
                    // https://gbdev.io/pandocs/LCDC.html#lcdc7--lcd-enable:~:text=be%20performed%0Aduring-,vblank%20only%2C,-disabling%20the%20display
                    if old_enabled {
                        assert_eq!(self.lcd_mode(), LCDMode::VBlank);
                    }
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
            0xFF47..=0xFF49 => self.palette.write(addr, value),
            0xFF4A => self.lcd.wy = value,
            0xFF4B => self.lcd.wx = value,
            0xFF4F => self.vram.write(addr, value),
            0xFF68..=0xFF6B => self.palette.write(addr, value),
            _ => unreachable!("Invalid PPU address: {:#X}", addr),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.vram.read(addr),
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00],
            0xFF40 => self.lcd.lcdc,
            0xFF41 => self.lcd.stat,
            0xFF42 => self.lcd.scy,
            0xFF43 => self.lcd.scx,
            0xFF44 => self.lcd.ly,
            0xFF45 => self.lcd.lyc,
            0xFF47..=0xFF49 => self.palette.read(addr),
            0xFF4A => self.lcd.wy,
            0xFF4B => self.lcd.wx,
            0xFF4F => self.vram.read(addr),
            0xFF68..=0xFF6B => self.palette.read(addr),
            _ => unreachable!("Invalid PPU address: {:#X}", addr),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PpuSnapshot {
    vram: VideoRamSnapshot, // 0x2000
    oam: Vec<u8>,           // 0xA0
    lcd: LCD,
    palette: Vec<u8>,
    //#region Work state
    scanline_x: u8,
    scanline_dots: u16,
    scanline_objects: Vec<ObjectSnapshot>,
    window_line: u8,
    window_used: bool,
    //#endregion
    irq: u8,
    video_buffer: Vec<u8>, // 160 * 144
}

impl Snapshot for Ppu {
    type Snapshot = PpuSnapshot;

    fn take_snapshot(&self) -> Self::Snapshot {
        PpuSnapshot {
            vram: self.vram.take_snapshot(),
            oam: self.oam.to_vec(),
            lcd: self.lcd,
            palette: self.palette.take_snapshot(),
            scanline_x: self.work_state.scanline_x,
            scanline_dots: self.work_state.scanline_dots,
            scanline_objects: self
                .work_state
                .scanline_objects
                .iter()
                .map(|o| o.take_snapshot())
                .collect(),
            window_line: self.work_state.window_line,
            window_used: self.work_state.window_used,
            irq: self.irq.0,
            video_buffer: self.video_buffer.to_vec(),
        }
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        self.vram.restore_snapshot(snapshot.vram);
        self.oam = BoxedArray::try_from_vec(snapshot.oam).unwrap();
        self.lcd = snapshot.lcd;
        self.palette.restore_snapshot(snapshot.palette);
        self.work_state.scanline_x = snapshot.scanline_x;
        self.work_state.scanline_dots = snapshot.scanline_dots;
        self.work_state.scanline_objects = snapshot
            .scanline_objects
            .into_iter()
            .map(|o| {
                let mut object = Object::default();
                object.restore_snapshot(o);
                object
            })
            .collect();
        self.work_state.window_line = snapshot.window_line;
        self.work_state.window_used = snapshot.window_used;
        self.irq.0 = snapshot.irq;
        self.video_buffer = BoxedArray::try_from_vec(snapshot.video_buffer).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_only_stat_bits() {
        let mut ppu = Ppu::default();
        ppu.lcd.stat = 0b0000_0101;

        ppu.write(0xFF41, 0b1000_0010);
        assert_eq!(ppu.read(0xFF41), 0b1000_0101);
    }

    #[test]
    fn read_only_ly() {
        let mut ppu = Ppu::default();
        ppu.lcd.ly = 0x12;

        ppu.write(0xFF44, 0x34);
        assert_eq!(ppu.read(0xFF44), 0x12);
    }
}
