use gb_shared::is_bit_set;

use crate::{clock::Clock, utils::freq_to_clock_cycles};

const VOLUME_ENVELOPE_CYCLES: u32 = freq_to_clock_cycles(64);

pub(super) struct VolumeEnvelope {
    clock: Clock,
    volume: u8,
}

impl VolumeEnvelope {
    /// About `nrx2`, see https://gbdev.io/pandocs/Audio_Registers.html#ff12--nr12-channel-1-volume--envelope
    pub(super) fn new(nrx2: u8) -> Self {
        let pace = nrx2 & 0b111;
        let init_volume = (nrx2 >> 4) & 0xF;
        Self { clock: Clock::new(VOLUME_ENVELOPE_CYCLES * pace as u32), volume: init_volume }
    }

    #[inline]
    pub(super) fn volume(&self) -> u8 {
        self.volume
    }

    pub(super) fn next(&mut self, nrx2: u8) {
        if self.clock.next() {
            if is_bit_set!(nrx2, 3) {
                self.volume = self.volume.wrapping_sub(1);
            } else {
                self.volume = self.volume.wrapping_add(1);
            }
        }
    }
}
