use gb_shared::{Memory, Snapshot};

use super::{compatibility_palettes::find_palette, Palette};

const COLORS: &[u32; 4] = &[0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000];

pub(crate) struct Monochrome {
    bgp: u8,
    obp0: u8,
    obp1: u8,
    bgp_palette: [u32; 4],
    obp0_palette: [u32; 4],
    obp1_palette: [u32; 4],
    colors: [[u32; 4]; 3],
}

impl Monochrome {
    pub(crate) fn new(palette_id: u16) -> Self {
        let [bgp_palette, obp0_palette, obp1_palette]: [[u32; 4]; 3] =
            match find_palette(palette_id) {
                Some(p) => {
                    [[p[0], p[1], p[2], p[3]], [p[4], p[5], p[6], p[7]], [p[8], p[9], p[10], p[11]]]
                }
                None => [*COLORS, *COLORS, *COLORS],
            };

        let colors = [bgp_palette, obp0_palette, obp1_palette];

        Self { bgp: 0xFC, obp0: 0, obp1: 0, bgp_palette, obp0_palette, obp1_palette, colors }
    }
}

impl Palette for Monochrome {
    fn background_color(&self, _palette_id: u8, color_id: u8) -> u32 {
        self.bgp_palette[((self.bgp >> (color_id * 2)) & 0b11) as usize]
    }

    fn object_color(&self, palette_id: u8, color_id: u8) -> u32 {
        if palette_id == 0 {
            let obp = self.obp0;
            self.obp0_palette[((obp >> (color_id * 2)) & 0b11) as usize]
        } else {
            let obp = self.obp1;
            self.obp1_palette[((obp >> (color_id * 2)) & 0b11) as usize]
        }
    }

    fn colors(&self) -> &[[u32; 4]] {
        &self.colors
    }
}

impl Memory for Monochrome {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            _ => unreachable!("Invalid address: {:#X}", addr),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            _ => unreachable!("Invalid address: {:#X}", addr),
        }
    }
}

impl Snapshot for Monochrome {
    type Snapshot = Vec<u8>;

    fn take_snapshot(&self) -> Self::Snapshot {
        todo!()
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        todo!()
    }
}
