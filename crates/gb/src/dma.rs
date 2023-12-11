use gb_shared::Memory;

#[derive(Debug)]
pub(crate) struct DMA {
    value: u8,
    /// 160 bytes, transferring each costs 1 cycle.
    offset: u8,
}

impl Memory for DMA {
    fn write(&mut self, _0xff46: u16, value: u8) {
        self.value = value;
        self.offset = 0;
    }

    fn read(&self, _0xff46: u16) -> u8 {
        self.value
    }
}

impl DMA {
    pub(crate) fn new() -> Self {
        Self { value: 0, offset: 160 }
    }

    pub(crate) fn active(&self) -> bool {
        self.offset < 160
    }

    pub(crate) fn next_addr(&mut self) -> Option<(u16, u16)> {
        if self.active() {
            let offset = self.offset as u16;
            let dst = 0xFE00 + offset;

            let addr = self.value as u16 * 0x100;
            let src = addr + offset;

            self.offset += 1;

            Some((src, dst))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn inactive_on_creation() {
        let dma = super::DMA::new();
        assert!(!dma.active());
    }
}