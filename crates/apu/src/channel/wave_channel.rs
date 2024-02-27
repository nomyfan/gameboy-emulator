use crate::blipbuf;

pub struct WaveChannel {
    blipbuf: blipbuf::BlipBuf,
    /// DAC enable.
    /// Bit 7: On/Off.
    pub(crate) nrx0: u8,
    /// Length timer.
    pub(crate) nrx1: u8,
    /// Output level.
    /// Bit 6..=5: Output level.
    /// 00: Mute.
    /// 01: 100%.
    /// 10: 50%.
    /// 11: 25%.
    pub(crate) nrx2: u8,
    /// Period low.
    /// The low 8 bits of the period value.
    pub(crate) nrx3: u8,
    /// Period hi and control.
    /// Bit 7: Trigger.
    /// Bit 6: Length enable.
    /// Bit 2..=0: The upper 3 bits of the period value.
    pub(crate) nrx4: u8,
    /// Wave pattern RAM.
    pub(crate) wave_ram: [u8; 16],
}

impl WaveChannel {
    pub fn new(frequency: u32, sample_rate: u32) -> Self {
        Self {
            blipbuf: blipbuf::new(frequency, sample_rate),
            nrx0: 0x7F,
            nrx1: 0xFF,
            nrx2: 0x9F,
            nrx3: 0xFF,
            nrx4: 0xBF,
            wave_ram: Default::default(),
        }
    }
}
