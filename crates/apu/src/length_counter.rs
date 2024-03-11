use gb_shared::is_bit_set;

use crate::frame_sequencer::FrameSequencer;

pub(crate) struct LengthCounter<const MAX: u16> {
    fs: FrameSequencer,
    /// When the length timer reaches MAX, the channel is turned off.
    pub(crate) len: u16,
    pub(crate) enabled: bool,
}

impl<const MAX: u16> LengthCounter<MAX> {
    pub(crate) fn new(init_value: u8) -> Self {
        let len = MAX.min(init_value as u16);
        Self { fs: FrameSequencer::new(), len, enabled: false }
    }

    pub(crate) fn new_expired() -> Self {
        Self::new(MAX as u8)
    }
}

impl<const MAX: u16> LengthCounter<MAX> {
    #[inline]
    pub(crate) fn expired(&self) -> bool {
        self.len == MAX
    }

    pub(crate) fn set_len(&mut self, len: u8) {
        self.len = len as u16;
    }

    #[inline]
    fn working(&mut self) -> bool {
        (self.fs.current_step() & 1) == 0
    }

    pub(crate) fn set_enabled(&mut self, nrx4: u8) {
        let was_enabled = self.enabled;
        self.enabled = is_bit_set!(nrx4, 6);

        // https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#:~:text=if%20the%20length%20counter%20was%20previously%20disabled%20and%20now%20enabled%20and%20the%20length%20counter%20is%20not%20zero
        if !was_enabled && self.enabled && !self.expired() && self.working() {
            self.len += 1;
        }
    }

    #[inline]
    pub(crate) fn active(&self) -> bool {
        !self.expired()
    }

    pub(crate) fn trigger(&mut self) {
        log::debug!("reset_len {} {}", self.len, MAX);
        if self.expired() {
            self.len = 0;

            // https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#:~:text=it%20is%20set%20to%2063%20instead
            if self.enabled && self.working() {
                self.len += 1;
            }
        }
    }

    pub(crate) fn step(&mut self) {
        if let Some(step) = self.fs.step() {
            if self.expired() || !self.enabled {
                return;
            }

            if (step & 1) == 0 {
                self.len += 1;
            }
        }
    }
}

pub(crate) type LengthCounter64 = LengthCounter<64>;
pub(crate) type LengthCounter256 = LengthCounter<256>;

pub(crate) type PulseChannelLengthCounter = LengthCounter64;
pub(crate) type WaveChannelLengthCounter = LengthCounter256;
pub(crate) type NoiseChannelLengthCounter = LengthCounter64;
