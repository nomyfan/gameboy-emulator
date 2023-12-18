use gb_shared::{is_bit_set, Memory};

enum CounterIncCycles {
    Cycles1024,
    Cycles16,
    Cycles64,
    Cycles256,
}

impl From<u8> for CounterIncCycles {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => CounterIncCycles::Cycles1024,
            0b01 => CounterIncCycles::Cycles16,
            0b10 => CounterIncCycles::Cycles64,
            0b11 => CounterIncCycles::Cycles256,
            _ => unreachable!(),
        }
    }
}

pub(crate) struct Timer<BUS: Memory> {
    cycles_so_far: u16,
    /// Divider
    /// It's increased at a rate of 16384 Hz.
    /// While CPU is working on a frequency of 4194304 Hz(256 times of 16384).
    /// Which means it's increased by 1 every 256 cycles.
    div: u8,
    /// Timer counter
    tima: u8,
    /// Timer modulo
    tma: u8,
    /// Timer control
    /// - Bit 2: Control whether `TIMA` is increased. Note that `DIV` is always increased.
    /// - Bit 1-0: Timer frequency.
    ///     - 00: 4096 Hz(increased by 1 every 1024 cycles)
    ///     - 01: 262144 Hz(increased by 1 every 16 cycles)
    ///     - 10: 65536 Hz(increased by 1 every 64 cycles)
    ///     - 11: 16384 Hz(increased by 1 every 256 cycles)
    tac: u8,
    bus: BUS,
}

impl<BUS: Memory> Memory for Timer<BUS> {
    fn write(&mut self, addr: u16, value: u8) {
        if addr == 0xFF04 {
            // Write any value to it will reset it to zero.
            self.div = 0;
        } else if addr == 0xFF05 {
            self.tima = value;
        } else if addr == 0xFF06 {
            self.tma = value;
        } else if addr == 0xFF07 {
            self.tac = value;
        } else {
            unreachable!()
        }
    }

    fn read(&self, addr: u16) -> u8 {
        if addr == 0xFF04 {
            self.div
        } else if addr == 0xFF05 {
            self.tima
        } else if addr == 0xFF06 {
            self.tma
        } else if addr == 0xFF07 {
            self.tac
        } else {
            unreachable!()
        }
    }
}

impl<BUS: Memory> Timer<BUS> {
    pub fn new(bus: BUS) -> Self {
        Self { div: 0, tima: 0, tma: 0, tac: 0, cycles_so_far: 0, bus }
    }

    pub fn step(&mut self, cycles: u8) {
        let old_cycles_so_far = self.cycles_so_far;
        self.cycles_so_far = self.cycles_so_far.wrapping_add(cycles as u16);

        if self.cycles_so_far / 256 != old_cycles_so_far / 256 {
            self.div = self.div.wrapping_add(1);
        }

        let inc_tima = match CounterIncCycles::from(self.tac) {
            CounterIncCycles::Cycles1024 => self.cycles_so_far / 1024 != old_cycles_so_far / 1024,
            CounterIncCycles::Cycles16 => self.cycles_so_far / 16 != old_cycles_so_far / 16,
            CounterIncCycles::Cycles64 => self.cycles_so_far / 64 != old_cycles_so_far / 64,
            CounterIncCycles::Cycles256 => self.cycles_so_far / 256 != old_cycles_so_far / 256,
        };

        if is_bit_set!(self.tac, 2) && inc_tima {
            self.tima = self.tima.wrapping_add(1);
            if self.tima == 0 {
                self.tima = self.tma;
                // TODO: refactor request interrupt impl
                self.bus.write(0xFF0F, self.bus.read(0xFF0F) | 0b100);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn div_increased_ignoring_tac() {
        //
    }

    #[test]
    fn tima_increased_every_1024_cycles() {
        //
    }

    #[test]
    fn tima_increased_every_16_cycles() {
        //
    }

    #[test]
    fn tima_increased_every_64_cycles() {
        //
    }

    #[test]
    fn tima_increased_every_256_cycles() {
        //
    }

    #[test]
    fn request_timer_interrupt() {
        //
    }
}
