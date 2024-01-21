use gb_shared::Memory;

pub(crate) struct Serial {
    serial_transfer_data: u8,
    serial_transfer_control: u8,
}

impl Memory for Serial {
    fn write(&mut self, addr: u16, value: u8) {
        if addr == 0xFF01 {
            self.serial_transfer_data = value;
        } else if addr == 0xFF02 {
            self.serial_transfer_control = value | 0x7C;
        } else {
            unreachable!()
        }
    }

    fn read(&self, addr: u16) -> u8 {
        if addr == 0xFF01 {
            self.serial_transfer_data
        } else if addr == 0xFF02 {
            self.serial_transfer_control
        } else {
            unreachable!()
        }
    }
}

impl Serial {
    pub(crate) fn new() -> Self {
        Self { serial_transfer_data: 0, serial_transfer_control: 0x7E }
    }
}
