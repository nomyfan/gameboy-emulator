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
        // It has no chance to overflow u32::MAX.
        self.clock_time = self.clock_time.saturating_add(duration);
        self.buf.add_delta(self.clock_time, volume - self.volume);
        self.volume = volume;
    }

    pub(crate) fn end(&mut self, buffer: &mut [i16], duration: u32) {
        self.buf.end_frame(duration);

        let samples_avail = self.buf.samples_avail();
        debug_assert_eq!(samples_avail as usize, buffer.len());
        self.buf.read_samples(buffer, false);
        self.clock_time = 0;
    }

    pub(crate) fn clear(&mut self) {
        self.buf.clear();
    }
}
