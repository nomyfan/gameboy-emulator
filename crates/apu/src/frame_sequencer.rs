use crate::{clock::Clock, utils::freq_to_clock_cycles};

pub(crate) struct FrameSequencer {
    clock: Clock,
    step: u8,
}

impl FrameSequencer {
    const FRAME_SEQUENCY_PERIOD: u32 = freq_to_clock_cycles(512);

    pub(crate) fn new() -> Self {
        Self { clock: Clock::new(Self::FRAME_SEQUENCY_PERIOD), step: 0 }
    }

    pub(crate) fn current_step(&self) -> u8 {
        self.step
    }

    pub(crate) fn step(&mut self) -> Option<u8> {
        if self.clock.step() {
            self.step = (self.step + 1) % 8;
            Some(self.step)
        } else {
            None
        }
    }
}
