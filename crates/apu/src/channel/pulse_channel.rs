use gb_shared::{is_bit_set, Memory, Snapshot};
use serde::{Deserialize, Serialize};

use crate::{blipbuf, clock::Clock};

use super::{Envelope, Frame, PulseChannelLengthCounter as LengthCounter, Sweep};

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

#[derive(Clone, Copy, Serialize, Deserialize)]
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
    SWEEP: Sweep,
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
    blipbuf: Option<blipbuf::BlipBuf>,
    channel_clock: PulseChannelClock,
    length_counter: LengthCounter,
    duty_cycle: DutyCycle,
    envelope: Envelope,
    sweep: SWEEP,
    /// Indicates if the channel is working.
    /// The only case where setting it to `true` is triggering the channel.
    /// When length timer expires, it is set to `false`.
    /// When the sweep overflows, it is set to `false`.
    /// Default is `false`.
    active: bool,
}

impl<SWEEP> std::fmt::Debug for PulseChannel<SWEEP>
where
    SWEEP: Sweep,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PulseChannel")
            .field("length_counter", &self.length_counter)
            .field("envelope", &self.envelope)
            .field("sweep", &self.sweep)
            .field("active", &self.active)
            .finish()
    }
}

impl<SWEEP> PulseChannel<SWEEP>
where
    SWEEP: Sweep,
{
    pub(crate) fn new(frequency: u32, sample_rate: Option<u32>) -> Self {
        let nrx0 = 0;
        let nrx1 = 0;
        let nrx2 = 0;
        let nrx3 = 0;
        let nrx4 = 0;

        let sweep = SWEEP::new(nrx0, nrx3, nrx4);
        let envelope = Envelope::new(nrx2);
        let channel_clock = PulseChannelClock::from_period(sweep.period_value());

        Self {
            blipbuf: sample_rate.map(|sample_rate| {
                blipbuf::BlipBuf::new(frequency, sample_rate, envelope.volume() as i32)
            }),
            channel_clock,
            length_counter: LengthCounter::new_expired(),
            nrx0,
            nrx1,
            nrx2,
            duty_cycle: DutyCycle::new(),
            envelope,
            sweep,
            active: false,
        }
    }
}

impl<SWEEP> PulseChannel<SWEEP>
where
    SWEEP: Sweep,
{
    #[inline]
    pub(crate) fn active(&self) -> bool {
        self.active
    }

    pub(crate) fn step(&mut self, frame: Option<Frame>) {
        if self.channel_clock.step() {
            if self.active() {
                let is_high_signal = self.duty_cycle.step(self.nrx1);
                let volume = self.envelope.volume() as i32;
                let volume = if is_high_signal { volume } else { -volume };
                if let Some(blipbuf) = &mut self.blipbuf {
                    blipbuf.add_delta(self.channel_clock.div(), volume);
                }
            } else if let Some(blipbuf) = &mut self.blipbuf {
                blipbuf.add_delta(self.channel_clock.div(), 0);
            }
        }

        if let Some(frame) = frame {
            if self.sweep.step(frame) {
                self.channel_clock.reload(self.sweep.period_value());
            }

            if self.active {
                self.envelope.step(frame);
            }
            self.length_counter.step(frame);
        }

        self.active &= self.length_counter.active();
        self.active &= self.sweep.active();
    }

    pub(crate) fn read_samples(&mut self, buffer: &mut [i16], duration: u32) -> usize {
        self.blipbuf.as_mut().map_or(0, |blipbuf| blipbuf.end(buffer, duration))
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
        self.duty_cycle = DutyCycle::new();
    }

    pub(crate) fn set_length_counter(&mut self, value: u8) {
        self.length_counter.set_len(value);
    }
}

impl<SWEEP> Memory for PulseChannel<SWEEP>
where
    SWEEP: Sweep,
{
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0 => {
                self.nrx0 = value;
                self.sweep.set_nrx0(value);
            }
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
                self.sweep.set_nrx3(value);
            }
            4 => {
                self.sweep.set_nrx4(value);
                self.length_counter.set_enabled(value);

                // Trigger the channel
                if is_bit_set!(value, 7) {
                    self.length_counter.trigger();

                    self.sweep.trigger();
                    self.channel_clock.reload(self.sweep.period_value());
                    self.envelope.trigger();
                }

                self.active = self.envelope.dac_on();
                self.active &= self.length_counter.active();
                self.active &= self.sweep.active();
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

#[derive(Serialize, Deserialize)]
pub(crate) struct PulseChannelSnapshot<SWEEP>
where
    SWEEP: Sweep,
{
    nrx0: u8,
    nrx1: u8,
    nrx2: u8,
    channel_clock: Clock,
    lenght_counter: LengthCounter,
    duty_cycle: DutyCycle,
    envelope: Envelope,
    sweep: SWEEP,
    active: bool,
}

impl<SWEEP> Snapshot for PulseChannel<SWEEP>
where
    SWEEP: Sweep + Clone,
{
    type Snapshot = PulseChannelSnapshot<SWEEP>;

    fn snapshot(&self) -> Self::Snapshot {
        PulseChannelSnapshot {
            nrx0: self.nrx0,
            nrx1: self.nrx1,
            nrx2: self.nrx2,
            channel_clock: self.channel_clock.0,
            lenght_counter: self.length_counter,
            duty_cycle: self.duty_cycle,
            envelope: self.envelope,
            sweep: self.sweep.clone(),
            active: self.active,
        }
    }

    fn restore(&mut self, snapshot: Self::Snapshot) {
        self.nrx0 = snapshot.nrx0;
        self.nrx1 = snapshot.nrx1;
        self.nrx2 = snapshot.nrx2;
        self.channel_clock.0 = snapshot.channel_clock;
        self.length_counter = snapshot.lenght_counter;
        self.duty_cycle = snapshot.duty_cycle;
        self.envelope = snapshot.envelope;
        self.sweep = snapshot.sweep;
        self.active = snapshot.active;

        if let Some(blipbuf) = &mut self.blipbuf {
            blipbuf.clear();
        }
    }
}
