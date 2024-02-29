use gb_shared::Memory;

use crate::{
    blipbuf,
    clock::Clock,
    length_timer::LengthTimer,
    utils::{pulse_channel_period_sweep, pulse_channel_sample_period, pulse_period_sweep_period},
};

pub(crate) struct PulseChannel {
    blipbuf: blipbuf::BlipBuf,
    channel_clock: Clock,
    // TODO: retrigger will reset
    period_sweep_clock: Clock,
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
    period_value: u16,
}

#[inline]
fn period_value(nrx3: u8, nrx4: u8) -> u16 {
    ((nrx4 as u16 & 0b111) << 8) | (nrx3 as u16)
}

#[inline]
fn new_channel_clock(period_value: u16) -> Clock {
    Clock::new(pulse_channel_sample_period(period_value))
}

#[inline]
fn new_period_sweep_clock(nrx0: u8) -> Clock {
    Clock::new(pulse_period_sweep_period(nrx0))
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
                channel_clock: new_channel_clock(period_value),
                period_sweep_clock: new_period_sweep_clock(nrx0),
                length_timer: LengthTimer::new(0x3F),
                nrx0,
                nrx1: 0xBF,
                nrx2: 0xF3,
                nrx3,
                nrx4,
                period_value,
            },
            Self {
                blipbuf: blipbuf::new(frequency, sample_rate),
                channel_clock: new_channel_clock(period_value),
                period_sweep_clock: new_period_sweep_clock(nrx0),
                length_timer: LengthTimer::new(0x3F),
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

    /// Any condition below satisfied will deactivate the channel.
    /// - DAC is off.
    /// - Length timer expired.
    /// - Period overflowed.
    #[inline]
    pub(crate) fn deactivated(&self) -> bool {
        self.dac_off() || self.length_timer.expired() || self.period_overflow()
    }

    pub(crate) fn next(&mut self) {
        if self.channel_clock.next() != 0 {
            if self.deactivated() {
                // TODO: if it's deactivated, generate 0
                unimplemented!()
            } else {
                // TODO: generate sample data
                unimplemented!()
            }
        }

        if self.period_sweep_clock.next() != 0 {
            self.period_value = pulse_channel_period_sweep(self.period_value, self.nrx0);
            if !self.period_overflow() {
                let lo = self.period_value as u8;
                let hi = ((self.period_value >> 8) as u8) & 0b111;
                self.nrx3 = lo;
                self.nrx4 = (self.nrx4 & (!0b111)) | hi;
                self.channel_clock = new_channel_clock(self.period_value);
            }

            self.period_sweep_clock = new_period_sweep_clock(self.nrx0);
        }

        self.length_timer.next();
    }
}

impl Memory for PulseChannel {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0 => {
                self.nrx0 = value;
                self.period_sweep_clock = new_period_sweep_clock(self.nrx0);
            }
            1 => {
                self.nrx1 = value;
                self.length_timer = LengthTimer::new(self.nrx1 & 0x3F);
            }
            2 => self.nrx2 = value,
            3 => {
                self.nrx3 = value;
                self.period_value = period_value(self.nrx3, self.nrx4);
                self.channel_clock = new_channel_clock(self.period_value);
            }
            4 => {
                self.nrx4 = value;
                self.period_value = period_value(self.nrx3, self.nrx4);
                self.channel_clock = new_channel_clock(self.period_value);
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
