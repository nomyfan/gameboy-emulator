use gb_shared::is_bit_set;

use super::Frame;

pub(super) struct VolumeEnvelope {
    frame: Frame,
    /// Complete one iteration when it reaches zero.
    /// Initialized and reset with `pace`.
    steps: u8,
    pace: u8,
    dir_increase: bool,
    volume: u8,
    initial_volume: u8,
}

impl VolumeEnvelope {
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
        // TODO: zombie mode
        let (pace, dir_increase, volume) = Self::parse_nrx2(nrx2);

        self.pace = pace;
        self.dir_increase = dir_increase;
        self.initial_volume = volume;
        if self.active() {
            self.steps = self.pace;
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
            if self.frame.next_frame().volume_envelope_frame() {
                // TODO: what if it reaches zero?
                self.steps = self.steps.saturating_sub(1);
            }
        }
    }

    /// Only called when the channle is active.
    pub(super) fn step(&mut self, frame: Frame) {
        self.frame = frame;

        if frame.volume_envelope_frame() && self.active() {
            self.steps = self.steps.saturating_sub(1);
            if self.steps == 0 {
                if self.dir_increase {
                    self.volume = self.volume.saturating_add(1) & 0xF;
                } else {
                    self.volume = self.volume.saturating_sub(1);
                }

                self.steps = self.pace;
            }
        }
    }
}
