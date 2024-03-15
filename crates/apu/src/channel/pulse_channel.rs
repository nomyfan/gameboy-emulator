use gb_shared::{is_bit_set, Memory};

use crate::{blipbuf, clock::Clock};

use super::{Frame, PeriodSweep, PulseChannelLengthCounter as LengthCounter, VolumeEnvelope};

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

    fn reload(&mut self, period: u16) {
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

pub(crate) struct PulseChannel<SWEEP>
where
    SWEEP: PeriodSweep,
{
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
    period_sweep: SWEEP,
    /// Indicates if the channel is working.
    /// The only case where setting it to `true` is triggering the channel.
    /// When length timer expires, it is set to `false`.
    /// When the sweep overflows, it is set to `false`.
    /// Default is `false`.
    active: bool,
}

impl<SWEEP> std::fmt::Debug for PulseChannel<SWEEP>
where
    SWEEP: PeriodSweep,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PulseChannel")
            .field("length_counter", &self.length_counter)
            // .field("volume_envelope", &self.volume_envelope) // TODO: Implement Debug for VolumeEnvelope
            .field("period_sweep", &self.period_sweep)
            .field("active", &self.active)
            .finish()
    }
}

impl<SWEEP> PulseChannel<SWEEP>
where
    SWEEP: PeriodSweep,
{
    pub(crate) fn new(frequency: u32, sample_rate: u32) -> Self {
        let nrx0 = 0;
        let nrx1 = 0;
        let nrx2 = 0;
        let nrx3 = 0;
        let nrx4 = 0;

        let period_sweep = SWEEP::new(nrx0, nrx3, nrx4);
        let volume_envelope = VolumeEnvelope::new(nrx2);
        let channel_clock = PulseChannelClock::from_period(period_sweep.period_value());

        Self {
            blipbuf: blipbuf::BlipBuf::new(frequency, sample_rate, volume_envelope.volume() as i32),
            channel_clock,
            length_counter: LengthCounter::new_expired(),
            nrx0,
            nrx1,
            nrx2,
            duty_cycle: DutyCycle::new(),
            volume_envelope,
            period_sweep,
            active: false,
        }
    }
}

impl<SWEEP> PulseChannel<SWEEP>
where
    SWEEP: PeriodSweep,
{
    #[inline]
    pub(crate) fn on(&self) -> bool {
        self.active
    }

    #[inline]
    fn dac_on(&self) -> bool {
        (self.nrx2 & 0xF8) != 0
    }

    pub(crate) fn step(&mut self, frame: Option<Frame>) {
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

        if let Some(frame) = frame {
            if self.period_sweep.step(frame) {
                self.channel_clock.reload(self.period_sweep.period_value());
            }

            self.volume_envelope.step(frame);
            self.length_counter.step(frame);
        }

        self.active &= self.length_counter.active();
        self.active &= self.period_sweep.active();
    }

    pub(crate) fn read_samples(&mut self, buffer: &mut [i16], duration: u32) {
        self.blipbuf.end(buffer, duration)
    }

    /// Called when the APU is turned off which resets all registers.
    pub(crate) fn power_off(&mut self) {
        self.write(0, 0);
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

impl<SWEEP> Memory for PulseChannel<SWEEP>
where
    SWEEP: PeriodSweep,
{
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0 => {
                self.nrx0 = value;
                self.period_sweep.set_nrx0(value);
            }
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
                self.period_sweep.set_nrx3(value);
                // self.channel_clock.reload(self.period_sweep.period_value());
            }
            4 => {
                self.period_sweep.set_nrx4(value);
                // self.channel_clock.reload(self.period_sweep.period_value());
                self.length_counter.set_enabled(value);

                // Trigger the channel
                if is_bit_set!(value, 7) {
                    self.length_counter.trigger();

                    self.period_sweep.trigger();
                    self.channel_clock.reload(self.period_sweep.period_value());

                    self.volume_envelope = VolumeEnvelope::new(self.nrx2);
                    self.blipbuf.clear();
                }

                self.active = self.dac_on();
                self.active &= self.length_counter.active();
                self.active &= self.period_sweep.active();
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
