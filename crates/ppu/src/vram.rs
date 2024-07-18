use gb_shared::{is_bit_set, MachineModel, Memory, Snapshot};

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct VideoRam {
    /// Tile data area(in size of 0x1800).
    /// There are total 384 tiles, each tile has 16 bytes.
    /// Thus, the size of this area is 6KB.
    /// - Block 0: 0x8000-0x87FF
    /// - Block 1: 0x8800-0x8FFF
    /// - Block 2: 0x9000-0x97FF
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
    /// - Tile map 0: 0x9800-0x9BFF
    /// - Tile map 1: 0x9C00-0x9FFF
    ram: Vec<u8>,
    /// a.k.a VBK. On CGB, there're two banks.
    bank_num: u8,
}

impl VideoRam {
    pub(crate) fn new(machine_model: MachineModel) -> Self {
        Self {
            ram: match machine_model {
                MachineModel::DMG => vec![0x00; 0x2000],
                MachineModel::CGB => vec![0x00; 0x4000],
            },
            bank_num: 0,
        }
    }
}

impl VideoRam {
    pub(crate) fn tile_data(&self, bank_num: u8, index: usize) -> &[u8; 16] {
        let offset = index * 16 + (bank_num as usize * 0x2000);

        self.ram[offset..(offset + 16)].as_ref().try_into().unwrap()
    }

    pub(crate) fn tile_index(&self, nth: usize) -> u8 {
        self.ram[nth + 0x1800]
    }

    pub(crate) fn tile_attrs(&self, nth: usize) -> Option<BackgroundAttrs> {
        self.ram.get(0x3800 + nth).map(|&value| BackgroundAttrs(value))
    }
}

impl Memory for VideoRam {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => {
                self.ram[self.bank_num as usize * 0x2000 + (addr as usize - 0x8000)] = value;
            }
            0xFF4F => self.bank_num = value & 0x01,
            _ => unreachable!("Invalid VRAM write at {:#X} {:#X}", addr, value),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.ram[self.bank_num as usize * 0x2000 + (addr as usize - 0x8000)],
            0xFF4F => self.bank_num | 0xFE,
            _ => unreachable!("Invalid VRAM read at {:#X}", addr),
        }
    }
}

pub(crate) type VideoRamSnapshot = VideoRam;

impl Snapshot for VideoRam {
    type Snapshot = VideoRamSnapshot;

    fn take_snapshot(&self) -> Self::Snapshot {
        VideoRamSnapshot { ram: self.ram.clone(), bank_num: self.bank_num }
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        self.ram = snapshot.ram;
        self.bank_num = snapshot.bank_num;
    }
}

#[derive(Clone, Copy)]
pub(crate) struct BackgroundAttrs(u8);

impl BackgroundAttrs {
    /// If set and BGW's color is 1-3, then BGW render over Object.
    pub(crate) fn bgw_over_object(&self) -> bool {
        is_bit_set!(self.0, 7)
    }

    /// If set, then the tile is render vertically flipped.
    pub(crate) fn y_flip(&self) -> bool {
        is_bit_set!(self.0, 6)
    }

    /// If set, then the tile is render horizontally flipped.
    pub(crate) fn x_flip(&self) -> bool {
        is_bit_set!(self.0, 5)
    }

    /// VRAM bank number. Return 0-1.
    pub(crate) fn bank_num(&self) -> u8 {
        is_bit_set!(self.0, 3) as u8
    }

    /// Palette number. Return 0-7.
    pub(crate) fn palette(&self) -> u8 {
        self.0 & 0x7
    }
}
