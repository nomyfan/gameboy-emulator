use gb_shared::is_bit_set;

use crate::{clock::Clock, utils::freq_to_clock_cycles};

pub(crate) struct LengthTimer<const MAX: u16> {
    clock: Clock,
    /// When the length timer reaches MAX, the channel is turned off.
    pub(crate) len: u16,
    pub(crate) enabled: bool,
}

const LENGTH_TIMER_CYCLES: u32 = freq_to_clock_cycles(256);

impl<const MAX: u16> LengthTimer<MAX> {
    pub(crate) fn new(init_value: u8) -> Self {
        let len = MAX.min(init_value as u16);
        Self { clock: Clock::new(LENGTH_TIMER_CYCLES), len, enabled: false }
    }

    pub(crate) fn new_expired() -> Self {
        Self::new(MAX as u8)
    }
}

impl<const MAX: u16> LengthTimer<MAX> {
    #[inline]
    pub(crate) fn expired(&self) -> bool {
        self.len == MAX
    }

    /// Reset the length to to maximum when it's expired.
    pub(crate) fn reset_len(&mut self) {
        log::debug!("reset_len {} {}", self.len, MAX);
        if self.expired() {
            log::debug!("Reset OK");
            self.len = 0;
        }
    }

    pub(crate) fn set_len(&mut self, len: u8) {
        self.len = len as u16;
    }

    pub(crate) fn set_enabled(&mut self, nrx4: u8) {
        self.enabled = is_bit_set!(nrx4, 6);
    }

    #[inline]
    pub(crate) fn active(&self) -> bool {
        !self.expired()
    }

    pub(crate) fn step(&mut self) {
        if self.expired() || !self.enabled {
            return;
        }

        if self.clock.step() {
            self.len += 1;
        }
    }
}

pub(crate) type LengthTimer64 = LengthTimer<64>;
pub(crate) type LengthTimer256 = LengthTimer<256>;

pub(crate) type PulseChannelLengthTimer = LengthTimer64;
pub(crate) type WaveChannelLengthTimer = LengthTimer256;
pub(crate) type NoiseChannelLengthTimer = LengthTimer64;
