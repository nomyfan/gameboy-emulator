#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Clock {
    div: u32,
    clocks: u32,
}

impl Clock {
    pub(crate) fn new(div: u32) -> Self {
        Self { div, clocks: 0 }
    }

    pub(crate) fn div(&self) -> u32 {
        self.div
    }

    pub(crate) fn step(&mut self) -> bool {
        if self.div == 0 {
            return false;
        }

        self.clocks += 1;
        let emit = (self.clocks / self.div) != 0;
        self.clocks %= self.div;
        emit
    }
}
