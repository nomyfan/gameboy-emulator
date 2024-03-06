use crate::{clock::Clock, utils::freq_to_clock_cycles};

pub(crate) struct LengthTimer {
    clock: Clock,
    /// When the length timer reaches 64, the channel is turned off.
    ticks: u8,
}

const LENGTH_TIMER_CYCLES: u32 = freq_to_clock_cycles(256);

impl LengthTimer {
    pub(crate) fn new(init_value: u8) -> Self {
        let init_value = 64.min(init_value);

        Self { clock: Clock::new(LENGTH_TIMER_CYCLES), ticks: init_value }
    }

    pub(crate) fn new_expired() -> Self {
        Self::new(64)
    }
}

impl LengthTimer {
    #[inline]
    pub(crate) fn expired(&self) -> bool {
        self.ticks == 64
    }

    pub(crate) fn reset(&mut self) {
        if self.expired() {
            self.ticks = 0;
        }
    }

    pub(crate) fn set(&mut self, ticks: u8) {
        self.ticks = ticks;
    }

    pub(crate) fn step(&mut self) {
        if self.expired() {
            return;
        }

        if self.clock.step() {
            self.ticks += 1;
        }
    }
}
