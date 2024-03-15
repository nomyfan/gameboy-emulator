use gb_shared::{is_bit_set, unset_bits, Memory};

use crate::{blipbuf, clock::Clock};

use super::{Frame, NoiseChannelLengthCounter as LengthCounter, VolumeEnvelope};

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
}

impl Lfsr {
    fn set_clock(&mut self, nrx3: u8) {
        self.clock = Self::new_lfsr_clock(nrx3);
    }

    fn step(&mut self, nrx3: u8) -> Option<bool> {
        if self.clock.step() {
            // Algorithm, see https://gbdev.io/pandocs/Audio_details.html#noise-channel-ch4:~:text=to%20shift%20in.-,when%20ch4%20is%20ticked,-(at%20the%20frequency
            let b0 = self.value & 1;
            let b1 = (self.value >> 1) & 1;
            let bit = b0 ^ b1 ^ 1;

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
    /// Length timer.
    nrx1: u8,
    /// Volume envelope.
    /// Bit0..=2, pace. Control volume envelope clock frequency. 0 disables the envelope.
    /// Bit3, direction. 0: decrease, 1: increase.
    /// Bit4..=7, initial volume. Used to set volume envelope's volume.
    /// When Bit3..=7 are all 0, the DAC is off.
    nrx2: u8,
    /// Frequency and randomness.
    /// Bit2..=0, Clock divider.
    /// Bit3, LFSR width.
    /// Bit7..=4, Clock shift.
    nrx3: u8,
    /// Channel control.
    /// Bit7, Trigger.
    /// Bit6, Length enable.
    nrx4: u8,
    blipbuf: blipbuf::BlipBuf,
    length_counter: LengthCounter,
    volume_envelope: VolumeEnvelope,
    lfsr: Lfsr,
    active: bool,
}

impl NoiseChannel {
    pub(crate) fn new(frequency: u32, sample_rate: u32) -> Self {
        let nrx1 = 0;
        let nrx2 = 0;
        let nrx3 = 0;
        let nrx4 = 0;
        Self {
            blipbuf: blipbuf::BlipBuf::new(frequency, sample_rate, 0),
            nrx1,
            nrx2,
            nrx3,
            nrx4,
            length_counter: LengthCounter::new_expired(),
            volume_envelope: VolumeEnvelope::new(nrx2),
            lfsr: Lfsr::new(nrx3),
            active: false,
        }
    }

    #[inline]
    pub(crate) fn on(&self) -> bool {
        self.active
    }

    #[inline]
    fn dac_on(&self) -> bool {
        (self.nrx2 & 0xF8) != 0
    }

    #[inline]
    pub(crate) fn step(&mut self, frame: Option<Frame>) {
        if let Some(use_volume) = self.lfsr.step(self.nrx3) {
            let volume =
                if use_volume && (self.on()) { self.volume_envelope.volume() as i32 } else { 0 };
            self.blipbuf.add_delta(self.lfsr.clock.div(), volume);
        }

        if let Some(frame) = frame {
            self.volume_envelope.step(frame);
            self.length_counter.step(frame);
        }

        self.active &= self.length_counter.active();
    }

    pub(crate) fn read_samples(&mut self, buffer: &mut [i16], duration: u32) {
        self.blipbuf.end(buffer, duration)
    }

    pub(crate) fn power_off(&mut self) {
        // On DMG, length counter are unaffected by power and can still be written while off.
        self.nrx1 = 0;
        self.write(2, 0);
        self.write(3, 0);
        self.write(4, 0);

        self.length_counter.frame = Default::default();
    }

    pub(crate) fn set_length_counter(&mut self, value: u8) {
        self.length_counter.set_len(value);
    }
}

impl Memory for NoiseChannel {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            1 => {
                self.length_counter.set_len(value & 0x3F);
                self.nrx1 = value;
            }
            2 => {
                self.nrx2 = value;
                self.volume_envelope.set_nrx2(value);

                self.active &= self.dac_on();
            }
            3 => {
                self.lfsr.set_clock(value);
                self.nrx3 = value;
            }
            4 => {
                self.length_counter.set_enabled(value);

                // Trigger the channel
                if is_bit_set!(value, 7) {
                    self.length_counter.trigger();
                    self.volume_envelope = VolumeEnvelope::new(self.nrx2);
                    self.lfsr = Lfsr::new(self.nrx3);
                    self.blipbuf.clear();
                }
                self.active = self.length_counter.active();
                self.active &= self.dac_on();

                self.nrx4 = value;
            }
            _ => unreachable!("Invalid address for NoiseChannel: {:#X}", addr),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            1 => self.nrx1,
            2 => self.nrx2,
            3 => self.nrx3,
            4 => self.nrx4,
            _ => unreachable!("Invalid address for NoiseChannel: {:#X}", addr),
        }
    }
}
