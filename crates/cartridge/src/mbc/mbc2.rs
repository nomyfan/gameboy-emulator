use super::Mbc;
use crate::CartridgeHeader;
use gb_shared::{boxed::BoxedArray, is_bit_set, kib};

/// Max 256 KiB ROM, 512x4 bits RAM
pub(crate) struct Mbc2 {
    ram_enabled: bool,
    /// Value range 0x01..=0x0F
    rom_bank: u8,
    ram: BoxedArray<u8, 0x200>,
    with_battery: bool,
}

impl Mbc2 {
    pub(crate) fn new(header: &CartridgeHeader) -> Self {
        let with_battery = header.cart_type == 0x06;
        let ram = Default::default();

        Self { ram_enabled: false, rom_bank: 0x01, ram, with_battery }
    }
}

impl Mbc for Mbc2 {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x3FFF => {
                if is_bit_set!(addr, 8) {
                    self.rom_bank = value & 0x0F;
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
                let mut rom_bank_num = self.rom_bank as usize;
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

    #[cfg(not(target_family = "wasm"))]
    fn store(&self, path: &std::path::Path) -> anyhow::Result<()> {
        if self.with_battery {
            use std::io::Write;
            let mut file = std::fs::File::create(path)?;
            file.write_all(self.ram.as_ref())?;
            file.flush()?;
        }

        Ok(())
    }

    #[cfg(not(target_family = "wasm"))]
    fn restore(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        if self.with_battery {
            use std::io::Read;
            let mut file = std::fs::File::open(path)?;
            if file.metadata()?.len() as usize != self.ram.len() {
                // Ignore invalid file.
                return Ok(());
            }
            file.read_exact(self.ram.as_mut())?;
        }

        Ok(())
    }
}
