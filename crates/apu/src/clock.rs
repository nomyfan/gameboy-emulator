pub(crate) struct Clock {
    div: u32,
    ticks: u32,
}

impl Clock {
    pub(crate) fn new(div: u32) -> Self {
        Self { div, ticks: 0 }
    }

    pub(crate) fn div(&self) -> u32 {
        self.div
    }

    pub(crate) fn step(&mut self) -> bool {
        if self.div == 0 {
            return false;
        }

        self.ticks += 1;
        let should_tick = (self.ticks / self.div) != 0;
        self.ticks %= self.div;
        should_tick
    }
}
