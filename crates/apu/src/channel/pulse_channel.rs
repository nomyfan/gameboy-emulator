use gb_shared::{is_bit_set, Memory};

use crate::{blipbuf, clock::Clock, length_timer::LengthTimer, utils::freq_to_clock_cycles};

/// How many CPU clock cycles to produce a sample.
#[inline]
pub(crate) fn pulse_channel_sample_period(period_value: u16) -> u32 {
    debug_assert!(period_value <= 2047);

    // CPU_FREQ / (1048576 / (2048 - period_value as u32))
    4 * (2048 - period_value as u32)
}

struct DutyCycle {
    index: u8,
}

impl DutyCycle {
    fn new() -> Self {
        Self { index: 0 }
    }

    /// We do not handle the case where duty cycle
    /// get changed during one cycle.
    /// Return true if the signal is high.
    fn next(&mut self, nrx1: u8) -> bool {
        let waveform = match (nrx1 >> 6) & 0b11 {
            // 12.5%
            0b00 => 0b1111_1110,
            // 25%
            0b01 => 0b0111_1110,
            // 50%
            0b10 => 0b0111_1000,
            // 75%
            0b11 => 0b1000_0001,
            _ => unreachable!(),
        };
        let signal = (waveform >> self.index) & 1 == 1;
        self.index = (self.index + 1) % 8;

        signal
    }
}

pub(crate) struct PulseChannel {
    blipbuf: blipbuf::BlipBuf,
    channel_clock: Clock,
    // TODO: retrigger will reset
    length_timer: LengthTimer,
    /// Sweep register.
    nrx0: u8,
    /// Sound length/Wave pattern duty.
    nrx1: u8,
    /// Volume envelope.
    nrx2: u8,
    /// Period lo.
    /// The low 8 bits of the period value.
    nrx3: u8,
    /// Period hi and control.
    /// Bit 7: Trigger.
    /// Bit 6: Length enable.
    /// Bit 2..=0: The upper 3 bits of the period value.
    nrx4: u8,
    duty_cycle: DutyCycle,
    volume_envelope: VolumeEnvelope,
    period_sweep: PeriodSweep,
}

#[inline]
fn period_value(nrx3: u8, nrx4: u8) -> u16 {
    ((nrx4 as u16 & 0b111) << 8) | (nrx3 as u16)
}

const PERIOD_SWEEP_CYCLES: u32 = freq_to_clock_cycles(128);

struct PeriodSweep {
    clock: Clock,
    period_value: u16,
}

impl PeriodSweep {
    fn new_period_sweep_clock(nrx0: u8) -> Clock {
        let pace = (nrx0 >> 4) & 0b111;
        Clock::new(PERIOD_SWEEP_CYCLES * pace as u32)
    }

    fn new(nrx0: u8, nrx3: u8, nrx4: u8) -> Self {
        let period_value = period_value(nrx3, nrx4);
        Self { clock: Self::new_period_sweep_clock(nrx0), period_value }
    }

    #[inline]
    fn overflow(&self) -> bool {
        self.period_value > 0x7FF
    }

    fn set_clock(&mut self, nrx0: u8) {
        self.clock = Self::new_period_sweep_clock(nrx0);
    }

    fn set_period_value(&mut self, nrx3: u8, nrx4: u8) {
        self.period_value = period_value(nrx3, nrx4);
    }

    fn period_value(&self) -> u16 {
        self.period_value
    }

    fn next_period_value(period_value: u16, nrx0: u8) -> u16 {
        let direction = (nrx0 >> 3) & 0b1;
        let step = nrx0 & 0b111;

        if direction == 1 {
            ((period_value as i16) - ((period_value / 2u16.pow(step as u32)) as i16)) as u16
        } else {
            period_value + (period_value / 2u16.pow(step as u32))
        }
    }

    fn next(&mut self, nrx0: u8) -> Option<(u8, u8)> {
        if self.clock.next() {
            self.period_value = Self::next_period_value(self.period_value, nrx0);
            if !self.overflow() {
                let lo = self.period_value as u8;
                let hi = ((self.period_value >> 8) as u8) & 0b111;

                return Some((lo, hi));
            }

            self.clock = Self::new_period_sweep_clock(nrx0);
        }

        None
    }
}

const VOLUME_ENVELOPE_CYCLES: u32 = freq_to_clock_cycles(64);

struct VolumeEnvelope {
    clock: Clock,
    volume: u8,
}

impl VolumeEnvelope {
    fn new(nrx2: u8) -> Self {
        let pace = nrx2 & 0b111;
        let init_volume = (nrx2 >> 4) & 0xF;
        Self { clock: Clock::new(VOLUME_ENVELOPE_CYCLES * pace as u32), volume: init_volume }
    }

    fn next(&mut self, nrx2: u8) {
        if self.clock.next() {
            if is_bit_set!(nrx2, 3) {
                self.volume = self.volume.wrapping_sub(1);
            } else {
                self.volume = self.volume.wrapping_add(1);
            }
        }
    }
}

impl PulseChannel {
    #[inline]
    fn new_channel_clock(period_value: u16) -> Clock {
        Clock::new(pulse_channel_sample_period(period_value))
    }

    /// Create CH1(left) and CH2(right).
    pub(crate) fn new_chs(frequency: u32, sample_rate: u32) -> (Self, Self) {
        let nrx0 = 0x80;
        let nrx3 = 0xFF;
        let nrx4 = 0xBF;

        let new_channel = |nrx2: u8| {
            let period_sweep = PeriodSweep::new(nrx0, nrx3, nrx4);
            Self {
                blipbuf: blipbuf::new(frequency, sample_rate),
                channel_clock: Self::new_channel_clock(period_sweep.period_value()),
                length_timer: LengthTimer::new(0x3F),
                nrx0,
                nrx1: 0xBF,
                nrx2,
                nrx3,
                nrx4,
                duty_cycle: DutyCycle::new(),
                volume_envelope: VolumeEnvelope::new(nrx2),
                period_sweep,
            }
        };

        (new_channel(0xF3), new_channel(0x00))
    }

    #[inline]
    fn dac_off(&self) -> bool {
        (self.nrx2 >> 3) == 0
    }

    #[inline]
    fn triggered(&self) -> bool {
        is_bit_set!(self.nrx4, 7)
    }

    /// Return `true` if the channel is active.
    pub(crate) fn active(&self) -> bool {
        // Any condition below satisfied will deactivate the channel.
        // - DAC is off.
        // - Length timer expired.
        // - Period overflowed.
        !(self.dac_off() || self.length_timer.expired() || self.period_sweep.overflow())
    }

    pub(crate) fn next(&mut self) {
        // TODO: confirm should channel continue working when deactivated.
        if self.channel_clock.next() {
            if self.active() {
                let is_high_signal = self.duty_cycle.next(self.nrx1);
                // TODO: generate sample data
                unimplemented!()
            } else {
                // TODO: if it's deactivated, generate 0
                unimplemented!()
            }
        }

        if let Some((lo, hi)) = self.period_sweep.next(self.nrx0) {
            self.nrx3 = lo;
            self.nrx4 = (self.nrx4 & (!0b111)) | hi;
            self.channel_clock = Self::new_channel_clock(self.period_sweep.period_value());
        }

        self.volume_envelope.next(self.nrx2);

        self.length_timer.next();
    }
}

impl Memory for PulseChannel {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0 => {
                self.nrx0 = value;
                self.period_sweep.set_clock(self.nrx0);
            }
            1 => {
                self.nrx1 = value;
                self.length_timer = LengthTimer::new(self.nrx1 & 0x3F);
            }
            2 => {
                // Writes to this register while the channel is on require retriggering it afterwards. If the write turns the channel off, retriggering is not necessary (it would do nothing).
                // @see https://gbdev.io/pandocs/Audio_Registers.html#ff20--nr41-channel-4-length-timer-write-only:~:text=writes%20to%20this%20register%20while%20the%20channel%20is%20on%20require%20retriggering%20it%20afterwards.%20if%20the%20write%20turns%20the%20channel%20off%2C%20retriggering%20is%20not%20necessary%20(it%20would%20do%20nothing).
                self.nrx2 = value;
            }
            3 => {
                self.nrx3 = value;
                // Period changes (written to NR13 or NR14) only take effect after the current “sample” ends.
                // @see https://gbdev.io/pandocs/Audio_Registers.html#ff20--nr41-channel-4-length-timer-write-only:~:text=period%20changes%20(written%20to%20nr13%20or%20nr14)%20only%20take%20effect%20after%20the%20current%20%E2%80%9Csample%E2%80%9D%20ends
                self.period_sweep.set_period_value(self.nrx3, self.nrx4);
                self.channel_clock = Self::new_channel_clock(self.period_sweep.period_value());
            }
            4 => {
                self.nrx4 = value;

                // Period changes (written to NR13 or NR14) only take effect after the current “sample” ends.
                // @see https://gbdev.io/pandocs/Audio_Registers.html#ff20--nr41-channel-4-length-timer-write-only:~:text=period%20changes%20(written%20to%20nr13%20or%20nr14)%20only%20take%20effect%20after%20the%20current%20%E2%80%9Csample%E2%80%9D%20ends
                self.period_sweep.set_period_value(self.nrx3, self.nrx4);
                self.channel_clock = Self::new_channel_clock(self.period_sweep.period_value());

                if self.triggered() && !self.dac_off() {
                    self.length_timer = if is_bit_set!(value, 6) {
                        LengthTimer::new(self.nrx1 & 0x3F)
                    } else {
                        LengthTimer::new_expired()
                    };

                    self.volume_envelope = VolumeEnvelope::new(self.nrx2);
                    self.period_sweep = PeriodSweep::new(self.nrx0, self.nrx3, self.nrx4);
                    self.channel_clock = Self::new_channel_clock(self.period_sweep.period_value());
                }
            }
            _ => unreachable!("Invalid address: {}", addr),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0 => self.nrx0,
            1 => self.nrx1,
            2 => self.nrx2,
            3 => self.nrx3,
            4 => self.nrx4,
            _ => unreachable!("Invalid address: {}", addr),
        }
    }
}
