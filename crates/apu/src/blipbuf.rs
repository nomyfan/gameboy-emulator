pub(crate) use blip_buf::BlipBuf;

pub fn new(frequency: u32, sample_rate: u32) -> BlipBuf {
    let mut blipbuf = BlipBuf::new(sample_rate);
    blipbuf.set_rates(f64::from(frequency), f64::from(sample_rate));
    blipbuf
}
