use gb_shared::is_bit_set;
use serde::{Deserialize, Serialize};

use super::Frame;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct LengthCounter<const MAX: u16> {
    pub(super) frame: Frame,
    /// When the length timer reaches MAX, the channel is turned off.
    len: u16,
    enabled: bool,
}

impl<const MAX: u16> LengthCounter<MAX> {
    pub(crate) fn new_expired() -> Self {
        Self { frame: Default::default(), len: MAX, enabled: false }
    }
}

impl<const MAX: u16> std::fmt::Debug for LengthCounter<MAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LengthCounter")
            .field("MAX", &MAX)
            .field("len", &self.len)
            .field("enabled", &self.enabled)
            .finish()
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
    pub(crate) fn enabled(&self) -> bool {
        self.enabled
    }

    pub(crate) fn set_enabled(&mut self, nrx4: u8) {
        let was_enabled = self.enabled;
        self.enabled = is_bit_set!(nrx4, 6);

        // https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#:~:text=if%20the%20length%20counter%20was%20previously%20disabled%20and%20now%20enabled%20and%20the%20length%20counter%20is%20not%20zero
        if !was_enabled && self.enabled && !self.expired() && self.frame.length_counter_frame() {
            self.len += 1;
        }
    }

    #[inline]
    pub(crate) fn active(&self) -> bool {
        !self.expired()
    }

    pub(crate) fn trigger(&mut self) {
        if self.expired() {
            self.len = 0;

            // https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#:~:text=it%20is%20set%20to%2063%20instead
            if self.enabled && self.frame.length_counter_frame() {
                self.len += 1;
            }
        }
    }

    pub(crate) fn step(&mut self, frame: Frame) {
        self.frame = frame;

        if self.expired() || !self.enabled {
            return;
        }

        if frame.length_counter_frame() {
            self.len += 1;
        }
    }
}

pub(crate) type LengthCounter64 = LengthCounter<64>;
pub(crate) type LengthCounter256 = LengthCounter<256>;

pub(crate) type PulseChannelLengthCounter = LengthCounter64;
pub(crate) type WaveChannelLengthCounter = LengthCounter256;
pub(crate) type NoiseChannelLengthCounter = LengthCounter64;
