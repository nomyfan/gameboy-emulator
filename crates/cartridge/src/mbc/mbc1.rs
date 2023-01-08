use super::{RamBank, RomBank};
use shared::boxed_array_fn;

pub(crate) struct Mbc1 {
    /// 00h = ROM Banking Mode (up to 8KiB RAM, 2MiB ROM) (default)
    /// 01h = RAM Banking Mode (up to 32KiB RAM, 512KiB ROM)
    banking_mode: u8,
    /// Only enable it when writing any value whose lower 4 bits is 0xA.
    ram_enabled: bool,
    /// Current ROM bank number, in range of [0x01, 0x7F], 127 in total.
    rom_banking_num: usize,
    /// Current RAM bank number, in range of [0x00, 0x03], 4 in total.
    ram_banking_num: usize,
    rom_banks: Box<[RomBank; 127]>,
    ram_banks: Box<[RamBank; 4]>,
}

impl Mbc1 {
    pub(crate) fn new() -> Self {
        Mbc1 {
            banking_mode: 0,
            ram_enabled: false,
            rom_banking_num: 1,
            ram_banking_num: 0,
            rom_banks: boxed_array_fn(|_| [0u8; 0x4000]),
            ram_banks: boxed_array_fn(|_| [0u8; 0x2000]),
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
                self.rom_banking_num = if value & 0b1_1111 == 0 {
                    // 0x20, 0x40, 0x60 will be translated to 0x21, 0x41, 0x61,
                    // which means 0x20, 0x40, 0x60 cannot be used.
                    (value & 0b110_0000) as usize | 1
                } else {
                    (value & 0b111_1111) as usize
                };
            }
            // Select RAM bank
            0x4000..=0x5FFF => {
                self.ram_banking_num = (value & 0b11) as usize;
                // TODO save battery if RAM is banking(banking mode is 1).
            }
            // Select banking mode
            0x6000..=0x7FFF => {
                self.banking_mode = value & 0b1;
                // TODO save battery if RAM is banking(banking mode is 1).
            }
            // Write RAM
            0xA000..=0xBFFF => {
                if self.ram_enabled && self.banking_mode == 1 {
                    self.ram_banks[self.ram_banking_num][(addr - 0xA000) as usize] = value;
                }
            }
            _ => {
                // Unreachable, do nothing.
            }
        }
    }

    fn read(&self, addr: u16, rom: &[u8]) -> u8 {
        match addr {
            // Fixed ROM(ROM bank 0)
            0x0000..=0x3FFF => {
                return rom[addr as usize];
            }
            // ROM bank
            0x4000..=0x7FFF => {
                return self.rom_banks[self.rom_banking_num - 1][(addr - 0x4000) as usize];
            }
            // RAM bank
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return 0xFF;
                }

                if self.ram_banking_num == 0 {
                    return 0xFF;
                }

                return self.ram_banks[self.ram_banking_num][(addr - 0xA000) as usize];
            }
            _ => unreachable!("Invalid addr {:#02X} for MBC1", addr),
        }
    }
}
