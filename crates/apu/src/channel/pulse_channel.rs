use gb_shared::{is_bit_set, Memory};

use crate::{
    blipbuf,
    clock::Clock,
    length_timer::LengthTimer,
    utils::{freq_to_clock_cycles, pulse_channel_period_sweep, pulse_channel_sample_period},
};

pub(crate) struct PulseChannel {
    blipbuf: blipbuf::BlipBuf,
    channel_clock: Clock,
    // TODO: retrigger will reset
    period_sweep_clock: Clock,
    envelope_clock: Clock,
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
    /// This value will be changed by period sweep functionality.
    period_value: u16,
    /// This value will be changed by envelope functionality.
    volume_value: u8,
}

#[inline]
fn period_value(nrx3: u8, nrx4: u8) -> u16 {
    ((nrx4 as u16 & 0b111) << 8) | (nrx3 as u16)
}

#[inline]
fn new_channel_clock(period_value: u16) -> Clock {
    Clock::new(pulse_channel_sample_period(period_value))
}

const PERIOD_SWEEP_CYCLES: u32 = freq_to_clock_cycles(128);

fn new_period_sweep_clock(nrx0: u8) -> Clock {
    let pace = (nrx0 >> 4) & 0b111;
    Clock::new(PERIOD_SWEEP_CYCLES * pace as u32)
}

const VOLUME_ENVELOPE_CYCLES: u32 = freq_to_clock_cycles(64);

fn new_envelope_clock(nrx2: u8) -> Clock {
    let pace = nrx2 & 0b111;
    Clock::new(VOLUME_ENVELOPE_CYCLES * pace as u32)
}

impl PulseChannel {
    /// Create CH1(left) and CH2(right).
    pub(crate) fn new_chs(frequency: u32, sample_rate: u32) -> (Self, Self) {
        let nrx0 = 0x80;
        let nrx3 = 0xFF;
        let nrx4 = 0xBF;

        let period_value = period_value(nrx3, nrx4);

        let new_channel = |nrx2: u8| Self {
            blipbuf: blipbuf::new(frequency, sample_rate),
            channel_clock: new_channel_clock(period_value),
            period_sweep_clock: new_period_sweep_clock(nrx0),
            envelope_clock: new_envelope_clock(nrx2),
            length_timer: LengthTimer::new(0x3F),
            nrx0,
            nrx1: 0xBF,
            nrx2,
            nrx3,
            nrx4,
            period_value,
            volume_value: (nrx2 >> 4) & 0xF,
        };

        (new_channel(0xF3), new_channel(0x00))
    }

    #[inline]
    fn dac_off(&self) -> bool {
        (self.nrx2 >> 3) == 0
    }

    #[inline]
    fn period_overflow(&self) -> bool {
        self.period_value > 0x7FF
    }

    fn wave_duty(&self) -> u8 {
        match (self.nrx1 >> 6) & 0b11 {
            // 12.5%
            0b00 => 0b1111_1110,
            // 25%
            0b01 => 0b0111_1110,
            // 50%
            0b10 => 0b0111_1000,
            // 75%
            0b11 => 0b1000_0001,
            _ => unreachable!(),
        }
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
        // TODO: confirm should channel continue working when deactivated.
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

        if self.envelope_clock.next() != 0 {
            if is_bit_set!(self.nrx2, 3) {
                self.volume_value = self.volume_value.wrapping_sub(1);
            } else {
                self.volume_value = self.volume_value.wrapping_add(1);
            }
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

                self.length_timer = if is_bit_set!(value, 6) {
                    LengthTimer::new(self.nrx1 & 0x3F)
                } else {
                    LengthTimer::new_disabled()
                };

                if is_bit_set!(value, 7) && !self.dac_off() {
                    // TODO: trigger channel only when its DAC is on
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
