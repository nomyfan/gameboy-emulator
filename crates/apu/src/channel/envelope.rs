use gb_shared::is_bit_set;
use serde::{Deserialize, Serialize};

use super::Frame;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(super) struct Envelope {
    frame: Frame,
    /// Complete one iteration when it reaches zero.
    /// Initialized and reset with `pace`.
    steps: u8,
    pace: u8,
    dir_increase: bool,
    volume: u8,
    initial_volume: u8,
}

impl std::fmt::Debug for Envelope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Envelope")
            .field("frame", &self.frame)
            .field("steps", &self.steps)
            .field("pace", &self.pace)
            .field("dir", &if self.dir_increase { "+" } else { "-" })
            .field("volume", &self.volume)
            .field("initial_volume", &self.initial_volume)
            .field("active", &self.active())
            .finish()
    }
}

impl Envelope {
    fn parse_nrx2(nrx2: u8) -> (u8, bool, u8) {
        let pace = {
            let mut pace = nrx2 & 0b111;
            if pace == 0 {
                pace = 8;
            }
            pace
        };
        let dir_increase = is_bit_set!(nrx2, 3);
        let volume = (nrx2 >> 4) & 0xF;
        (pace, dir_increase, volume)
    }

    pub(super) fn new(nrx2: u8) -> Self {
        let (pace, dir_increase, volume) = Self::parse_nrx2(nrx2);
        Self {
            frame: Default::default(),
            steps: pace,
            pace,
            dir_increase,
            volume,
            initial_volume: volume,
        }
    }

    #[inline]
    fn active(&self) -> bool {
        self.pace != 8 && if self.dir_increase { self.volume != 0xF } else { self.volume != 0 }
    }

    pub(super) fn set_nrx2(&mut self, nrx2: u8) {
        let (pace, dir_increase, volume) = Self::parse_nrx2(nrx2);

        self.pace = pace;
        self.dir_increase = dir_increase;
        self.initial_volume = volume;

        if self.active() {
            self.steps = self.pace;
        }

        // Zombie mode
        if pace == 8 && dir_increase {
            // DMG
            self.volume = (self.volume + 1) & 0xF;
        }
    }

    #[inline]
    pub(super) fn volume(&self) -> u8 {
        self.volume
    }

    #[inline]
    pub(super) fn dac_on(&self) -> bool {
        self.initial_volume != 0 || self.dir_increase
    }

    pub(super) fn trigger(&mut self) {
        self.volume = self.initial_volume;
        if self.active() {
            self.steps = self.pace;
            if self.frame.next_frame().envelope_frame() {
                self.steps += 1;
            }
        }
    }

    /// Only called when the channle is active.
    pub(super) fn step(&mut self, frame: Frame) {
        self.frame = frame;

        if frame.envelope_frame() && self.active() {
            self.steps = self.steps.saturating_sub(1);
            if self.steps == 0 {
                if self.dir_increase {
                    self.volume = (self.volume + 1) & 0xF;
                } else {
                    self.volume = self.volume.saturating_sub(1);
                }

                self.steps = self.pace;
            }
        }
    }
}
