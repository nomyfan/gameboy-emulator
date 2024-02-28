use crate::clock::Clock;

pub(crate) struct LengthTimer {
    clock: Clock,
    /// When the length timer reaches 64, the channel is turned off.
    ticks: u8,
}

impl LengthTimer {
    pub(crate) fn new(init_value: u8) -> Self {
        let init_value = 64.min(init_value);

        Self {
            // CPU_FREQ / 256 Hz = 16384
            clock: Clock::new(16384),
            ticks: init_value,
        }
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
