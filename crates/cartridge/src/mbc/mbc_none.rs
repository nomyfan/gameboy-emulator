pub(crate) struct MbcNone {}

impl MbcNone {
    pub(crate) fn new() -> Self {
        MbcNone {}
    }
}

impl super::Mbc for MbcNone {
    fn write(&mut self, _addr: u16, _value: u8) {
        // Noop
    }

    fn read(&self, addr: u16, rom: &[u8]) -> u8 {
        rom[addr as usize]
    }
}
