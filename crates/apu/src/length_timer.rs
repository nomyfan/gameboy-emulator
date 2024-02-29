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

    pub(crate) fn new_disabled() -> Self {
        Self::new(64)
    }

    #[inline]
    pub(crate) fn expired(&self) -> bool {
        self.ticks == 64
    }

    pub(crate) fn next(&mut self) {
        if self.expired() {
            return;
        }

        if self.clock.next() != 0 {
            self.ticks += 1;
        }
    }
}
