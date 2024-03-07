use gb_shared::{is_bit_set, Memory};

use crate::{
    blipbuf, clock::Clock, length_timer::PulseChannelLengthTimer as LengthTimer,
    utils::freq_to_clock_cycles,
};

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
    /// Period sweep. Channel 2 lacks this feature.
    /// Bit0..=2, individual step. Used to calculate next period value.
    /// Bit3, direction. 0: increase, 1: decrease.
    /// Bit4..=6, pace. Control period sweep clock frequency.
    nrx0: u8,
    /// Sound length/Wave pattern duty.
    /// Bit0..=5, initial length timer. Used to set length timer's length. Write-only.
    /// Bit6..=7, wave duty.
    nrx1: u8,
    /// Volume envelope.
    /// Bit0..=2, pace. Control volume envelope clock frequency. 0 disables the envelope.
    /// Bit3, direction. 0: decrease, 1: increase.
    /// Bit4..=7, initial volume. Used to set volume envelope's volume.
    /// When Bit3..=7 are all 0, the DAC is off.
    nrx2: u8,
    /// Period lo.
    /// The low 8 bits of the period value.
    ///
    /// Period changes (written to NR13 or NR14) only take effect after the current “sample” ends; see description above.
    /// @see https://gbdev.io/pandocs/Audio_Registers.html#ff11--nr11-channel-1-length-timer--duty-cycle:~:text=period%20changes%20(written%20to%20nr13%20or%20nr14)%20only%20take%20effect%20after%20the%20current%20%E2%80%9Csample%E2%80%9D%20ends%3B%20see%20description%20above.
    nrx3: u8,
    /// Period hi and control.
    /// Bit 2..=0: The upper 3 bits of the period value. Write-only.
    ///
    /// Period changes (written to NR13 or NR14) only take effect after the current “sample” ends; see description above.
    /// @see https://gbdev.io/pandocs/Audio_Registers.html#ff11--nr11-channel-1-length-timer--duty-cycle:~:text=period%20changes%20(written%20to%20nr13%20or%20nr14)%20only%20take%20effect%20after%20the%20current%20%E2%80%9Csample%E2%80%9D%20ends%3B%20see%20description%20above.
    ///
    /// Bit 6: Length enable.
    /// Bit 7: Trigger. Write-only.
    nrx4: u8,
    blipbuf: blipbuf::BlipBuf,
    channel_clock: PulseChannelClock,
    length_timer: LengthTimer,
    duty_cycle: DutyCycle,
    volume_envelope: VolumeEnvelope,
    period_sweep: Option<PeriodSweep>,
    /// Indicates if the channel is working.
    /// The only case where setting it to `true` is triggering the channel.
    /// When length timer expires, it is set to `false`.
    /// When the sweep overflows, it is set to `false`.
    /// Default is `false`.
    active: bool,
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
            if !self.overflow() {
                self.period_value = Self::next_period_value(self.period_value, nrx0);
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
            length_timer: LengthTimer::new_expired(),
            nrx0,
            nrx1,
            nrx2,
            nrx3,
            nrx4,
            duty_cycle: DutyCycle::new(),
            volume_envelope,
            period_sweep: if with_period_sweep { Some(period_sweep) } else { None },
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

    pub(crate) fn step(&mut self) {
        if self.channel_clock.step() {
            if self.on() {
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
        self.length_timer.step();

        self.active &= self.length_timer.active()
            && self.period_sweep.as_ref().map_or(true, |s| !s.overflow());
    }

    pub(crate) fn read_samples(&mut self, buffer: &mut [i16], duration: u32) {
        self.blipbuf.end(buffer, duration)
    }

    /// Called when the APU is turned off which resets all registers.
    pub(crate) fn turn_off(&mut self) {
        for addr in 0..=4 {
            self.write(addr, 0);
        }
    }
}

impl Memory for PulseChannel {
    fn write(&mut self, addr: u16, value: u8) {
        let ch1 = self.period_sweep.is_some();
        log::debug!("Write to NR{}{}: {:#X}", if ch1 { 1 } else { 2 }, addr, value);

        match addr {
            0 => {
                self.nrx0 = value;
            }
            1 => {
                self.length_timer.set_len(value & 0x3F);
                self.nrx1 = value;
            }
            2 => {
                self.nrx2 = value;

                log::debug!(
                    "CH{} dac {}",
                    if ch1 { 1 } else { 2 },
                    if self.dac_on() { "on" } else { "off" }
                );
                self.active &= self.dac_on();
            }
            3 => {
                // TODO: extract this logic to a function
                // TODO: update channel clock only? Should sweep get updated when trigger?
                match self.period_sweep.as_mut() {
                    Some(period_sweep) => {
                        period_sweep.set_period_value(value, self.nrx4);
                        self.channel_clock =
                            PulseChannelClock::from_period(period_sweep.period_value());
                    }
                    None => {
                        self.channel_clock = PulseChannelClock::from_nrxs(value, self.nrx4);
                    }
                };

                self.nrx3 = value;
            }
            4 => {
                // TODO: update channel clock only? Should sweep get updated when trigger?
                match self.period_sweep.as_mut() {
                    Some(period_sweep) => {
                        period_sweep.set_period_value(self.nrx3, value);
                        self.channel_clock =
                            PulseChannelClock::from_period(period_sweep.period_value());
                    }
                    None => {
                        self.channel_clock = PulseChannelClock::from_nrxs(self.nrx3, value);
                    }
                };

                log::debug!(
                    "{} CH{} length",
                    if is_bit_set!(value, 6) { "enable" } else { "disable" },
                    if ch1 { 1 } else { 2 }
                );
                self.length_timer.set_enabled(value);

                // Trigger the channel
                if is_bit_set!(value, 7) {
                    log::debug!("CH{} trigger", if self.period_sweep.is_some() { 1 } else { 2 });
                    self.length_timer.reset_len();
                    self.volume_envelope = VolumeEnvelope::new(self.nrx2);
                    self.blipbuf.clear();

                    // TODO: Should sweep get updated when trigger?
                }

                self.active = self.length_timer.active()
                    && self.period_sweep.as_ref().map_or(true, |s| !s.overflow());
                self.active &= self.dac_on();
                log::info!("CH{} active: {}", if ch1 { 1 } else { 2 }, self.active);

                self.nrx4 = value;
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
