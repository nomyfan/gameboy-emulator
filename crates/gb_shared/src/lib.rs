pub mod bitwise;
pub mod command;

#[macro_export]
macro_rules! box_array {
    ($t:ty; $elem:expr; $n:expr) => {{
        let boxed_slice = vec![$elem; $n].into_boxed_slice();
        let ptr = std::boxed::Box::into_raw(boxed_slice) as *mut [$t; $n];
        unsafe { std::boxed::Box::from_raw(ptr) }
    }};
    ($t:ty; $n:expr) => {{
        box_array![$t; <$t as std::default::Default>::default(); $n]
    }};
}

pub trait Memory {
    fn write(&mut self, addr: u16, value: u8);
    fn read(&self, addr: u16) -> u8;
}

pub trait Bus: Memory {
    fn step(&mut self, clocks: u8);
    fn vdma_active(&self) -> bool;
    fn step_vdma(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptType {
    VBlank = 0b1,
    LCDStat = 0b10,
    Timer = 0b100,
    Serial = 0b1000,
    Joypad = 0b10000,
}

pub trait InterruptRequest {
    fn request(&mut self, interrupt_type: InterruptType);
    fn request_vblank(&mut self) {
        self.request(InterruptType::VBlank);
    }
    fn request_lcd_stat(&mut self) {
        self.request(InterruptType::LCDStat);
    }
    fn request_timer(&mut self) {
        self.request(InterruptType::Timer);
    }
    fn request_serial(&mut self) {
        self.request(InterruptType::Serial);
    }
    fn request_joypad(&mut self) {
        self.request(InterruptType::Joypad);
    }
}

#[derive(Debug, Default)]
pub struct Interrupt(pub u8);

impl InterruptRequest for Interrupt {
    fn request(&mut self, interrupt_type: InterruptType) {
        self.0 |= interrupt_type as u8;
    }
}

impl Interrupt {
    pub fn take(&mut self) -> u8 {
        let value = self.0;
        self.0 = 0;
        value
    }
}

pub const fn kib(k: usize) -> usize {
    k * 1024
}

pub const fn mib(m: usize) -> usize {
    kib(m) * 1024
}

pub const CPU_FREQ: u32 = 4_194_304;

pub trait Snapshot {
    type Snapshot;
    fn take_snapshot(&self) -> Self::Snapshot;
    fn restore_snapshot(&mut self, snapshot: Self::Snapshot);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineModel {
    DMG,
    CGB,
}

pub trait ByteView {
    /// Most significant byte
    fn msb(self) -> u8;
    /// Least significant byte
    fn lsb(self) -> u8;
    /// N-th byte
    #[allow(dead_code)]
    fn at(self, nth: u8) -> u8;
}

impl ByteView for u16 {
    #[inline]
    fn msb(self) -> u8 {
        (self >> 8) as u8
    }

    #[inline]
    fn lsb(self) -> u8 {
        self as u8
    }

    fn at(self, nth: u8) -> u8 {
        match nth {
            0 => self.lsb(),
            1 => self.msb(),
            _ => unreachable!("Invalid nth for u16: {}", nth),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_view_for_u16() {
        let value: u16 = 0x1234;
        assert_eq!(value.msb(), 0x12);
        assert_eq!(value.lsb(), 0x34);
        assert_eq!(value.at(0), 0x34);
        assert_eq!(value.at(1), 0x12);
    }
}
