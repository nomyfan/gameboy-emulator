use gb_shared::{boxed_array, Memory};

pub(crate) struct HighRam {
    /// [FF80, FFFF)
    ram: Box<[u8; 0x80]>,
}

impl HighRam {
    pub(crate) fn new() -> Self {
        Self { ram: boxed_array(0) }
    }
}

impl Memory for HighRam {
    fn write(&mut self, addr: u16, value: u8) {
        debug_assert!((0xFF80..=0xFFFE).contains(&addr));

        let addr = (addr as usize) - 0xFF80;
        self.ram[addr] = value;
    }

    fn read(&self, addr: u16) -> u8 {
        debug_assert!((0xFF80..=0xFFFE).contains(&addr));

        let addr = (addr as usize) - 0xFF80;
        self.ram[addr]
    }
}
