use gb_shared::{box_array, Memory, Snapshot};

pub(crate) struct HighRam {
    /// [FF80, FFFF)
    ram: Box<[u8; 0x80]>,
}

impl HighRam {
    pub(crate) fn new() -> Self {
        Self { ram: box_array![u8; 0x80] }
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

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct HighRamSnapshot {
    ram: Vec<u8>,
}

impl Snapshot for HighRam {
    type Snapshot = HighRamSnapshot;

    fn take_snapshot(&self) -> Self::Snapshot {
        HighRamSnapshot { ram: self.ram.to_vec() }
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        assert_eq!(snapshot.ram.len(), 0x80);
        let boxed_slice = snapshot.ram.into_boxed_slice();
        let ptr = Box::into_raw(boxed_slice) as *mut [u8; 0x80];
        self.ram = unsafe { Box::from_raw(ptr) };
    }
}
