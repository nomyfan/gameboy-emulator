use gb_shared::{MachineModel, Memory, Snapshot};

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct WorkRam {
    /// In DMG, there're 2 banks in used.
    /// In CGB, there're 8 banks in used, and 1-7 is switchable.
    ram: Vec<u8>,
    /// a.k.a SVBK. 1-7, 0 will be mapped to 1.
    bank_num: u8,
}

impl WorkRam {
    pub(crate) fn new(machine_model: MachineModel) -> Self {
        Self {
            ram: match machine_model {
                MachineModel::DMG => vec![0; 0x4000],
                MachineModel::CGB => vec![0; 0x8000],
            },
            bank_num: 0,
        }
    }
}

impl WorkRam {
    fn bank_num(&self) -> u8 {
        self.bank_num.max(1)
    }
}

impl Memory for WorkRam {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xC000..=0xCFFF => self.ram[addr as usize - 0xC000] = value,
            0xD000..=0xDFFF => {
                let addr = addr as usize - 0xD000 + (0x1000 * self.bank_num() as usize);
                self.ram[addr] = value;
            }
            0xFF70 => self.bank_num = value & 0x07,
            _ => unreachable!("Invalid WRAM write at {:#X} {:#X}", addr, value),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xC000..=0xCFFF => self.ram[addr as usize - 0xC000],
            0xD000..=0xDFFF => {
                let addr = addr as usize - 0xD000 + (0x1000 * self.bank_num() as usize);
                self.ram[addr]
            }
            0xFF70 => self.bank_num | 0xF8,
            _ => unreachable!("Invalid WRAM read at {:#X}", addr),
        }
    }
}

pub(crate) type WorkRamSnapshot = WorkRam;

impl Snapshot for WorkRam {
    type Snapshot = WorkRamSnapshot;

    fn take_snapshot(&self) -> Self::Snapshot {
        Self::Snapshot { ram: self.ram.clone(), bank_num: self.bank_num }
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        self.ram = snapshot.ram;
        self.bank_num = snapshot.bank_num;
    }
}
