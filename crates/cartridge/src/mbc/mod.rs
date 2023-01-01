pub(crate) mod mbc1;
pub(crate) mod none_mbc;

/// The size of each ROM bank is 16KiB
type RomBank = [u8; 0x4000];

/// The size of each RAM bank is 8KiB
type RamBank = [u8; 0x2000];

pub(crate) trait Mbc {
    fn write(&mut self, addr: u16, value: u8);
    fn read(&self, addr: u16, rom: &[u8]) -> u8;
}
