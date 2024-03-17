mod envelope;
mod frame_sequencer;
mod length_counter;
mod noise_channel;
mod pulse_channel;
mod sweep;
mod wave_channel;

use envelope::Envelope;
pub(crate) use frame_sequencer::{Frame, FrameSequencer};
use length_counter::{
    NoiseChannelLengthCounter, PulseChannelLengthCounter, WaveChannelLengthCounter,
};
use noise_channel::NoiseChannel;
use pulse_channel::PulseChannel;
use sweep::Sweep;
use wave_channel::WaveChannel;

pub(crate) type Channel1 = PulseChannel<sweep::SomeSweep>;
pub(crate) type Channel2 = PulseChannel<sweep::NoneSweep>;
pub(crate) type Channel3 = WaveChannel;
pub(crate) type Channel4 = NoiseChannel;
