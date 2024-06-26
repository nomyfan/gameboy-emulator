use gb_shared::{is_bit_set, unset_bits, Memory, Snapshot};
use serde::{Deserialize, Serialize};

use crate::{blipbuf, clock::Clock};

use super::{Envelope, Frame, NoiseChannelLengthCounter as LengthCounter};

#[derive(Clone, Serialize, Deserialize)]
struct Lfsr {
    value: u16,
    clock: Clock,
}

impl std::fmt::Debug for Lfsr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#018b}", self.value))
    }
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
    blipbuf: Option<blipbuf::BlipBuf>,
    length_counter: LengthCounter,
    envelope: Envelope,
    lfsr: Lfsr,
    active: bool,
}

impl std::fmt::Debug for NoiseChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoiseChannel")
            .field("length_counter", &self.length_counter)
            .field("envelope", &self.envelope)
            .field("lfsr", &self.lfsr)
            .field("active", &self.active)
            .finish()
    }
}

impl NoiseChannel {
    pub(crate) fn new(frequency: u32, sample_rate: Option<u32>) -> Self {
        let nrx1 = 0;
        let nrx2 = 0;
        let nrx3 = 0;
        let nrx4 = 0;
        Self {
            blipbuf: sample_rate
                .map(|sample_rate| blipbuf::BlipBuf::new(frequency, sample_rate, 0)),
            nrx1,
            nrx2,
            nrx3,
            nrx4,
            length_counter: LengthCounter::new_expired(),
            envelope: Envelope::new(nrx2),
            lfsr: Lfsr::new(nrx3),
            active: false,
        }
    }

    #[inline]
    pub(crate) fn active(&self) -> bool {
        self.active
    }

    #[inline]
    pub(crate) fn step(&mut self, frame: Option<Frame>) {
        if let Some(use_volume) = self.lfsr.step(self.nrx3) {
            let volume =
                if use_volume && (self.active()) { self.envelope.volume() as i32 } else { 0 };
            if let Some(blipbuf) = &mut self.blipbuf {
                blipbuf.add_delta(self.lfsr.clock.div(), volume);
            }
        }

        if let Some(frame) = frame {
            if self.active {
                self.envelope.step(frame);
            }
            self.length_counter.step(frame);
        }

        self.active &= self.length_counter.active();
    }

    pub(crate) fn read_samples(&mut self, buffer: &mut [i16], duration: u32) -> usize {
        self.blipbuf.as_mut().map_or(0, |blipbuf| blipbuf.end(buffer, duration))
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

                self.envelope.set_nrx2(value);
                self.active &= self.envelope.dac_on();
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
                    self.envelope.trigger();
                    self.lfsr = Lfsr::new(self.nrx3);
                }
                self.active = self.length_counter.active();
                self.active &= self.envelope.dac_on();

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

#[derive(Serialize, Deserialize)]
pub(crate) struct NoiseChannelSnapshot {
    nrx1: u8,
    nrx2: u8,
    nrx3: u8,
    nrx4: u8,
    length_counter: LengthCounter,
    envelope: Envelope,
    lfsr: Lfsr,
    active: bool,
}

impl Snapshot for NoiseChannel {
    type Snapshot = NoiseChannelSnapshot;

    fn take_snapshot(&self) -> Self::Snapshot {
        NoiseChannelSnapshot {
            nrx1: self.nrx1,
            nrx2: self.nrx2,
            nrx3: self.nrx3,
            nrx4: self.nrx4,
            length_counter: self.length_counter.clone(),
            envelope: self.envelope.clone(),
            lfsr: self.lfsr.clone(),
            active: self.active,
        }
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        self.nrx1 = snapshot.nrx1;
        self.nrx2 = snapshot.nrx2;
        self.nrx3 = snapshot.nrx3;
        self.nrx4 = snapshot.nrx4;
        self.length_counter = snapshot.length_counter;
        self.envelope = snapshot.envelope;
        self.lfsr = snapshot.lfsr;
        self.active = snapshot.active;

        if let Some(blipbuf) = &mut self.blipbuf {
            blipbuf.clear();
        }
    }
}
