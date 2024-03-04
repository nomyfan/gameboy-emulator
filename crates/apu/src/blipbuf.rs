pub(crate) struct BlipBuf {
    buf: blip_buf::BlipBuf,
    volume: i32,
    clock_time: u32,
}

impl BlipBuf {
    pub(crate) fn new(frequency: u32, sample_rate: u32, init_volume: i32) -> Self {
        let mut blipbuf = blip_buf::BlipBuf::new(sample_rate);
        blipbuf.set_rates(f64::from(frequency), f64::from(sample_rate));

        Self { buf: blipbuf, volume: init_volume, clock_time: 0 }
    }

    pub(crate) fn add_delta(&mut self, duration: u32, volume: i32) {
        // TODO: check this MUST not overflow
        self.clock_time = self.clock_time.saturating_add(duration);
        self.buf.add_delta(self.clock_time, volume - self.volume);
        self.volume = volume;
    }

    pub(crate) fn end(&mut self, duration: u32) -> Vec<i16> {
        self.buf.end_frame(duration);

        let samples_avail = self.buf.samples_avail();
        // TODO: keep an eye on performance
        let mut buf = vec![0; samples_avail as usize];
        self.buf.read_samples(&mut buf, false);
        self.clock_time = 0;

        buf
    }
}
