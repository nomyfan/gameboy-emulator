use std::path::Path;

pub(crate) mod mbc1;
pub(crate) mod mbc_none;

/// The size of each RAM bank is 8KiB
type RamBank = [u8; 0x2000];

pub(crate) trait Mbc {
    fn write(&mut self, addr: u16, value: u8);
    fn read(&self, addr: u16, rom: &[u8]) -> u8;
    #[allow(unused_variables)]
    /// For battery-backed cartridge.
    fn save_ram(&self, path: &Path) -> anyhow::Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    /// For battery-backed cartridge.
    fn load_ram(&mut self, path: &Path) -> anyhow::Result<()> {
        Ok(())
    }
}
