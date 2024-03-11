use gb_shared::is_bit_set;

use crate::frame_sequencer::FrameSequencer;

pub(super) struct VolumeEnvelope {
    fs: FrameSequencer,
    /// Complete one iteration when it reaches zero.
    /// Initialized and reset with `pace`.
    steps: u8,
    pace: u8,
    dir_increase: bool,
    volume: u8,
    nrx2: Option<u8>,
}

impl VolumeEnvelope {
    /// About `nrx2`, see https://gbdev.io/pandocs/Audio_Registers.html#ff12--nr12-channel-1-volume--envelope
    pub(super) fn new(nrx2: u8) -> Self {
        let pace = nrx2 & 0b111;
        let dir_increase = is_bit_set!(nrx2, 3);
        let volume = (nrx2 >> 4) & 0xF;
        Self { fs: FrameSequencer::new(), steps: pace, pace, dir_increase, volume, nrx2: None }
    }

    pub(crate) fn set_nrx2(&mut self, nrx2: u8) {
        self.nrx2 = Some(nrx2);
    }

    #[inline]
    pub(super) fn volume(&self) -> u8 {
        self.volume
    }

    pub(super) fn step(&mut self) {
        if let Some(step) = self.fs.step() {
            if self.pace == 0 {
                return;
            }

            if step == 7 {
                self.steps = self.steps.saturating_sub(1);
                // Complete one iteration.
                if self.steps == 0 {
                    if self.dir_increase {
                        self.volume = self.volume.saturating_sub(1);
                    } else {
                        self.volume = self.volume.saturating_add(1) & 0xF;
                    }

                    if let Some(nrx2) = self.nrx2.take() {
                        self.pace = nrx2 & 0b111;
                        self.dir_increase = is_bit_set!(nrx2, 3);
                        self.volume = (nrx2 >> 4) & 0xF;
                    }

                    self.steps = self.pace;
                }
            }
        }
    }
}
