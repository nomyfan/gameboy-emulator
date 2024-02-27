use crate::blipbuf;

pub struct PulseChannel {
    blipbuf: blipbuf::BlipBuf,
    /// Sweep register.
    pub(crate) nrx0: u8,
    /// Sound length/Wave pattern duty at 0xFF11.
    pub(crate) nrx1: u8,
    /// Volume envelope.
    pub(crate) nrx2: u8,
    /// Period lo.
    /// The low 8 bits of the period value.
    pub(crate) nrx3: u8,
    /// Period hi and control.
    /// Bit 7: Trigger.
    /// Bit 6: Length enable.
    /// Bit 2..=0: The upper 3 bits of the period value.
    pub(crate) nrx4: u8,
}

impl PulseChannel {
    /// Create CH1(left) and CH2(right).
    pub fn new_chs(frequency: u32, sample_rate: u32) -> (Self, Self) {
        (
            Self {
                blipbuf: blipbuf::new(frequency, sample_rate),
                nrx0: 0x80,
                nrx1: 0xBF,
                nrx2: 0xF3,
                nrx3: 0xFF,
                nrx4: 0xBF,
            },
            Self {
                blipbuf: blipbuf::new(frequency, sample_rate),
                nrx0: 0x80,
                nrx1: 0x3F,
                nrx2: 0x00,
                nrx3: 0xFF,
                nrx4: 0xBF,
            },
        )
    }
}
