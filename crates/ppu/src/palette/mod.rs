mod compatibility_palettes;

use gb_shared::{is_bit_set, MachineModel, Memory, Snapshot};

use self::compatibility_palettes::find_palette;

const FALLBACK_COLORS: &[u32; 4] = &[0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000];

pub(crate) struct Palette {
    machine_model: MachineModel,
    bgp: u8,
    obp0: u8,
    obp1: u8,
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

impl Palette {
    pub(crate) fn new(machine_model: MachineModel, palette_id: u16) -> Self {
        let mut colors = [[0xFFFFFF; 4]; 16];
        if machine_model == MachineModel::DMG {
            match find_palette(palette_id) {
                Some(p) => {
                    colors[0] = [p[0], p[1], p[2], p[3]];
                    colors[1] = [p[4], p[5], p[6], p[7]];
                    colors[2] = [p[8], p[9], p[10], p[11]];
                }
                None => {
                    colors[0] = *FALLBACK_COLORS;
                    colors[1] = *FALLBACK_COLORS;
                    colors[2] = *FALLBACK_COLORS;
                }
            }
        }

        Self {
            machine_model,
            bcps: 0,
            bcpd: [0xFF; 64],
            ocps: 0,
            ocpd: [0xFF; 64],
            colors,
            bgp: 0xFC,
            obp0: 0,
            obp1: 0,
        }
    }
}

impl Palette {
    pub(crate) fn background_color(&self, palette_id: u8, color_id: u8) -> u32 {
        match self.machine_model {
            MachineModel::DMG => self.colors[0][((self.bgp >> (color_id * 2)) & 0b11) as usize],
            MachineModel::CGB => self.colors[palette_id as usize][color_id as usize],
        }
    }

    pub(crate) fn object_color(&self, palette_id: u8, color_id: u8) -> u32 {
        match self.machine_model {
            MachineModel::DMG => {
                let obp = if palette_id == 0 { self.obp0 } else { self.obp1 };
                self.colors[palette_id as usize + 1][((obp >> (color_id * 2)) & 0b11) as usize]
            }
            MachineModel::CGB => self.colors[palette_id as usize + 8][color_id as usize],
        }
    }

    #[cfg(feature = "debug_frame")]
    pub(crate) fn colors(&self) -> &[[u32; 4]] {
        match self.machine_model {
            MachineModel::DMG => &self.colors[..3],
            MachineModel::CGB => &self.colors,
        }
    }

    fn update_color(&mut self, bg: bool, cps: u8, value: u8) -> u8 {
        let addr = cps & 0x3F;
        let palette_id = addr / 8;
        let color_id = (addr % 8) / 2;
        let color_index = palette_id as usize + if bg { 0 } else { 8 };

        let cpd = if bg { &mut self.bcpd } else { &mut self.ocpd };
        cpd[addr as usize] = value;
        let color = read_color(cpd, palette_id, color_id);
        self.colors[color_index][color_id as usize] = color;

        if is_bit_set!(cps, 7) {
            ((addr + 1) & 0x3F) | 0x80
        } else {
            addr
        }
    }
}

impl Memory for Palette {
    fn write(&mut self, addr: u16, value: u8) {
        let is_cgb = self.machine_model == MachineModel::CGB;

        match addr {
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF68 if is_cgb => self.bcps = value,
            0xFF69 if is_cgb => self.bcps = self.update_color(true, self.bcps, value),
            0xFF6A if is_cgb => self.ocps = value,
            0xFF6B if is_cgb => self.ocps = self.update_color(false, self.ocps, value),
            _ => unreachable!("Invalid Palette write at {:#X} {:#X}", addr, value),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        let is_cgb = self.machine_model == MachineModel::CGB;

        match addr {
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF68 if is_cgb => self.bcps,
            0xFF69 if is_cgb => self.bcpd[(self.bcps & 0x3F) as usize],
            0xFF6A if is_cgb => self.ocps,
            0xFF6B if is_cgb => self.ocpd[(self.ocps & 0x3F) as usize],
            _ => unreachable!("Invalid Palette read at {:#X}", addr),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct PaletteSnapshot {
    bgp: u8,
    obp0: u8,
    obp1: u8,
    bcps: u8,
    bcpd: Vec<u8>,
    ocps: u8,
    ocpd: Vec<u8>,
    colors: [[u32; 4]; 16],
}

impl Snapshot for Palette {
    type Snapshot = PaletteSnapshot;

    fn take_snapshot(&self) -> Self::Snapshot {
        Self::Snapshot {
            bgp: self.bgp,
            obp0: self.obp0,
            obp1: self.obp1,
            bcps: self.bcps,
            bcpd: self.bcpd.to_vec(),
            ocps: self.ocps,
            ocpd: self.ocpd.to_vec(),
            colors: self.colors,
        }
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        self.bgp = snapshot.bgp;
        self.obp0 = snapshot.obp0;
        self.obp1 = snapshot.obp1;
        self.bcps = snapshot.bcps;
        self.bcpd = snapshot.bcpd.as_slice().try_into().unwrap();
        self.ocps = snapshot.ocps;
        self.ocpd = snapshot.ocpd.as_slice().try_into().unwrap();
        self.colors = snapshot.colors;
    }
}
