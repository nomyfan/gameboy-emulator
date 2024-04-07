use gb_shared::{Memory, Snapshot};

pub(crate) struct Serial {
    data: u8,
    control: u8,
}

impl Memory for Serial {
    fn write(&mut self, addr: u16, value: u8) {
        if addr == 0xFF01 {
            self.data = value;
        } else if addr == 0xFF02 {
            self.control = value | 0x7C;
        } else {
            unreachable!()
        }
    }

    fn read(&self, addr: u16) -> u8 {
        if addr == 0xFF01 {
            self.data
        } else if addr == 0xFF02 {
            self.control
        } else {
            unreachable!()
        }
    }
}

impl Serial {
    pub(crate) fn new() -> Self {
        Self { data: 0, control: 0x7E }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SerialSnapshot {
    data: u8,
    control: u8,
}

impl Snapshot for Serial {
    type Snapshot = SerialSnapshot;

    fn snapshot(&self) -> Self::Snapshot {
        SerialSnapshot { data: self.data, control: self.control }
    }

    fn restore(&mut self, snapshot: Self::Snapshot) {
        self.data = snapshot.data;
        self.control = snapshot.control;
    }
}
