pub(crate) struct NoneMbc {}

impl NoneMbc {
    pub(crate) fn new() -> Self {
        NoneMbc {}
    }
}

impl super::Mbc for NoneMbc {
    fn write(&mut self, _addr: u16, _value: u8) {
        // Noop
    }

    fn read(&self, addr: u16, rom: &[u8]) -> u8 {
        debug_assert!(addr < 0x4000);
        rom[addr as usize]
    }
}
