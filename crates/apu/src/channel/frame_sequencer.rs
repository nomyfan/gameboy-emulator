use std::ops::Deref;

use crate::{clock::Clock, utils::freq_to_clock_cycles};

pub(crate) struct FrameSequencer {
    clock: Clock,
    frame: Frame,
}

impl FrameSequencer {
    const FRAME_SEQUENCY_PERIOD: u32 = freq_to_clock_cycles(512);

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

#[derive(Debug, Clone, Copy)]
pub(crate) struct Frame(u8);

impl Default for Frame {
    fn default() -> Self {
        Self(7)
    }
}

impl From<u8> for Frame {
    fn from(frame: u8) -> Self {
        Self(frame & 0x7)
    }
}

impl From<&Frame> for u8 {
    fn from(value: &Frame) -> Self {
        value.0
    }
}

impl Deref for Frame {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Frame {
    pub(crate) fn length_counter_frame(&self) -> bool {
        (self.0 & 1) == 0
    }

    pub(crate) fn volume_envelope_frame(&self) -> bool {
        self.0 == 7
    }

    pub(crate) fn sweep_frame(&self) -> bool {
        self.0 == 2 || self.0 == 6
    }

    pub(crate) fn next_frame(&self) -> Self {
        Self((self.0 + 1) & 0x7)
    }
}
