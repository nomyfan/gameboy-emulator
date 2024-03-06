use std::sync::{Arc, Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait},
    Sample, Stream,
};

pub(crate) type AudioSamplesBuffer = Vec<(f32, f32)>;

pub(crate) fn init_audio() -> (Stream, Arc<Mutex<AudioSamplesBuffer>>, u32) {
    let samples_buf: Arc<Mutex<AudioSamplesBuffer>> = Arc::new(Mutex::new(Vec::new()));
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    log::debug!("Audio device: {}", device.name().unwrap());
    let config = device.default_output_config().unwrap();
    let sample_format = config.sample_format();
    log::debug!("Sample format: {}", sample_format);
    let config: cpal::StreamConfig = config.into();
    log::debug!("Stream config: {:?}", config);
    let sample_rate = config.sample_rate.0 as u32;

    let stream = {
        let samples_buf = samples_buf.clone();
        let stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        let mut samples_buf = samples_buf.lock().unwrap();
                        let len = std::cmp::min(data.len() / 2, samples_buf.len());
                        for (i, (sample_left, sample_right)) in samples_buf.drain(..len).enumerate()
                        {
                            data[i * 2] = sample_left;
                            data[i * 2 + 1] = sample_right;
                        }
                    },
                    move |err| log::error!("{}", err),
                    None,
                )
                .unwrap(),
            cpal::SampleFormat::F64 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [f64], _: &cpal::OutputCallbackInfo| {
                        let mut samples_buf = samples_buf.lock().unwrap();
                        let len = std::cmp::min(data.len() / 2, samples_buf.len());
                        for (i, (sample_left, sample_right)) in samples_buf.drain(..len).enumerate()
                        {
                            data[i * 2] = sample_left.to_sample::<f64>();
                            data[i * 2 + 1] = sample_right.to_sample::<f64>();
                        }
                    },
                    move |err| log::error!("{}", err),
                    None,
                )
                .unwrap(),
            _ => panic!("unreachable"),
        };

        stream
    };

    (stream, samples_buf, sample_rate)
}
