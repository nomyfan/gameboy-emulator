use gb_shared::{is_bit_set, InterruptRequest, Memory};

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

pub(crate) struct Timer<INT: InterruptRequest> {
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
    interrupt_request: INT,
}

impl<INT: InterruptRequest> Memory for Timer<INT> {
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

impl<INT: InterruptRequest> Timer<INT> {
    pub fn new(interrupt_request: INT) -> Self {
        Self { div: 0, tima: 0, tma: 0, tac: 0, cycles_so_far: 0, interrupt_request }
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
                self.interrupt_request.request_timer();
            }
        }
    }
}

#[cfg(test)]
use gb_shared::InterruptType;
#[cfg(test)]
use mockall::mock;

#[cfg(test)]
mock! {
    pub InterruptRequest {}

    impl InterruptRequest for InterruptRequest {
        fn request(&mut self, interrupt_type: InterruptType) {}
    }
}

#[cfg(test)]
mod tests {
    use core::time;
    use mockall::predicate::*;

    use super::*;

    fn write_div(timer: &mut Timer<MockInterruptRequest>, value: u8) {
        timer.write(0xFF04, value);
    }

    fn write_tima(timer: &mut Timer<MockInterruptRequest>, value: u8) {
        timer.write(0xFF05, value);
    }

    fn write_tma(timer: &mut Timer<MockInterruptRequest>, value: u8) {
        timer.write(0xFF06, value);
    }

    fn write_tac(timer: &mut Timer<MockInterruptRequest>, value: u8) {
        timer.write(0xFF07, value);
    }

    #[test]
    fn div_increased_ignoring_tac() {
        let mut timer = Timer::new(MockInterruptRequest::new());
        write_tac(&mut timer, 0);

        timer.step(255);
        assert_eq!(timer.cycles_so_far, 255);
        assert_eq!(timer.div, 0);

        timer.step(1);
        assert_eq!(timer.cycles_so_far, 256);
        assert_eq!(timer.div, 1);
    }

    #[test]
    fn tima_increased_every_1024_cycles() {
        let mut timer = Timer::new(MockInterruptRequest::new());
        write_tac(&mut timer, 0b100);

        for _ in 0..4 {
            timer.step(255);
        }
        assert_eq!(timer.cycles_so_far, 1020);
        assert_eq!(timer.tima, 0);

        timer.step(4);
        assert_eq!(timer.cycles_so_far, 1024);
        assert_eq!(timer.tima, 1);

        for _ in 0..4 {
            timer.step(255);
        }
        assert_eq!(timer.cycles_so_far, 2044);
        assert_eq!(timer.tima, 1);

        timer.step(4);
        assert_eq!(timer.cycles_so_far, 2048);
        assert_eq!(timer.tima, 2);
    }

    #[test]
    fn tima_increased_every_16_cycles() {
        let mut timer = Timer::new(MockInterruptRequest::new());
        write_tac(&mut timer, 0b101);

        timer.step(16);
        assert_eq!(timer.tima, 1);

        timer.step(15);
        assert_eq!(timer.tima, 1);
        timer.step(1);
        assert_eq!(timer.tima, 2);
    }

    #[test]
    fn tima_increased_every_64_cycles() {
        let mut timer = Timer::new(MockInterruptRequest::new());
        write_tac(&mut timer, 0b110);

        timer.step(64);
        assert_eq!(timer.tima, 1);

        timer.step(63);
        assert_eq!(timer.tima, 1);
        timer.step(1);
        assert_eq!(timer.tima, 2);
    }

    #[test]
    fn tima_increased_every_256_cycles() {
        let mut timer = Timer::new(MockInterruptRequest::new());
        write_tac(&mut timer, 0b111);

        timer.step(255);
        assert_eq!(timer.tima, 0);
        timer.step(1);
        assert_eq!(timer.tima, 1);

        timer.step(255);
        assert_eq!(timer.tima, 1);
        timer.step(1);
        assert_eq!(timer.tima, 2);
    }

    #[test]
    fn tima_not_increased_if_tima_is_disabled() {
        let mut timer = Timer::new(MockInterruptRequest::new());
        write_tac(&mut timer, 0b001);

        timer.step(16);
        assert_eq!(timer.tima, 0);

        timer.step(16);
        assert_eq!(timer.tima, 0);
    }

    #[test]
    fn request_timer_interrupt() {
        let mut mock_int_req = MockInterruptRequest::new();
        mock_int_req.expect_request().with(eq(InterruptType::Timer)).once().return_const(());

        let mut timer = Timer::new(mock_int_req);
        write_tma(&mut timer, 0xFE);
        write_tima(&mut timer, 0xFF);
        write_tac(&mut timer, 0b111);

        timer.step(255);
        timer.step(1);

        assert_eq!(timer.tima, 0xFE);
    }
}
