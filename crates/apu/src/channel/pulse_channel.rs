use gb_shared::{is_bit_set, Memory};

use crate::{
    blipbuf, clock::Clock, frame_sequencer::FrameSequencer,
    length_counter::PulseChannelLengthCounter as LengthCounter,
};

use super::VolumeEnvelope;

struct PulseChannelClock(Clock);

impl PulseChannelClock {
    fn new_clock(period: u16) -> Clock {
        debug_assert!(period <= 2047);
        // CPU_FREQ / (1048576 / (2048 - period_value as u32))
        Clock::new(4 * (2048 - period as u32))
    }
    #[inline]
    fn from_period(period: u16) -> Self {
        Self(Self::new_clock(period))
    }

    #[inline]
    fn from_nrxs(nrx3: u8, nrx4: u8) -> Self {
        Self::from_period(((nrx4 as u16 & 0b111) << 8) | (nrx3 as u16))
    }

    fn reload(&mut self, nrx3: Option<u8>, nrx4: Option<u8>) {
        let nrx3 = nrx3.unwrap_or_else(|| self.0.div() as u8) as u16;
        let nrx4 = nrx4.unwrap_or_else(|| (self.0.div() >> 8) as u8) as u16;
        self.reload_with_period(((nrx4 & 0b111) << 8) | nrx3);
    }

    fn reload_with_period(&mut self, period: u16) {
        self.0 = Self::new_clock(period);
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
    blipbuf: blipbuf::BlipBuf,
    channel_clock: PulseChannelClock,
    length_counter: LengthCounter,
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
    fs: FrameSequencer,
    /// Complete one iteration when it reaches zero.
    /// Initialized and reset with `pace`.
    steps: u8,
    /// Control period sweep clock frequency.
    pace: u8,
    /// Used to calculate next period value.
    dir_decrease: bool,
    /// Used to calculate next period value.
    shift: u8,
    period_value: u16,
    /// Writes to NR13 and NR14 will be stored here and
    /// will be used to override `shadow_period_value` on
    /// triggered. And it can be overriden by `period_value`
    /// on next period sweep update.
    nrx34: u16,
    enabled: bool,
    overflow: bool,
}

impl PeriodSweep {
    fn parse_nrx0(nrx0: u8) -> (u8, bool, u8) {
        let pace = {
            let mut pace = (nrx0 >> 4) & 0b111;
            if pace == 0 {
                pace = 8;
            }
            pace
        };
        let dir_decrease = is_bit_set!(nrx0, 3);
        let shift = nrx0 & 0b111;
        (pace, dir_decrease, shift)
    }

    fn new(nrx0: u8, nrx3: u8, nrx4: u8) -> Self {
        let (pace, dir_decrease, shift) = Self::parse_nrx0(nrx0);

        let period_value = period_value(nrx3, nrx4);
        Self {
            fs: FrameSequencer::new(),
            steps: pace,
            pace,
            dir_decrease,
            shift,
            period_value,
            nrx34: period_value,
            enabled: false,
            overflow: false,
        }
    }
}

impl PeriodSweep {
    #[inline]
    fn overflow(&self) -> bool {
        self.overflow
    }

    fn period_value(&self) -> u16 {
        self.period_value
    }

    fn set_nrx0(&mut self, nrx0: u8) {
        let (pace, dir_decrease, shift) = Self::parse_nrx0(nrx0);
        self.pace = pace;
        self.dir_decrease = dir_decrease;
        self.shift = shift;
    }

    fn set_nrx3(&mut self, nrx3: u8) {
        let lo = nrx3 as u16;
        self.nrx34 = (self.nrx34 & 0x700) | lo;
    }

    fn set_nrx4(&mut self, nrx4: u8) {
        let hi = (nrx4 as u16 & 0x7) << 8;
        self.nrx34 = (self.nrx34 & 0xFF) | hi;
    }

    fn calculate_next_period_value(period_value: u16, dir_decrease: bool, shift: u8) -> u16 {
        if period_value == 0 {
            return 0;
        }

        let delta = period_value >> shift;
        if dir_decrease {
            // delta won't > shadow_period_value,
            // so if it's subtracting, it won't underflow 0.
            period_value.saturating_sub(delta)
        } else {
            period_value.wrapping_add(delta)
        }
    }

    pub(crate) fn trigger(&mut self) {
        self.period_value = self.nrx34;
        self.steps = self.pace;
        self.enabled = self.pace != 8 || self.shift != 0;
        self.overflow = false;

        if self.shift != 0 {
            let new_period_value =
                Self::calculate_next_period_value(self.period_value, self.dir_decrease, self.shift);
            self.overflow = new_period_value > 2047;
            if !self.overflow {
                self.period_value = new_period_value;
                self.nrx34 = new_period_value;
            }
        }
    }

    fn step(&mut self) -> Option<()> {
        if let Some(step) = self.fs.step() {
            if self.enabled && (step == 2 || step == 6) {
                self.steps = self.steps.saturating_sub(1);
                if self.steps == 0 {
                    log::debug!("calculate pace {}", self.pace);
                    if self.pace != 8 {
                        let new_period_value = Self::calculate_next_period_value(
                            self.period_value,
                            self.dir_decrease,
                            self.shift,
                        );
                        self.overflow = new_period_value > 2047;
                        if !self.overflow && self.shift != 0 {
                            // Writen back
                            self.period_value = new_period_value;
                            self.nrx34 = new_period_value;

                            // AGAIN
                            let new_period_value = Self::calculate_next_period_value(
                                new_period_value,
                                self.dir_decrease,
                                self.shift,
                            );
                            self.overflow = new_period_value > 2047;
                        }
                    }
                    self.steps = self.pace;
                }
            }
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
        // TODO: 2 kinds of sweep for CH1 and CH2
        let channel_clock = if with_period_sweep {
            PulseChannelClock::from_period(period_sweep.period_value())
        } else {
            PulseChannelClock::from_nrxs(nrx3, nrx4)
        };

        Self {
            blipbuf: blipbuf::BlipBuf::new(frequency, sample_rate, volume_envelope.volume() as i32),
            channel_clock,
            length_counter: LengthCounter::new_expired(),
            nrx0,
            nrx1,
            nrx2,
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
            if period_sweep.step().is_some() {
                self.channel_clock.reload_with_period(period_sweep.period_value());
            }
        }

        self.volume_envelope.step();
        self.length_counter.step();

        self.active &= self.length_counter.active();
        self.active &= self.period_sweep.as_ref().map_or(true, |s| !s.overflow());
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
                if let Some(period_sweep) = self.period_sweep.as_mut() {
                    period_sweep.set_nrx0(value);
                }
            }
            1 => {
                self.length_counter.set_len(value & 0x3F);
                self.nrx1 = value;
            }
            2 => {
                self.nrx2 = value;
                self.volume_envelope.set_nrx2(value);

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
                        period_sweep.set_nrx3(value);
                        self.channel_clock.reload_with_period(period_sweep.period_value());
                    }
                    None => {
                        self.channel_clock.reload(Some(value), None);
                    }
                };
            }
            4 => {
                match self.period_sweep.as_mut() {
                    Some(period_sweep) => {
                        period_sweep.set_nrx4(value);
                        self.channel_clock.reload_with_period(period_sweep.period_value());
                    }
                    None => {
                        self.channel_clock.reload(None, Some(value));
                    }
                };

                log::debug!(
                    "{} CH{} length",
                    if is_bit_set!(value, 6) { "enable" } else { "disable" },
                    if ch1 { 1 } else { 2 }
                );
                self.length_counter.set_enabled(value);

                // Trigger the channel
                if is_bit_set!(value, 7) {
                    log::debug!("CH{} trigger", if self.period_sweep.is_some() { 1 } else { 2 });
                    self.length_counter.trigger();

                    if let Some(period_sweep) = self.period_sweep.as_mut() {
                        period_sweep.trigger();
                        self.channel_clock.reload_with_period(period_sweep.period_value());
                    }

                    self.volume_envelope = VolumeEnvelope::new(self.nrx2);
                    self.blipbuf.clear();
                }

                self.active = self.dac_on();
                self.active &= self.length_counter.active();
                self.active &= self.period_sweep.as_ref().map_or(true, |s| !s.overflow());

                log::info!("CH{} active: {}", if ch1 { 1 } else { 2 }, self.active);
            }
            _ => unreachable!("Invalid address for PulseChannel: {:#X}", addr),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0 => self.nrx0,
            1 => self.nrx1,
            2 => self.nrx2,
            3 => 0,
            4 => (self.length_counter.enabled() as u8) << 6,
            _ => unreachable!("Invalid address for PulseChannel: {:#X}", addr),
        }
    }
}
