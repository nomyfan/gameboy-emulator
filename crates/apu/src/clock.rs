pub(crate) struct Clock {
    div: u32,
    ticks: u32,
}

impl Clock {
    pub(crate) fn new(div: u32) -> Self {
        Self { div, ticks: 0 }
    }

    pub(crate) fn next(&mut self) -> u32 {
        if self.div == 0 {
            return 0;
        }

        self.ticks += 1;
        let cycle = self.ticks / self.div;
        self.ticks %= self.div;
        cycle
    }
}
