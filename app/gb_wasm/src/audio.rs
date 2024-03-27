use std::sync::{Arc, Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait},
    ChannelCount, FromSample, Sample, SizedSample, Stream,
};
use web_sys::console;

pub(crate) type AudioSamplesBuffer = Vec<(f32, f32)>;

fn write_data<T>(channel_count: ChannelCount, output: &mut [T], samples: &mut AudioSamplesBuffer)
where
    T: SizedSample + FromSample<f32>,
{
    let channel_count = channel_count as usize;
    let len = std::cmp::min(output.len() / channel_count, samples.len());

    samples.drain(..len).zip(output.chunks_mut(channel_count)).for_each(
        |((left_channel, right_channel), channles)| {
            channles.iter_mut().zip(&[left_channel, right_channel]).for_each(|(o, i)| {
                *o = i.to_sample();
            });
        },
    );
}

pub(crate) fn init_audio() -> anyhow::Result<(Stream, Arc<Mutex<AudioSamplesBuffer>>, u32)> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("No output device available"))?;
    let config = device.default_output_config()?;
    let sample_format = config.sample_format();
    let config: cpal::StreamConfig = config.into();
    let sample_rate = config.sample_rate.0 as u32;
    let channel_count = config.channels;

    let samples_buf: Arc<Mutex<AudioSamplesBuffer>> = Arc::new(Mutex::new(Vec::new()));
    let stream = {
        let samples_buf = samples_buf.clone();
        let stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        write_data(channel_count, data, &mut samples_buf.lock().unwrap());
                    },
                    move |err| console::error_1(&err.to_string().into()),
                    None,
                )
                .unwrap(),
            cpal::SampleFormat::F64 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [f64], _: &cpal::OutputCallbackInfo| {
                        write_data(channel_count, data, &mut samples_buf.lock().unwrap());
                    },
                    move |err| console::error_1(&err.to_string().into()),
                    None,
                )
                .unwrap(),
            _ => unreachable!(),
        };

        stream
    };

    Ok((stream, samples_buf, sample_rate))
}
