use gb_shared::{kib, Snapshot};

pub(crate) mod mbc1;
pub(crate) mod mbc2;
pub(crate) mod mbc3;
pub(crate) mod mbc5;
pub(crate) mod mbc_none;

/// The size of each RAM bank is 8KiB
type RamBank = [u8; 0x2000];

pub(crate) trait Mbc: Snapshot {
    fn write(&mut self, addr: u16, value: u8);
    fn read(&self, addr: u16, rom: &[u8]) -> u8;
    /// For battery-backed cartridge.
    fn suspend(&self) -> Option<Vec<u8>> {
        None
    }
    #[allow(unused_variables)]
    /// For battery-backed cartridge.
    fn resume(&mut self, data: &[u8]) -> anyhow::Result<()> {
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
