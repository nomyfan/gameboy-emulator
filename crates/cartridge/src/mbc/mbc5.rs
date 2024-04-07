use gb_shared::{boxed_array, kib};

use super::{real_ram_size, Mbc, RamBank};
use crate::CartridgeHeader;

pub(crate) struct Mbc5 {
    /// Only lower 9 bits are used. So its' value is in range of
    /// 0x0000..=0x01FF.
    rom_bank: u16,
    ram_banks: Vec<Box<RamBank>>,
    /// Writing values whose lower nibble is 0xA into 0x0000..=0x1FFF
    /// enable this, while others disable it.
    ram_enabled: bool,
    /// RAM bank number. On cartridges which feature a rumble motor,
    /// bit 3 of the RAM Bank register is connected to the Rumble
    /// circuitry instead of the RAM chip. Setting the bit to 1
    /// enables the rumble motor and keeps it enabled until the
    /// bit is reset again.
    ram_bank: u8,
    /// It controls how the bit 3 in RAM bank is used.
    /// - If it's true, then bit 3 controls rumble motor.
    /// - If it's false, then bit 3 will be used in RAM bank number.
    with_rumble_motor: bool,
    with_battery: bool,
}

impl Mbc5 {
    pub(crate) fn new(header: &CartridgeHeader) -> Self {
        let ram_banks_len = real_ram_size(header.ram_size) / kib(8);
        let mut ram_banks: Vec<Box<RamBank>> = Vec::with_capacity(ram_banks_len);
        for _ in 0..ram_banks_len {
            ram_banks.push(boxed_array(0));
        }

        let cart_type = header.cart_type;
        let with_rumble_motor = (0x1C..=0x1E).contains(&cart_type);
        let with_battery = cart_type == 0x1B || cart_type == 0x1E;

        Self {
            rom_bank: 0,
            ram_banks,
            ram_enabled: false,
            ram_bank: 0,
            with_rumble_motor,
            with_battery,
        }
    }

    #[inline]
    fn ram_bank_num(&self) -> usize {
        if self.with_rumble_motor {
            (self.ram_bank as usize) & 0b111
        } else {
            (self.ram_bank as usize) & 0b1111
        }
    }
}

impl Mbc for Mbc5 {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0A) == 0x0A;
            }
            0x2000..=0x2FFF => {
                self.rom_bank = (self.rom_bank & 0x0100) | value as u16;
            }
            0x3000..=0x3FFF => {
                self.rom_bank = (self.rom_bank & 0x00FF) | ((value as u16 & 1) << 8);
            }
            0x4000..=0x5FFF => {
                self.ram_bank = value & 0xF;
            }
            0x6000..=0x7FFF => {
                // noop
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return;
                }
                let ram_bank_num = self.ram_bank_num();
                self.ram_banks[ram_bank_num][addr as usize - 0xA000] = value;
            }
            _ => unreachable!("Invalid addr {:#04X} for MBC5", addr),
        }
    }

    fn read(&self, addr: u16, rom: &[u8]) -> u8 {
        match addr {
            0x0000..=0x3FFF => rom[addr as usize],
            0x4000..=0x7FFF => {
                let rom_bank_num = self.rom_bank as usize;
                rom[rom_bank_num * kib(16) + (addr as usize - 0x4000)]
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return 0xFF;
                }
                let ram_bank_num = self.ram_bank_num();
                self.ram_banks[ram_bank_num][addr as usize - 0xA000]
            }
            _ => unreachable!("Invalid addr {:#04X} for MBC5", addr),
        }
    }

    #[cfg(not(target_family = "wasm"))]
    fn store(&self, path: &std::path::Path) -> anyhow::Result<()> {
        if self.with_battery {
            use std::io::Write;
            let mut file = std::fs::File::create(path)?;
            for bank in &self.ram_banks {
                file.write_all(bank.as_ref())?;
            }
            file.flush()?;
        }

        Ok(())
    }

    #[cfg(not(target_family = "wasm"))]
    fn restore(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        if self.with_battery {
            use std::io::Read;
            let mut file = std::fs::File::open(path)?;
            if file.metadata()?.len() as usize != self.ram_banks.len() * kib(8) {
                // Ignore invalid file.
                return Ok(());
            }
            for bank in &mut self.ram_banks {
                file.read_exact(bank.as_mut())?;
            }
        }

        Ok(())
    }
}
