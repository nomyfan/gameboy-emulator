use crate::blipbuf;

pub struct NoiseChannel {
    blipbuf: blipbuf::BlipBuf,
    /// Length timer.
    pub(crate) nrx1: u8,
    /// Volume envelope.
    pub(crate) nrx2: u8,
    /// Frequency and randomness.
    /// Bit 7..=4: Clock shift.
    /// Bit 3: LFSR width.
    /// Bit 2..=0: Clock divider.
    pub(crate) nrx3: u8,
    /// Channel control.
    /// Bit 7: Trigger.
    /// Bit 6: Length enable.
    pub(crate) nrx4: u8,
}

impl NoiseChannel {
    pub fn new(frequency: u32, sample_rate: u32) -> Self {
        Self {
            blipbuf: blipbuf::BlipBuf::new(frequency, sample_rate, 0),
            nrx1: 0xFF,
            nrx2: 0x00,
            nrx3: 0x00,
            nrx4: 0xBF,
        }
    }
}
