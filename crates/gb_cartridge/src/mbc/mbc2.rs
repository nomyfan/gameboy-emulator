use super::Mbc;
use crate::CartridgeHeader;
use gb_shared::{boxed::BoxedArray, is_bit_set, kib, Snapshot};
use serde::{Deserialize, Serialize};

/// Max 256 KiB ROM, 512x4 bits RAM
pub(crate) struct Mbc2 {
    ram_enabled: bool,
    /// Value range 0x01..=0x0F
    rom_bank_num: u8,
    ram: BoxedArray<u8, 0x200>,
    with_battery: bool,
}

impl Mbc2 {
    pub(crate) fn new(header: &CartridgeHeader) -> Self {
        let with_battery = header.cart_type == 0x06;
        let ram = Default::default();

        Self { ram_enabled: false, rom_bank_num: 0x01, ram, with_battery }
    }
}

impl Mbc for Mbc2 {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x3FFF => {
                if is_bit_set!(addr, 8) {
                    self.rom_bank_num = value & 0x0F;
                } else {
                    self.ram_enabled = (value & 0x0F) == 0x0A;
                }
            }
            0x4000..=0x7FFF => {
                // Noop
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let addr = (addr & 0xA1FF) - 0xA000;
                    self.ram[addr as usize] = value & 0x0F;
                }
            }
            _ => unreachable!("Invalid addr {:#04X} for MBC2", addr),
        }
    }

    fn read(&self, addr: u16, rom: &[u8]) -> u8 {
        match addr {
            0x0000..=0x3FFF => rom[addr as usize],
            0x4000..=0x7FFF => {
                let mut rom_bank_num = self.rom_bank_num as usize;
                if rom_bank_num == 0 {
                    rom_bank_num = 1;
                }
                let rom_addr = rom_bank_num * kib(16) + (addr - 0x4000) as usize;
                rom[rom_addr]
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    self.ram[((addr & 0xA1FF) - 0xA000) as usize]
                } else {
                    0xFF
                }
            }
            _ => unreachable!("Invalid addr {:#04X} for MBC2", addr),
        }
    }

    fn suspend(&self) -> Option<Vec<u8>> {
        if self.with_battery {
            let mut data = vec![];
            data.extend_from_slice(self.ram.as_ref());

            return Some(data);
        }

        None
    }

    fn resume(&mut self, data: &[u8]) -> anyhow::Result<()> {
        if self.with_battery {
            if data.len() != self.ram.len() {
                anyhow::bail!("Invalid data length for MBC2")
            }
            self.ram.copy_from_slice(data);
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Mbc2Snapshot {
    ram_enabled: bool,
    rom_bank_num: u8,
    ram: Vec<u8>,
    with_battery: bool,
}

impl Snapshot for Mbc2 {
    type Snapshot = Vec<u8>;

    fn take_snapshot(&self) -> Self::Snapshot {
        let mut ram_snapshot = vec![];
        ram_snapshot.extend_from_slice(self.ram.as_ref());

        bincode::serialize(&Mbc2Snapshot {
            ram_enabled: self.ram_enabled,
            rom_bank_num: self.rom_bank_num,
            ram: ram_snapshot,
            with_battery: self.with_battery,
        })
        .unwrap()
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        let Mbc2Snapshot { ram_enabled, rom_bank_num, ram, with_battery } =
            bincode::deserialize(&snapshot).unwrap();
        assert_eq!(ram.len(), self.ram.len());

        self.ram_enabled = ram_enabled;
        self.rom_bank_num = rom_bank_num;
        self.ram.copy_from_slice(&ram);
        self.with_battery = with_battery;
    }
}
