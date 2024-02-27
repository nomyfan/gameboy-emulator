pub(crate) use blip_buf::BlipBuf;

pub fn new(freqency: u32, sample_rate: u32) -> BlipBuf {
    let mut blipbuf = BlipBuf::new(sample_rate);
    blipbuf.set_rates(f64::from(freqency), f64::from(sample_rate));
    blipbuf
}
