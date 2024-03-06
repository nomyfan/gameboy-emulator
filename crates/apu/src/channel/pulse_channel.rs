use gb_shared::{is_bit_set, Memory};

use crate::{blipbuf, clock::Clock, length_timer::LengthTimer, utils::freq_to_clock_cycles};

use super::VolumeEnvelope;

struct PulseChannelClock(Clock);

impl PulseChannelClock {
    #[inline]
    fn from_period(period: u16) -> Self {
        debug_assert!(period <= 2047);
        // CPU_FREQ / (1048576 / (2048 - period_value as u32))
        Self(Clock::new(4 * (2048 - period as u32)))
    }

    #[inline]
    fn from_nrxs(nrx3: u8, nrx4: u8) -> Self {
        Self::from_period(((nrx4 as u16 & 0b111) << 8) | (nrx3 as u16))
    }

    #[inline(always)]
    fn step(&mut self) -> bool {
        self.0.step()
    }

    #[inline]
    fn div(&self) -> u32 {
        self.0.div()
    }
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
    fn step(&mut self, nrx1: u8) -> bool {
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
    channel_clock: PulseChannelClock,
    length_timer: Option<LengthTimer>,
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
    period_sweep: Option<PeriodSweep>,
}

#[inline]
fn period_value(nrx3: u8, nrx4: u8) -> u16 {
    ((nrx4 as u16 & 0b111) << 8) | (nrx3 as u16)
}

struct PeriodSweep {
    clock: Clock,
    period_value: u16,
}

impl PeriodSweep {
    fn new_sweep_clock(nrx0: u8) -> Clock {
        const PERIOD_SWEEP_CYCLES: u32 = freq_to_clock_cycles(128);
        let pace = (nrx0 >> 4) & 0b111;
        Clock::new(PERIOD_SWEEP_CYCLES * pace as u32)
    }

    fn new(nrx0: u8, nrx3: u8, nrx4: u8) -> Self {
        let period_value = period_value(nrx3, nrx4);
        Self { clock: Self::new_sweep_clock(nrx0), period_value }
    }
}

impl PeriodSweep {
    #[inline]
    fn overflow(&self) -> bool {
        self.period_value > 0x7FF
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

        let delta = period_value / 2u16.pow(step as u32);

        if direction == 1 {
            period_value.wrapping_sub(delta)
        } else {
            period_value.wrapping_add(delta)
        }
    }

    fn step(&mut self, nrx0: u8) -> Option<(u8, u8)> {
        if self.clock.step() {
            self.period_value = Self::next_period_value(self.period_value, nrx0);
            if !self.overflow() {
                let lo = self.period_value as u8;
                let hi = ((self.period_value >> 8) as u8) & 0b111;

                return Some((lo, hi));
            }

            self.clock = Self::new_sweep_clock(nrx0);
        }

        None
    }
}

impl PulseChannel {
    pub(crate) fn new(frequency: u32, sample_rate: u32, with_period_sweep: bool) -> Self {
        let nrx0 = 0;
        let nrx1 = 0;
        let nrx2 = 0;
        let nrx3 = 0;
        let nrx4 = 0;

        let period_sweep = PeriodSweep::new(nrx0, nrx3, nrx4);
        let volume_envelope = VolumeEnvelope::new(nrx2);
        Self {
            blipbuf: blipbuf::BlipBuf::new(frequency, sample_rate, volume_envelope.volume() as i32),
            channel_clock: PulseChannelClock::from_period(period_sweep.period_value()),
            length_timer: None,
            nrx0,
            nrx1,
            nrx2,
            nrx3,
            nrx4,
            duty_cycle: DutyCycle::new(),
            volume_envelope,
            period_sweep: if with_period_sweep { Some(period_sweep) } else { None },
        }
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
        let length_timer_expired = self.length_timer.as_ref().map_or(false, |lt| lt.expired());
        let period_overflow = self.period_sweep.as_ref().map_or(false, |s| s.overflow());

        !self.dac_off() && !length_timer_expired && !period_overflow
    }

    pub(crate) fn step(&mut self) {
        if self.channel_clock.step() {
            if self.active() {
                let is_high_signal = self.duty_cycle.step(self.nrx1);
                let volume = self.volume_envelope.volume() as i32;
                let volume = if is_high_signal { volume } else { -volume };
                self.blipbuf.add_delta(self.channel_clock.div(), volume);
            } else {
                self.blipbuf.add_delta(self.channel_clock.div(), 0);
            }
        }

        if let Some(period_sweep) = self.period_sweep.as_mut() {
            if let Some((lo, hi)) = period_sweep.step(self.nrx0) {
                self.nrx3 = lo;
                self.nrx4 = (self.nrx4 & (!0b111)) | hi;
                self.channel_clock = PulseChannelClock::from_period(period_sweep.period_value());
            }
        }

        self.volume_envelope.step(self.nrx2);

        if let Some(length_timer) = self.length_timer.as_mut() {
            length_timer.step();
        }
    }

    pub(crate) fn read_samples(&mut self, buffer: &mut [i16], duration: u32) {
        self.blipbuf.end(buffer, duration)
    }
}

impl Memory for PulseChannel {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0 => {
                self.nrx0 = value;
            }
            1 => {
                self.nrx1 = value;
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
                match self.period_sweep.as_mut() {
                    Some(period_sweep) => {
                        period_sweep.set_period_value(self.nrx3, self.nrx4);
                        self.channel_clock =
                            PulseChannelClock::from_period(period_sweep.period_value());
                    }
                    None => {
                        self.channel_clock = PulseChannelClock::from_nrxs(self.nrx3, self.nrx4);
                    }
                };
            }
            4 => {
                self.nrx4 = value;

                // Period changes (written to NR13 or NR14) only take effect after the current “sample” ends.
                // @see https://gbdev.io/pandocs/Audio_Registers.html#ff20--nr41-channel-4-length-timer-write-only:~:text=period%20changes%20(written%20to%20nr13%20or%20nr14)%20only%20take%20effect%20after%20the%20current%20%E2%80%9Csample%E2%80%9D%20ends
                match self.period_sweep.as_mut() {
                    Some(period_sweep) => {
                        period_sweep.set_period_value(self.nrx3, self.nrx4);
                        self.channel_clock =
                            PulseChannelClock::from_period(period_sweep.period_value());
                    }
                    None => {
                        self.channel_clock = PulseChannelClock::from_nrxs(self.nrx3, self.nrx4);
                    }
                };

                if self.triggered() && !self.dac_off() {
                    self.length_timer = if is_bit_set!(self.nrx4, 6) {
                        Some(LengthTimer::new(self.nrx1 & 0x3F))
                    } else {
                        None
                    };

                    self.volume_envelope = VolumeEnvelope::new(self.nrx2);

                    if self.period_sweep.is_some() {
                        let period_sweep = PeriodSweep::new(self.nrx0, self.nrx3, self.nrx4);
                        self.channel_clock =
                            PulseChannelClock::from_period(period_sweep.period_value());
                        self.period_sweep = Some(period_sweep);
                    } else {
                        self.channel_clock = PulseChannelClock::from_nrxs(self.nrx3, self.nrx4);
                    }
                }
            }
            _ => unreachable!("Invalid address for PulseChannel: {:#X}", addr),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0 => self.nrx0,
            1 => self.nrx1,
            2 => self.nrx2,
            3 => self.nrx3,
            4 => self.nrx4,
            _ => unreachable!("Invalid address for PulseChannel: {:#X}", addr),
        }
    }
}
