use gb_shared::Snapshot;

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

impl Snapshot for MbcNone {
    type Snapshot = Vec<u8>;

    fn take_snapshot(&self) -> Self::Snapshot {
        vec![]
    }

    fn restore_snapshot(&mut self, _snapshot: Self::Snapshot) {}
}
