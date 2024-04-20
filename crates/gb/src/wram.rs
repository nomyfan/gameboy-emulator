use gb_shared::{boxed_array, boxed_array_try_from_vec, Memory, Snapshot};

pub(crate) struct WorkRam {
    /// [C000, E000)
    /// (4 + 4)KiB
    ram: Box<[u8; 0x2000]>,
}

impl WorkRam {
    pub(crate) fn new() -> Self {
        Self { ram: boxed_array(0) }
    }
}

impl Memory for WorkRam {
    fn write(&mut self, addr: u16, value: u8) {
        debug_assert!((0xC000..=0xDFFF).contains(&addr));

        let addr = (addr as usize) - 0xC000;
        self.ram[addr] = value;
    }

    fn read(&self, addr: u16) -> u8 {
        debug_assert!((0xC000..=0xDFFF).contains(&addr));

        let addr = (addr as usize) - 0xC000;
        self.ram[addr]
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct WorkRamSnapshot {
    ram: Vec<u8>,
}

impl Snapshot for WorkRam {
    type Snapshot = WorkRamSnapshot;

    fn take_snapshot(&self) -> Self::Snapshot {
        WorkRamSnapshot { ram: self.ram.to_vec() }
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        self.ram = boxed_array_try_from_vec(snapshot.ram).unwrap();
    }
}
