use serde::{Deserialize, Serialize};

use crate::{clock::Clock, utils::freq_to_period};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct FrameSequencer {
    clock: Clock,
    frame: Frame,
}

impl FrameSequencer {
    const FRAME_SEQUENCY_PERIOD: u32 = freq_to_period(512);

    pub(crate) fn new() -> Self {
        Self { clock: Clock::new(Self::FRAME_SEQUENCY_PERIOD), frame: Default::default() }
    }

    pub(crate) fn step(&mut self) -> Option<Frame> {
        if self.clock.step() {
            self.frame.0 = (self.frame.0 + 1) & 0x7;
            Some(self.frame)
        } else {
            None
        }
    }

    pub(crate) fn power_off(&mut self) {
        self.frame = Default::default();
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct Frame(u8);

impl Default for Frame {
    fn default() -> Self {
        Self(7)
    }
}

impl Frame {
    pub(crate) fn length_counter_frame(&self) -> bool {
        (self.0 & 1) == 0
    }

    pub(crate) fn envelope_frame(&self) -> bool {
        self.0 == 7
    }

    pub(crate) fn sweep_frame(&self) -> bool {
        self.0 == 2 || self.0 == 6
    }

    pub(crate) fn next_frame(&self) -> Self {
        Self((self.0 + 1) & 0x7)
    }
}
