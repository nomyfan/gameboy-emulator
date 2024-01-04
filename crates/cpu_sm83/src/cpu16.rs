#[cfg(test)]
use mockall::{mock, predicate::*};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Register8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Register {
    R8(Register8),
    R16(Register16),
}

/// 16 bits CPU
pub(crate) trait Cpu16 {
    fn adv_cycles(&mut self, cycles: u8);
    fn read_r8(&self, reg: Register8) -> u8;
    fn write_r8(&mut self, reg: Register8, value: u8);
    fn read_r16(&self, reg: Register16) -> u16;
    fn write_r16(&mut self, reg: Register16, value: u16);
    fn bus_read(&mut self, addr: u16) -> u8;
    fn bus_write(&mut self, addr: u16, value: u8);
    fn read_pc(&mut self) -> u8;
    fn set_flags(&mut self, z: Option<bool>, n: Option<bool>, h: Option<bool>, c: Option<bool>);
    /// Z,N,H,C in order.
    fn flags(&self) -> (bool, bool, bool, bool);
    fn stack_push(&mut self, value: u8);
    fn stack_push2(&mut self, value: u16) {
        self.stack_push((value >> 8) as u8);
        self.stack_push(value as u8);
    }
    fn stack_pop(&mut self) -> u8;
    fn stack_pop2(&mut self) -> u16 {
        let lo = self.stack_pop();
        let hi = self.stack_pop();

        ((hi as u16) << 8) | (lo as u16)
    }
    fn stack_push_pc(&mut self);
    /// Jump to a  specific address.
    fn jp(&mut self, addr: u16);
    /// Jump to a address relative to PC.
    fn jr(&mut self, r8: i8);
    /// Enable or disable interrupts.
    fn set_ime(&mut self, enabled: bool);
    fn set_halt(&mut self, halted: bool);
    fn stop(&mut self);
}

#[cfg(test)]
mock! {
    pub Cpu16 {}

    impl Cpu16 for Cpu16 {
        fn adv_cycles(&mut self, cycles: u8);
        fn read_r8(&self, reg: Register8) -> u8;
        fn write_r8(&mut self, reg: Register8, value: u8);
        fn read_r16(&self, reg: Register16) -> u16;
        fn write_r16(&mut self, reg: Register16, value: u16);
        fn bus_read(&mut self, addr: u16) -> u8;
        fn bus_write(&mut self, addr: u16, value: u8);
        fn read_pc(&mut self) -> u8;
        fn set_flags(&mut self, z: Option<bool>, n: Option<bool>, h: Option<bool>, c: Option<bool>);
        /// Z,N,H,C in order.
        fn flags(&self) -> (bool, bool, bool, bool);
        fn stack_push(&mut self, value: u8);
        fn stack_pop(&mut self) -> u8;
        fn stack_push_pc(&mut self);
        /// Jump to a  specific address.
        fn jp(&mut self, addr: u16);
        /// Jump to a address relative to PC.
        fn jr(&mut self, r8: i8);
        /// Enable or disable interrupts.
        fn set_ime(&mut self, enabled: bool);
        fn set_halt(&mut self, halted: bool);
        fn stop(&mut self);
    }
}
