mod noise_channel;
mod period_sweep;
mod pulse_channel;
mod volume_envelope;
mod wave_channel;

use noise_channel::NoiseChannel;
use pulse_channel::PulseChannel;
use volume_envelope::VolumeEnvelope;
use wave_channel::WaveChannel;

pub(crate) type Channel1 = PulseChannel<period_sweep::SomePeriodSweep>;
pub(crate) type Channel2 = PulseChannel<period_sweep::NonePeriodSweep>;
pub(crate) type Channel3 = WaveChannel;
pub(crate) type Channel4 = NoiseChannel;
