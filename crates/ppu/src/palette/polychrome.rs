use gb_shared::{is_bit_set, Memory, Snapshot};

use super::Palette;

pub(crate) struct Polychrome {
    bcps: u8,
    bcpd: [u8; 64],
    ocps: u8,
    ocpd: [u8; 64],
    colors: [[u32; 4]; 16],
}

fn read_color(data: &[u8; 64], palette_id: u8, color_id: u8) -> u32 {
    // 4 colors/palette, 2 bytes/color
    let base_addr = (palette_id * 4 + color_id) as usize * 2;

    let lo = data[base_addr];
    let hi = data[base_addr + 1];
    let rgb555 = ((hi as u16) << 8) | (lo as u16);

    let r = (rgb555 & 0x1F) as u32;
    let g = ((rgb555 >> 5) & 0x1F) as u32;
    let b = ((rgb555 >> 10) & 0x1F) as u32;

    let r = (r << 3) | (r >> 2);
    let g = (g << 3) | (g >> 2);
    let b = (b << 3) | (b >> 2);

    (r << 16) | (g << 8) | b
}

impl Polychrome {
    pub(crate) fn new() -> Self {
        Self { bcps: 0, bcpd: [0xFF; 64], ocps: 0, ocpd: [0xFF; 64], colors: [[0xFFFFFF; 4]; 16] }
    }
}

impl Palette for Polychrome {
    fn background_color(&self, palette_id: u8, color_id: u8) -> u32 {
        self.colors[palette_id as usize][color_id as usize]
    }

    fn object_color(&self, palette_id: u8, color_id: u8) -> u32 {
        self.colors[palette_id as usize + 8][color_id as usize]
    }

    fn colors(&self) -> &[[u32; 4]] {
        &self.colors
    }
}

impl Memory for Polychrome {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF47 => {}
            0xFF48 => {}
            0xFF49 => {}
            0xFF68 => self.bcps = value,
            0xFF69 => {
                let addr = self.bcps & 0x3F;
                self.bcpd[addr as usize] = value;

                // Update colors
                let palette_id = addr / 8;
                let color_id = (addr % 8) / 2;
                let color = read_color(&self.bcpd, palette_id, color_id);
                self.colors[palette_id as usize][color_id as usize] = color;

                if is_bit_set!(self.bcps, 7) {
                    self.bcps = ((addr + 1) & 0x3F) | 0x80;
                }
            }
            0xFF6A => self.ocps = value,
            0xFF6B => {
                let addr = self.ocps & 0x3F;
                self.ocpd[addr as usize] = value;

                // Update colors
                let palette_id = addr / 8;
                let color_id = (addr % 8) / 2;
                let color = read_color(&self.ocpd, palette_id, color_id);
                self.colors[palette_id as usize + 8][color_id as usize] = color;

                if is_bit_set!(self.ocps, 7) {
                    self.ocps = ((addr + 1) & 0x3F) | 0x80;
                }
            }
            _ => unreachable!("Invalid Polychrome write at {:#X} {:#X}", addr, value),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF47 => 0xFF,
            0xFF48 => 0xFF,
            0xFF49 => 0xFF,
            0xFF68 => self.bcps,
            0xFF69 => {
                let addr = (self.bcps & 0x3F) as usize;
                self.bcpd[addr]
            }
            0xFF6A => self.ocps,
            0xFF6B => {
                let addr = (self.ocps & 0x3F) as usize;
                self.ocpd[addr]
            }
            _ => unreachable!("Invalid Polychrome read at {:#X}", addr),
        }
    }
}

impl Snapshot for Polychrome {
    type Snapshot = Vec<u8>;

    fn take_snapshot(&self) -> Self::Snapshot {
        todo!()
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        todo!()
    }
}
