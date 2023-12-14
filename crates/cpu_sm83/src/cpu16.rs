use crate::instruction::AddressingMode;
#[cfg(test)]
use mockall::{automock, predicate::*};

/// 16 bits CPU
#[cfg_attr(test, automock)]
pub(crate) trait Cpu16 {
    fn fetch_data(&mut self, am: &AddressingMode) -> u16;
    fn write_data(&mut self, am: &AddressingMode, address: u16, value: u16);
    fn bus_read(&self, addr: u16) -> u8;
    fn bus_write(&mut self, addr: u16, value: u8);
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
    /// Increase or decrease HL register.
    fn inc_dec_hl(&mut self, inc: bool);
    /// Enable or disable interrupts.
    fn set_ime(&mut self, enabled: bool);
    fn set_halt(&mut self, halted: bool);
    fn stop(&mut self);
}
