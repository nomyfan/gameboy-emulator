use super::{real_ram_size, RamBank};
use crate::CartridgeHeader;
use gb_shared::{boxed_array, kib};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// https://gbdev.io/pandocs/MBC1.html
pub(crate) struct Mbc1 {
    /// 00h = ROM Banking Mode (up to 8KiB banked RAM, 2MiB ROM) (default)
    /// 01h = RAM Banking Mode (up to 32KiB banked RAM, 512KiB ROM)
    banking_mode: u8,
    /// Only enable it when writing any value whose lower 4 bits is 0xA.
    ram_enabled: bool,
    /// The lower 2 + 5 bits are used.
    banking_num: usize,
    /// Max size, 32KiB.
    ram_banks: Vec<Box<RamBank>>,
    with_battery: bool,
}

impl Mbc1 {
    pub(crate) fn new(header: &CartridgeHeader) -> Self {
        let cart_type = header.cart_type;
        let ram_banks_len = real_ram_size(header.ram_size) / kib(8);
        let mut ram_banks: Vec<Box<RamBank>> = Vec::with_capacity(ram_banks_len);
        for _ in 0..ram_banks_len {
            ram_banks.push(boxed_array(0));
        }

        Mbc1 {
            banking_mode: 0,
            ram_enabled: false,
            banking_num: 0,
            ram_banks,
            with_battery: cart_type == 0x03,
        }
    }
}

impl super::Mbc for Mbc1 {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            // Enable or disable RAM
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            // Select ROM bank
            0x2000..=0x3FFF => {
                let value = value & 0x1F;
                self.banking_num = (self.banking_num & 0x60) | value as usize;
            }
            // Select RAM bank or upper 2 bits of ROM bank
            0x4000..=0x5FFF => {
                let value = value & 0b11;
                self.banking_num = (self.banking_num & !(0x60)) | (value << 5) as usize;
            }
            // Select banking mode
            0x6000..=0x7FFF => {
                self.banking_mode = value & 0b1;
            }
            // Write RAM
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let ram_bank_num = (self.banking_num >> 5) & 0b11;
                    self.ram_banks[ram_bank_num][(addr - 0xA000) as usize] = value;
                }
            }
            _ => unreachable!("Invalid addr {:#04X} for MBC1", addr),
        }
    }

    fn read(&self, addr: u16, rom: &[u8]) -> u8 {
        match addr {
            // Fixed ROM(ROM bank 0)
            0x0000..=0x3FFF => rom[addr as usize],
            // ROM bank
            0x4000..=0x7FFF => {
                let mut rom_bank_num = if self.banking_mode == 1 {
                    // 7 bits
                    self.banking_num & 0x7F
                } else {
                    // 5 bits
                    self.banking_num & 0x1F
                };
                if rom_bank_num == 0 {
                    rom_bank_num = 1; // Bank 0 is the fixed ROM.
                }
                let rom_offset = rom_bank_num * kib(16) + (addr - 0x4000) as usize;
                rom[rom_offset]
            }
            // RAM bank
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return 0xFF;
                }

                let ram_bank_num = (self.banking_num >> 5) & 0b11;
                self.ram_banks[ram_bank_num][(addr - 0xA000) as usize]
            }
            _ => unreachable!("Invalid addr {:#04X} for MBC1", addr),
        }
    }

    fn store(&self, path: &Path) -> anyhow::Result<()> {
        if self.with_battery {
            let mut file = File::create(path)?;
            for bank in &self.ram_banks {
                file.write_all(bank.as_ref())?;
            }
            file.flush()?;
        }

        Ok(())
    }

    fn restore(&mut self, path: &Path) -> anyhow::Result<()> {
        if self.with_battery {
            let mut file = File::open(path)?;
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
