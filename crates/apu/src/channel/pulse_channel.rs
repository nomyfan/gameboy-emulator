use crate::{blipbuf, clock::Clock, utils::pulse_channel_sample_period};

pub(crate) struct PulseChannel {
    blipbuf: blipbuf::BlipBuf,
    clock: Clock,
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
    period_value: u16,
}

#[inline]
fn period_value(nrx3: u8, nrx4: u8) -> u16 {
    ((nrx4 as u16 & 0b111) << 8) | (nrx3 as u16)
}

impl PulseChannel {
    /// Create CH1(left) and CH2(right).
    pub(crate) fn new_chs(frequency: u32, sample_rate: u32) -> (Self, Self) {
        let nrx0 = 0x80;
        let nrx3 = 0xFF;
        let nrx4 = 0xBF;
        let period_value = period_value(nrx3, nrx4);

        (
            Self {
                blipbuf: blipbuf::new(frequency, sample_rate),
                clock: Clock::new(pulse_channel_sample_period(period_value)),
                nrx0,
                nrx1: 0xBF,
                nrx2: 0xF3,
                nrx3,
                nrx4,
                period_value,
            },
            Self {
                blipbuf: blipbuf::new(frequency, sample_rate),
                clock: Clock::new(pulse_channel_sample_period(period_value)),
                nrx0,
                nrx1: 0x3F,
                nrx2: 0x00,
                nrx3,
                nrx4,
                period_value,
            },
        )
    }

    #[inline]
    fn dac_off(&self) -> bool {
        (self.nrx2 >> 3) == 0
    }

    #[inline]
    fn period_overflow(&self) -> bool {
        self.period_value > 0x7FF
    }

    pub(crate) fn next(&mut self) {
        todo!()
    }
}
