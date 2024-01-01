mod alu;
mod cpu16;
mod instruction;
mod interrupt;
mod proc;

use cpu16::Cpu16;
use gb_shared::{is_bit_set, set_bits, unset_bits};
use instruction::{get_instruction, AddressingMode, Instruction};
use interrupt::INTERRUPTS;

impl<BUS: gb_shared::Memory> Cpu16 for Cpu<BUS> {
    fn fetch_data(&mut self, am: &AddressingMode) -> u16 {
        match am {
            AddressingMode::Direct_A => self.reg_a as u16,
            AddressingMode::Direct_B => self.reg_b as u16,
            AddressingMode::Direct_C => self.reg_c as u16,
            AddressingMode::Direct_D => self.reg_d as u16,
            AddressingMode::Direct_E => self.reg_e as u16,
            AddressingMode::Direct_H => self.reg_h as u16,
            AddressingMode::Direct_L => self.reg_l as u16,
            AddressingMode::Direct_AF => self.af(),
            AddressingMode::Direct_BC => self.bc(),
            AddressingMode::Direct_DE => self.de(),
            AddressingMode::Direct_HL => self.hl(),
            AddressingMode::Direct_SP => self.sp,
            AddressingMode::Indirect_BC => self.bus_read(self.bc()) as u16,
            AddressingMode::Indirect_DE => self.bus_read(self.de()) as u16,
            AddressingMode::Indirect_HL => self.bus_read(self.hl()) as u16,
            AddressingMode::Eight => self.read_pc() as u16,
            AddressingMode::Sixteen => self.read_pc2(),
        }
    }

    fn write_data(&mut self, am: &AddressingMode, address: u16, value: u16) {
        match am {
            AddressingMode::Direct_A => self.reg_a = value as u8,
            AddressingMode::Direct_B => self.reg_b = value as u8,
            AddressingMode::Direct_C => self.reg_c = value as u8,
            AddressingMode::Direct_D => self.reg_d = value as u8,
            AddressingMode::Direct_E => self.reg_e = value as u8,
            AddressingMode::Direct_H => self.reg_h = value as u8,
            AddressingMode::Direct_L => self.reg_l = value as u8,
            AddressingMode::Direct_AF => self.set_af(value),
            AddressingMode::Direct_BC => self.set_bc(value),
            AddressingMode::Direct_DE => self.set_de(value),
            AddressingMode::Direct_HL => self.set_hl(value),
            AddressingMode::Direct_SP => self.sp = value,
            AddressingMode::Indirect_BC => self.bus_write(self.bc(), value as u8),
            AddressingMode::Indirect_DE => self.bus_write(self.de(), value as u8),
            AddressingMode::Indirect_HL => self.bus_write(self.hl(), value as u8),
            AddressingMode::Eight => {
                self.bus_write(address, value as u8);
            }
            AddressingMode::Sixteen => {
                self.bus_write(address, value as u8);
                self.bus_write(address.wrapping_add(1), (value >> 8) as u8);
            }
        }
    }

    fn bus_read(&self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    fn bus_write(&mut self, addr: u16, value: u8) {
        self.bus.write(addr, value);
    }

    fn set_flags(&mut self, z: Option<bool>, n: Option<bool>, h: Option<bool>, c: Option<bool>) {
        /// Turn on or off for specific bit.
        fn set_flag(value: u8, flag: Option<bool>, bit: u8) -> u8 {
            match flag {
                None => value,
                Some(true) => set_bits!(value, bit),
                Some(false) => unset_bits!(value, bit),
            }
        }

        let v = set_flag(self.reg_f, c, 4);
        let v = set_flag(v, h, 5);
        let v = set_flag(v, n, 6);
        self.reg_f = set_flag(v, z, 7);
    }

    fn flags(&self) -> (bool, bool, bool, bool) {
        (self.flag_z(), self.flag_n(), self.flag_h(), self.flag_c())
    }

    fn stack_push(&mut self, value: u8) {
        self.sp = self.sp.wrapping_sub(1);
        self.bus_write(self.sp, value);
    }

    fn stack_pop(&mut self) -> u8 {
        let value = self.bus_read(self.sp);
        self.sp = self.sp.wrapping_add(1);

        value
    }

    fn stack_push_pc(&mut self) {
        let pc = self.pc;
        self.stack_push2(pc);
    }

    fn jp(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn jr(&mut self, r8: i8) {
        self.pc = self.pc.wrapping_add_signed(r8 as i16);
    }

    fn inc_dec_hl(&mut self, inc: bool) {
        if inc {
            self.set_hl(self.hl().wrapping_add(1));
        } else {
            self.set_hl(self.hl().wrapping_sub(1));
        }
    }

    fn set_ime(&mut self, enabled: bool) {
        self.ime = enabled;
    }

    fn set_halt(&mut self, halted: bool) {
        self.halted = halted;
    }

    fn stop(&mut self) {
        self.stopped = true;
    }
}

pub struct Cpu<BUS>
where
    BUS: gb_shared::Memory,
{
    /// Accumulator register
    pub reg_a: u8,
    /// Flags register
    /// Bit 7: z(Zero flag)
    /// Bit 6: n(Subtraction flag(BCD))
    /// Bit 5: h(Half Carry flag(BFD))
    /// Bit c: Carry flag
    pub reg_f: u8,

    pub reg_b: u8,
    pub reg_c: u8,

    pub reg_d: u8,
    pub reg_e: u8,

    pub reg_h: u8,
    pub reg_l: u8,

    /// Stack pointer
    pub sp: u16,
    /// Program counter
    pub pc: u16,

    /// Interrupt master enable.
    /// Set by instructions(EI, RETI, DI).
    pub ime: bool,
    /// Set by instruction HALT
    ///
    /// HALT mode is exited when a flag in register IF is set and
    /// the corresponding flag in IE is also set, regardless of
    /// the value of IME. The only difference is that IME='1' will
    /// make CPU jump to the interrupt vector(and clear the IF flag),
    /// while IME='0' will only make the CPU continue executing
    /// instructions, but the jump won't be performed(and the IF flag
    /// won't be cleared).
    pub halted: bool,
    /// Set by instruction STOP
    pub stopped: bool,

    bus: BUS,
    // TODO
}

impl<BUS> core::fmt::Debug for Cpu<BUS>
where
    BUS: gb_shared::Memory,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cpu")
            .field("SP", &format_args!("{:#04X}", &self.sp))
            .field("PC", &format_args!("{:#04X}", &self.pc))
            .field("AF", &format_args!("{:#04X}", &self.af()))
            .field("BC", &format_args!("{:#04X}", &self.bc()))
            .field("DE", &format_args!("{:#04X}", &self.de()))
            .field("HL", &format_args!("{:#04X}", &self.hl()))
            .field("IME", &self.ime)
            .field("HALTED", &self.halted)
            .field("STOPPED", &self.stopped)
            .finish()
    }
}

#[inline]
fn convert_u16_to_u8_tuple(value: u16) -> (u8, u8) {
    let hi = (value & 0xFF00) >> 8;
    let lo = value & 0x00FF;

    (hi as u8, lo as u8)
}

#[inline]
fn convert_u8_tuple_to_u16(hi: u8, lo: u8) -> u16 {
    ((hi as u16) << 8) | (lo as u16)
}

impl<BUS> Cpu<BUS>
where
    BUS: gb_shared::Memory,
{
    pub fn new(bus: BUS) -> Self {
        // TODO init
        Self {
            reg_a: 0x01,
            reg_f: 0xB0,
            reg_b: 0,
            reg_c: 0x13,
            reg_d: 0,
            reg_e: 0xD8,
            reg_h: 0x01,
            reg_l: 0x4D,
            sp: 0xFFFE,
            pc: 0x100,

            ime: false,
            halted: false,
            stopped: false,
            bus,
        }
    }

    #[inline]
    pub fn af(&self) -> u16 {
        convert_u8_tuple_to_u16(self.reg_a, self.reg_f)
    }

    #[inline]
    fn set_af(&mut self, value: u16) {
        let (hi, lo) = convert_u16_to_u8_tuple(value);

        self.reg_a = hi;
        self.reg_f = lo;
    }

    #[inline]
    pub fn bc(&self) -> u16 {
        convert_u8_tuple_to_u16(self.reg_b, self.reg_c)
    }

    fn set_bc(&mut self, value: u16) {
        let (hi, lo) = convert_u16_to_u8_tuple(value);

        self.reg_b = hi;
        self.reg_c = lo;
    }

    #[inline]
    pub fn de(&self) -> u16 {
        convert_u8_tuple_to_u16(self.reg_d, self.reg_e)
    }

    fn set_de(&mut self, value: u16) {
        let (hi, lo) = convert_u16_to_u8_tuple(value);

        self.reg_d = hi;
        self.reg_e = lo;
    }

    #[inline]
    pub fn hl(&self) -> u16 {
        convert_u8_tuple_to_u16(self.reg_h, self.reg_l)
    }

    fn set_hl(&mut self, value: u16) {
        let (hi, lo) = convert_u16_to_u8_tuple(value);

        self.reg_h = hi;
        self.reg_l = lo;
    }

    #[inline]
    fn flag_z(&self) -> bool {
        is_bit_set!(self.reg_f, 7)
    }

    #[inline]
    fn flag_n(&self) -> bool {
        is_bit_set!(self.reg_f, 6)
    }

    #[inline]
    fn flag_h(&self) -> bool {
        is_bit_set!(self.reg_f, 5)
    }

    #[inline]
    fn flag_c(&self) -> bool {
        is_bit_set!(self.reg_f, 4)
    }

    fn inc_pc(&mut self) -> u16 {
        let pc = self.pc;
        self.pc = self.pc.wrapping_add(1);

        pc
    }

    #[inline]
    fn read_pc(&mut self) -> u8 {
        let addr = self.inc_pc();
        self.bus_read(addr)
    }

    #[inline]
    fn read_pc2(&mut self) -> u16 {
        let lo = self.read_pc();
        let hi = self.read_pc();

        convert_u8_tuple_to_u16(hi, lo)
    }

    /// Push current PC to stack, and jump to corresponding
    /// interrupt handler address.
    pub fn handle_interrupts(&mut self) {
        // TODO abstract interrupts RW
        let interrupt_flag = self.bus_read(0xFF0F);
        let interrupt_enable = self.bus_read(0xFFFF);

        if let Some(interrupt_source) = INTERRUPTS
            .iter()
            .find(|it| (interrupt_flag & it.flag) != 0 && (interrupt_enable & it.flag) != 0)
        {
            self.stack_push_pc();
            self.jp(interrupt_source.handler_address);
            self.bus_write(0xFF0F, interrupt_flag & (!interrupt_source.flag));
            self.set_halt(false);
            // Interrupt handler can let CPU continue to handle
            // interrupts via RETI instruction.
            self.set_ime(false);
        }
    }

    pub fn step(&mut self) -> u8 {
        let opcode = self.read_pc();
        let inst = get_instruction(opcode);
        log::debug!("{:?}", inst);

        4 + match inst {
            Instruction::NONE => {
                panic!("No such instruction");
            }
            Instruction::NOP => 4,
            Instruction::LD(addr1, addr2) => proc::proc_ld(self, opcode, addr1, addr2),
            Instruction::INC(addr) => proc::proc_inc(self, opcode, addr),
            Instruction::DEC(addr) => proc::proc_dec(self, opcode, addr),
            Instruction::JR(cond) => proc::proc_jr(self, opcode, cond),
            Instruction::JP(cond, addr) => proc::proc_jp(self, opcode, cond, addr),
            Instruction::CALL(cond) => proc::proc_call(self, opcode, cond),
            Instruction::ADD(addr1, addr2) => proc::proc_add(self, opcode, addr1, addr2),
            Instruction::ADC(addr) => proc::proc_adc(self, opcode, addr),
            Instruction::SUB(addr) => proc::proc_sub(self, opcode, addr),
            Instruction::SBC(addr) => proc::proc_sbc(self, opcode, addr),
            Instruction::PUSH(addr) => proc::proc_push(self, opcode, addr),
            Instruction::POP(addr) => proc::proc_pop(self, opcode, addr),
            Instruction::RET(cond) => proc::proc_ret(self, opcode, cond),
            Instruction::RETI => proc::proc_reti(self, opcode),
            Instruction::RST => proc::proc_rst(self, opcode),
            Instruction::AND(addr) => proc::proc_and(self, opcode, addr),
            Instruction::OR(addr) => proc::proc_or(self, opcode, addr),
            Instruction::XOR(addr) => proc::proc_xor(self, opcode, addr),
            Instruction::STOP => proc::proc_stop(self, opcode),
            Instruction::DI => proc::proc_di(self, opcode),
            Instruction::EI => proc::proc_ei(self, opcode),
            Instruction::HALT => proc::proc_halt(self, opcode),
            Instruction::RLA => proc::proc_rla(self, opcode),
            Instruction::RRA => proc::proc_rra(self, opcode),
            Instruction::RLCA => proc::proc_rlca(self, opcode),
            Instruction::RRCA => proc::proc_rrca(self, opcode),
            Instruction::DAA => proc::proc_daa(self, opcode),
            Instruction::CPL => proc::proc_cpl(self, opcode),
            Instruction::SCF => proc::proc_scf(self, opcode),
            Instruction::CCF => proc::proc_ccf(self, opcode),
            Instruction::CP(addr) => proc::proc_cp(self, opcode, addr),
            Instruction::CB => proc::proc_cb(self),
        }
    }
}

#[cfg(test)]
use gb_shared::Memory;
#[cfg(test)]
use mockall::mock;

#[cfg(test)]
mock! {
    pub Bus {}

    impl Memory for Bus {
        fn write(&mut self, addr: u16, value: u8) {
            // Noop
        }
        fn read(&self, addr: u16) -> u8 {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    mod flags {
        use super::*;

        #[test]
        fn read_flags() {
            let mut cpu = Cpu::new(MockBus::new());
            cpu.reg_f = 0b1010_0000;

            assert_eq!(cpu.flags(), (true, false, true, false));
        }

        #[test]
        fn set_flags() {
            let mut cpu = Cpu::new(MockBus::new());
            cpu.reg_f = 0b1010_0000;

            cpu.set_flags(Some(false), Some(true), Some(false), Some(true));

            assert_eq!(cpu.flags(), (false, true, false, true));
        }
    }

    mod registers {
        use super::*;

        #[test]
        fn cpu_initial_register_value() {
            let cpu = Cpu::new(MockBus::new());

            assert_ne!(cpu.af(), 0x010B);
            assert_eq!(cpu.bc(), 0x0013);
            assert_eq!(cpu.de(), 0x00D8);
            assert_eq!(cpu.hl(), 0x014D);
            assert_eq!(cpu.sp, 0xFFFE);
            assert_eq!(cpu.pc, 0x0100);
        }
    }

    mod stack {

        use std::ops::Deref;

        use super::*;

        fn setup_stack_bus() -> (MockBus, Rc<RefCell<Vec<(u16, u8)>>>) {
            let mut mock = MockBus::new();
            let stack = Rc::new(RefCell::new(Vec::new()));

            let stack_writer = stack.clone();
            let stack_reader = stack.clone();
            mock.expect_write().returning_st(move |addr, value| {
                stack_writer.borrow_mut().push((addr, value));
            });
            mock.expect_read().returning_st(move |addr| {
                let (addr_, data) = stack_reader.borrow_mut().pop().unwrap();
                assert_eq!(addr_, addr);

                data
            });

            (mock, stack)
        }

        #[test]
        fn stack_push() {
            let sp = 0xFFFE;
            let data = 0x12;

            let mut mock = MockBus::new();
            mock.expect_write().with(eq(sp - 1), eq(data)).once().return_const(());

            let mut cpu = Cpu::new(mock);
            cpu.sp = sp;

            cpu.stack_push(data);
            assert_eq!(cpu.sp, 0xFFFD);
        }

        #[test]
        fn stack_pop() {
            let sp = 0xFFFE;
            let data = 0x12;

            let mut mock = MockBus::new();
            mock.expect_read().with(eq(sp)).once().return_const(data);

            let mut cpu = Cpu::new(mock);
            cpu.sp = sp;

            assert_eq!(cpu.stack_pop(), data);
            assert_eq!(cpu.sp, 0xFFFF);
        }

        #[test]
        fn stack_push2_pop2() {
            let sp = 0xFFF2;
            let data = 0x1234u16;

            let (mock, stack) = setup_stack_bus();

            let mut cpu = Cpu::new(mock);
            cpu.sp = sp;

            cpu.stack_push2(data);
            assert_eq!(stack.borrow().deref(), &vec![(sp - 1, 0x12), (sp - 2, 0x34)]);
            assert_eq!(cpu.sp, 0xFFF0);

            assert_eq!(data, cpu.stack_pop2());
            assert_eq!(cpu.sp, sp);
        }

        #[test]
        fn stack_push_pc() {
            let pc = 0x1234;
            let sp = 0xFFF2;

            let (mock, stack) = setup_stack_bus();

            let mut cpu = Cpu::new(mock);
            cpu.pc = pc;
            cpu.sp = sp;

            cpu.stack_push_pc();
            assert_eq!(stack.borrow().deref(), &vec![(0xFFF1, 0x12), (0xFFF0, 0x34)]);
            assert_eq!(cpu.sp, 0xFFF0);
        }
    }

    mod rr {
        use super::*;

        #[test]
        fn read() {
            let mut cpu = Cpu::new(MockBus::new());
            cpu.reg_a = 0x12;
            cpu.reg_f = 0x34;
            cpu.reg_b = 0x56;
            cpu.reg_c = 0x78;
            cpu.reg_d = 0x9A;
            cpu.reg_e = 0xBC;
            cpu.reg_h = 0xDE;
            cpu.reg_l = 0xF0;

            assert_eq!(cpu.af(), 0x1234);
            assert_eq!(cpu.bc(), 0x5678);
            assert_eq!(cpu.de(), 0x9ABC);
            assert_eq!(cpu.hl(), 0xDEF0);
        }

        #[test]
        fn write() {
            let mut cpu = Cpu::new(MockBus::new());

            cpu.set_af(0x1234);
            cpu.set_bc(0x5678);
            cpu.set_de(0x9ABC);
            cpu.set_hl(0xDEF0);

            assert_eq!(cpu.reg_a, 0x12);
            assert_eq!(cpu.reg_f, 0x34);
            assert_eq!(cpu.reg_b, 0x56);
            assert_eq!(cpu.reg_c, 0x78);
            assert_eq!(cpu.reg_d, 0x9A);
            assert_eq!(cpu.reg_e, 0xBC);
            assert_eq!(cpu.reg_h, 0xDE);
            assert_eq!(cpu.reg_l, 0xF0);
        }

        #[test]
        fn inc_dec_hl() {
            let mut cpu = Cpu::new(MockBus::new());
            cpu.set_hl(0x1234);

            cpu.inc_dec_hl(true);
            assert_eq!(cpu.hl(), 0x1235);

            cpu.inc_dec_hl(false);
            assert_eq!(cpu.hl(), 0x1234);
        }
    }

    mod pc {
        use super::*;

        #[test]
        fn read_pc() {
            let mut mock = MockBus::new();
            mock.expect_read().with(eq(0x1234)).once().return_const(0x56);

            let mut cpu = Cpu::new(mock);
            cpu.pc = 0x1234;

            let data = cpu.read_pc();
            assert_eq!(data, 0x56);
            assert_eq!(cpu.pc, 0x1235);
        }

        #[test]
        fn read_pc2() {
            let pc = 0x1234;

            let mut mock = MockBus::new();
            let mut count = 0;
            mock.expect_read().returning(move |addr| {
                count += 1;
                if count == 1 {
                    assert_eq!(addr, pc);
                    0x56
                } else {
                    assert_eq!(addr, pc + 1);
                    0x78
                }
            });

            let mut cpu = Cpu::new(mock);
            cpu.pc = pc;
            let data = cpu.read_pc2();
            assert_eq!(cpu.pc, 0x1236);
            assert_eq!(data, 0x7856);
        }
    }

    mod address {
        use super::*;

        #[test]
        fn jp() {
            let mut cpu = Cpu::new(MockBus::new());
            cpu.pc = 0x1234;

            cpu.jp(0x5678);
            assert_eq!(cpu.pc, 0x5678);
        }

        #[test]
        fn jr() {
            let mut cpu = Cpu::new(MockBus::new());
            cpu.pc = 0x1234;

            cpu.jr(-1);
            assert_eq!(cpu.pc, 0x1233);

            cpu.jr(2);
            assert_eq!(cpu.pc, 0x1235);
        }
    }

    mod memory {
        use super::*;
        use crate::instruction::AddressingMode;
        type Am = AddressingMode;

        #[test]
        fn fetch_data() {
            let mut mock_bus = MockBus::new();
            mock_bus.expect_read().returning(|addr| {
                if addr == 0x5678 {
                    // BC
                    0x87
                } else if addr == 0x9ABC {
                    // DE
                    0xCB
                } else if addr == 0xDEF0 {
                    // HL
                    0x0F
                } else if addr == 0x4321 {
                    0x55
                } else if addr == 0x4322 {
                    0x66
                } else if addr == 0x4323 {
                    0x77
                } else {
                    unreachable!()
                }
            });
            let mut cpu = Cpu::new(mock_bus);
            cpu.reg_a = 0x12;
            cpu.reg_f = 0x34;
            cpu.reg_b = 0x56;
            cpu.reg_c = 0x78;
            cpu.reg_d = 0x9A;
            cpu.reg_e = 0xBC;
            cpu.reg_h = 0xDE;
            cpu.reg_l = 0xF0;
            cpu.sp = 0x1122;
            cpu.pc = 0x4321;

            assert_eq!(cpu.fetch_data(&Am::Direct_A), 0x12);
            assert_eq!(cpu.fetch_data(&Am::Direct_B), 0x56);
            assert_eq!(cpu.fetch_data(&Am::Direct_C), 0x78);
            assert_eq!(cpu.fetch_data(&Am::Direct_D), 0x9A);
            assert_eq!(cpu.fetch_data(&Am::Direct_E), 0xBC);
            assert_eq!(cpu.fetch_data(&Am::Direct_H), 0xDE);
            assert_eq!(cpu.fetch_data(&Am::Direct_L), 0xF0);
            assert_eq!(cpu.fetch_data(&Am::Direct_AF), 0x1234);
            assert_eq!(cpu.fetch_data(&Am::Direct_BC), 0x5678);
            assert_eq!(cpu.fetch_data(&Am::Direct_DE), 0x9ABC);
            assert_eq!(cpu.fetch_data(&Am::Direct_HL), 0xDEF0);
            assert_eq!(cpu.fetch_data(&Am::Direct_SP), 0x1122);
            assert_eq!(cpu.fetch_data(&Am::Indirect_BC), 0x87);
            assert_eq!(cpu.fetch_data(&Am::Indirect_DE), 0xCB);
            assert_eq!(cpu.fetch_data(&Am::Indirect_HL), 0x0F);
            assert_eq!(cpu.fetch_data(&Am::Eight), 0x55);
            assert_eq!(cpu.pc, 0x4322);
            assert_eq!(cpu.fetch_data(&Am::Sixteen), 0x7766);
            assert_eq!(cpu.pc, 0x4324);
        }

        #[test]
        fn write_data() {
            let mut mock_bus = MockBus::new();
            mock_bus.expect_write().returning(|addr, value| {
                match addr {
                    0x5678 => {
                        // BC
                        assert_eq!(value, 0x87);
                    }
                    0x9ABC => {
                        // DE
                        assert_eq!(value, 0xCB);
                    }
                    0xDEF0 => {
                        // HL
                        assert_eq!(value, 0x0F);
                    }
                    0x1111 => {
                        assert_eq!(value, 0x55);
                    }
                    0x1112 => {
                        assert_eq!(value, 0x66);
                    }
                    0x1113 => {
                        assert_eq!(value, 0x77);
                    }
                    _ => unreachable!(),
                }
            });
            let mut cpu = Cpu::new(mock_bus);
            cpu.reg_a = 0x12;
            cpu.reg_f = 0x34;
            cpu.reg_b = 0x56;
            cpu.reg_c = 0x78;
            cpu.reg_d = 0x9A;
            cpu.reg_e = 0xBC;
            cpu.reg_h = 0xDE;
            cpu.reg_l = 0xF0;
            cpu.sp = 0x1122;
            cpu.pc = 0x4321;

            cpu.write_data(&Am::Direct_A, 0, 0x21);
            assert_eq!(cpu.reg_a, 0x21);
            cpu.write_data(&Am::Direct_B, 0, 0x65);
            assert_eq!(cpu.reg_b, 0x65);
            cpu.write_data(&Am::Direct_C, 0, 0x87);
            assert_eq!(cpu.reg_c, 0x87);
            cpu.write_data(&Am::Direct_D, 0, 0xA9);
            assert_eq!(cpu.reg_d, 0xA9);
            cpu.write_data(&Am::Direct_E, 0, 0xCB);
            assert_eq!(cpu.reg_e, 0xCB);
            cpu.write_data(&Am::Direct_H, 0, 0xED);
            assert_eq!(cpu.reg_h, 0xED);
            cpu.write_data(&Am::Direct_L, 0, 0x0F);
            assert_eq!(cpu.reg_l, 0x0F);

            cpu.write_data(&Am::Direct_AF, 0, 0x4321);
            assert_eq!(cpu.af(), 0x4321);
            cpu.write_data(&Am::Direct_BC, 0, 0x8765);
            assert_eq!(cpu.bc(), 0x8765);
            cpu.write_data(&Am::Direct_DE, 0, 0xCBA9);
            assert_eq!(cpu.de(), 0xCBA9);
            cpu.write_data(&Am::Direct_HL, 0, 0x0FED);
            assert_eq!(cpu.hl(), 0x0FED);
            cpu.write_data(&Am::Direct_SP, 0, 0x1122);
            assert_eq!(cpu.sp, 0x1122);

            // Reset rr
            cpu.reg_a = 0x12;
            cpu.reg_f = 0x34;
            cpu.reg_b = 0x56;
            cpu.reg_c = 0x78;
            cpu.reg_d = 0x9A;
            cpu.reg_e = 0xBC;
            cpu.reg_h = 0xDE;
            cpu.reg_l = 0xF0;
            cpu.write_data(&Am::Indirect_BC, 0, 0x87);
            cpu.write_data(&Am::Indirect_DE, 0, 0xCB);
            cpu.write_data(&Am::Indirect_HL, 0, 0x0F);

            cpu.write_data(&Am::Eight, 0x1111, 0x55);
            cpu.write_data(&Am::Sixteen, 0x1112, 0x7766);
        }
    }
}
