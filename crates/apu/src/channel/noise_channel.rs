use gb_shared::{is_bit_set, unset_bits};

use crate::{blipbuf, clock::Clock, length_timer::LengthTimer};

use super::volume_envelope::VolumeEnvelope;

struct Lfsr {
    value: u16,
    clock: Clock,
}

impl Lfsr {
    fn new_lfsr_clock(nrx3: u8) -> Clock {
        let shift = (nrx3 >> 4) & 0xF;
        let divider = (nrx3 & 0b111) as u32;

        // CPU_FREQ / (262144 / (divider * 2^shift))
        let div = if divider == 0 {
            // Note that divider = 0 is treated as divider = 0.5 instead.
            1 << (shift + 3)
        } else {
            divider * (1 << (shift + 4))
        };
        Clock::new(div)
    }
    fn new(nrx3: u8) -> Self {
        Self { value: 0, clock: Self::new_lfsr_clock(nrx3) }
    }

    fn next(&mut self, nrx3: u8) -> Option<bool> {
        if self.clock.next() {
            // Algorithm, see https://gbdev.io/pandocs/Audio_details.html#noise-channel-ch4:~:text=to%20shift%20in.-,when%20ch4%20is%20ticked,-(at%20the%20frequency
            let b0 = self.value & 1;
            let b1 = (self.value >> 1) & 1;
            let bit = b0 ^ b1;

            self.value = unset_bits!(self.value, 15) | (bit << 15);
            if is_bit_set!(nrx3, 3) {
                self.value = unset_bits!(self.value, 7) | (bit << 7);
            }

            self.value >>= 1;

            return Some(b0 == 1);
        }

        None
    }
}

pub(crate) struct NoiseChannel {
    blipbuf: blipbuf::BlipBuf,
    /// Length timer.
    nrx1: u8,
    /// Volume envelope.
    nrx2: u8,
    /// Frequency and randomness.
    /// Bit 7..=4: Clock shift.
    /// Bit 3: LFSR width.
    /// Bit 2..=0: Clock divider.
    nrx3: u8,
    /// Channel control.
    /// Bit 7: Trigger.
    /// Bit 6: Length enable.
    nrx4: u8,
    length_timer: Option<LengthTimer>,
    volume_envelope: VolumeEnvelope,
    lfsr: Lfsr,
}

impl NoiseChannel {
    pub(crate) fn new(frequency: u32, sample_rate: u32) -> Self {
        let nrx2 = 0x00;
        let nrx3 = 0x00;
        Self {
            blipbuf: blipbuf::BlipBuf::new(frequency, sample_rate, 0),
            nrx1: 0xFF,
            nrx2,
            nrx3,
            nrx4: 0xBF,
            length_timer: None,
            volume_envelope: VolumeEnvelope::new(nrx2),
            lfsr: Lfsr::new(nrx3),
        }
    }

    fn triggered(&self) -> bool {
        is_bit_set!(self.nrx4, 7)
    }

    fn active(&self) -> bool {
        let length_timer_expired = self.length_timer.as_ref().map_or(false, |lt| lt.expired());

        !length_timer_expired
    }

    pub(crate) fn next(&mut self) {
        if let Some(use_volume) = self.lfsr.next(self.nrx3) {
            let volume =
                if use_volume && self.active() { self.volume_envelope.volume() as i32 } else { 0 };
            self.blipbuf.add_delta(self.lfsr.clock.div(), volume);
        }

        self.volume_envelope.next(self.nrx2);

        if let Some(length_timer) = self.length_timer.as_mut() {
            length_timer.next();
        }
    }

    pub(crate) fn read_samples(&mut self, duration: u32) -> Vec<i16> {
        self.blipbuf.end(duration)
    }
}

impl NoiseChannel {
    #[inline]
    pub(crate) fn nrx1(&self) -> u8 {
        self.nrx1
    }

    #[inline]
    pub(crate) fn nrx2(&self) -> u8 {
        self.nrx2
    }

    #[inline]
    pub(crate) fn nrx3(&self) -> u8 {
        self.nrx3
    }

    #[inline]
    pub(crate) fn nrx4(&self) -> u8 {
        self.nrx4
    }

    #[inline]
    pub(crate) fn set_nrx1(&mut self, value: u8) {
        self.nrx1 = value;
    }

    pub(crate) fn set_nrx2(&mut self, value: u8) {
        self.nrx2 = value;
    }

    pub(crate) fn set_nrx3(&mut self, value: u8) {
        self.nrx3 = value;
    }

    pub(crate) fn set_nrx4(&mut self, value: u8) {
        self.nrx4 = value;

        if self.triggered() {
            self.length_timer = if is_bit_set!(self.nrx4, 6) {
                Some(LengthTimer::new(self.nrx1 & 0x3F))
            } else {
                None
            };

            self.volume_envelope = VolumeEnvelope::new(self.nrx2);
            self.lfsr = Lfsr::new(self.nrx3);
        }
    }
}
