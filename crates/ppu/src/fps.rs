use web_time::Instant;

#[derive(Debug)]
pub(super) struct Fps {
    frame_count: u32,
    last_frame_time: Instant,
}

impl Fps {
    pub(super) fn new() -> Self {
        Self { frame_count: 0, last_frame_time: Instant::now() }
    }

    pub(super) fn stop(&mut self) {
        self.frame_count = 0;
    }

    pub(super) fn update(&mut self) -> Option<f32> {
        if self.frame_count == 0 {
            // This is the point where we assume it starts rendering.
            self.last_frame_time = Instant::now();
            self.frame_count = 1;
            return None;
        }

        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now - self.last_frame_time;
        let elapsed = elapsed.as_secs_f32();
        if elapsed >= 1.0 {
            let fps = (self.frame_count - 1) as f32 / elapsed;
            self.frame_count = 1;
            self.last_frame_time = now;
            Some(fps)
        } else {
            None
        }
    }
}

impl Default for Fps {
    fn default() -> Self {
        Self::new()
    }
}
