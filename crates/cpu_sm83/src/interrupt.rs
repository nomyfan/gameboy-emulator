pub(crate) struct Interrupt {
    pub(crate) flag: u8,
    pub(crate) handler_address: u16,
}

/// Lower bits have higher priorities.
pub(crate) const INTERRUPTS: &[Interrupt; 5] = &[
    // VBlank
    Interrupt { flag: 0b1, handler_address: 0x40 },
    // STAT
    Interrupt { flag: 0b10, handler_address: 0x48 },
    // Timer
    Interrupt { flag: 0b100, handler_address: 0x50 },
    // Serial
    Interrupt { flag: 0b1000, handler_address: 0x58 },
    // Joypad
    Interrupt { flag: 0b10000, handler_address: 0x60 },
];
