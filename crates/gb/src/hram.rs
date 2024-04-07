use gb_shared::{boxed_array, boxed_array_try_from_vec, Memory, Snapshot};

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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct HighRamSnapshot {
    ram: Vec<u8>,
}

impl Snapshot for HighRam {
    type Snapshot = HighRamSnapshot;

    fn snapshot(&self) -> Self::Snapshot {
        HighRamSnapshot { ram: self.ram.to_vec() }
    }

    fn restore(&mut self, snapshot: Self::Snapshot) {
        self.ram = boxed_array_try_from_vec(snapshot.ram).unwrap();
    }
}
