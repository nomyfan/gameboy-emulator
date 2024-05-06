use gb_shared::{MachineModel, Memory, Snapshot};

pub(crate) struct MiscRam {
    machine_model: MachineModel,
    reg_ff72: u8,
    reg_ff73: u8,
    reg_ff74: u8,
    reg_ff75: u8,
}

impl MiscRam {
    pub(crate) fn new(machine_model: MachineModel) -> Self {
        Self { machine_model, reg_ff72: 0, reg_ff73: 0, reg_ff74: 0, reg_ff75: 0 }
    }
}

impl Memory for MiscRam {
    fn write(&mut self, addr: u16, value: u8) {
        if self.machine_model == MachineModel::DMG && (0xFF72..=0xFF75).contains(&addr) {
            log::warn!("{:#X} should not be written on DMG", addr);
        }

        match addr {
            0xFF72 => self.reg_ff72 = value,
            0xFF73 => self.reg_ff73 = value,
            0xFF74 => self.reg_ff74 = value,
            0xFF75 => self.reg_ff75 = value,
            _ => unreachable!("Invalid MiscRAM write {:#X} {:#X}", addr, value),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        if self.machine_model == MachineModel::DMG && (0xFF72..=0xFF75).contains(&addr) {
            log::warn!("{:#X} should not be read on DMG", addr);
        }

        match addr {
            0xFF72 => self.reg_ff72,
            0xFF73 => self.reg_ff73,
            0xFF74 => self.reg_ff74,
            0xFF75 => 0x8F | self.reg_ff75,
            _ => unreachable!("Invalid MiscRAM write {:#X}", addr),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct MiscRamSnapshot {
    reg_ff72: u8,
    reg_ff73: u8,
    reg_ff74: u8,
    reg_ff75: u8,
}

impl Snapshot for MiscRam {
    type Snapshot = MiscRamSnapshot;

    fn take_snapshot(&self) -> Self::Snapshot {
        MiscRamSnapshot {
            reg_ff72: self.reg_ff72,
            reg_ff73: self.reg_ff73,
            reg_ff74: self.reg_ff74,
            reg_ff75: self.reg_ff75,
        }
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        self.reg_ff72 = snapshot.reg_ff72;
        self.reg_ff73 = snapshot.reg_ff73;
        self.reg_ff74 = snapshot.reg_ff74;
        self.reg_ff75 = snapshot.reg_ff75;
    }
}
