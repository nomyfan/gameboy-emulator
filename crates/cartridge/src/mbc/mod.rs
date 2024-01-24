use gb_shared::kib;
use std::path::Path;

pub(crate) mod mbc1;
pub(crate) mod mbc3;
pub(crate) mod mbc5;
pub(crate) mod mbc_none;

/// The size of each RAM bank is 8KiB
type RamBank = [u8; 0x2000];

pub(crate) trait Mbc {
    fn write(&mut self, addr: u16, value: u8);
    fn read(&self, addr: u16, rom: &[u8]) -> u8;
    #[allow(unused_variables)]
    /// For battery-backed cartridge.
    fn store(&self, path: &Path) -> anyhow::Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    /// For battery-backed cartridge.
    fn restore(&mut self, path: &Path) -> anyhow::Result<()> {
        Ok(())
    }
}

/// https://gbdev.io/pandocs/The_Cartridge_Header.html#0149--ram-size
pub(crate) fn real_ram_size(ram_size: u8) -> usize {
    match ram_size {
        0x02 => kib(8),
        0x03 => kib(32),
        0x04 => kib(128),
        0x05 => kib(64),
        _ => 0,
    }
}
