mod instructions;
mod proc;

use instructions::{get_instruction, AddressingMode, InstructionType, Register};
use log::debug;
use proc::{
    proc_add, proc_and, proc_call, proc_dec, proc_di, proc_ei, proc_halt, proc_inc, proc_jp,
    proc_jr, proc_ld, proc_or, proc_pop, proc_push, proc_ret, proc_reti, proc_rst, proc_stop,
    proc_sub, proc_xor,
};

pub struct Cpu<BUS>
where
    BUS: io::IO,
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

    /// Set by instructions(EI, RETI, DI).
    pub interrupt_master_enable: bool,
    /// R/W. Set the bit to be 1 if interrupt
    /// is enabled
    ///
    /// - Bit 4, Joypad
    /// - Bit 3, Serial
    /// - Bit 2, Timer
    /// - Bit 1, LCD STAT
    /// - Bit 0, Vertical Blank
    pub interrupt_enable: u8,
    /// R/W. Set the bit to be 1 if interrupt
    /// is requested.
    ///
    /// - Bit 4, Joypad
    /// - Bit 3, Serial
    /// - Bit 2, Timer
    /// - Bit 1, LCD STAT
    /// - Bit 0, Vertical Blank
    pub interrupt_flags: u8,
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
    BUS: io::IO,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cpu")
            .field("SP", &format_args!("{:#04X}", &self.sp))
            .field("PC", &format_args!("{:#04X}", &self.pc))
            .field("AF", &format_args!("{:#04X}", &self.af()))
            .field("BC", &format_args!("{:#04X}", &self.bc()))
            .field("DE", &format_args!("{:#04X}", &self.de()))
            .field("HL", &format_args!("{:#04X}", &self.hl()))
            .field("IME", &self.interrupt_master_enable)
            .field("IE", &format_args!("{:#08b}", &self.interrupt_enable))
            .field("IF", &format_args!("{:#08b}", &self.interrupt_flags))
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

fn cycles(c: u8) {
    todo!()
}

impl<BUS> Cpu<BUS>
where
    BUS: io::IO,
{
    pub fn new(bus: BUS) -> Self {
        // TODO init
        Self {
            reg_a: 0xB0,
            reg_f: 0x01,
            reg_b: 0x13,
            reg_c: 0,
            reg_d: 0xB8,
            reg_e: 0,
            reg_h: 0x4D,
            reg_l: 0x01,
            sp: 0xFFFE,
            pc: 0x100,

            interrupt_enable: 0,
            interrupt_flags: 0,
            interrupt_master_enable: false,
            halted: false,
            stopped: false,
            bus,
        }
    }

    fn bus_read(&self, addr: u16) -> u8 {
        if addr == 0xFFFF {
            return self.interrupt_enable;
        }

        self.bus.read(addr)
    }

    fn bus_write(&mut self, addr: u16, value: u8) {
        if addr == 0xFFFF {
            self.interrupt_enable = value;
        } else {
            self.bus.write(addr, value);
        }
    }

    pub(crate) fn fetch_data(&mut self, am: &AddressingMode) -> u16 {
        match am {
            AddressingMode::Direct(register) => match register {
                Register::A => self.reg_a as u16,
                Register::F => self.reg_f as u16,
                Register::B => self.reg_b as u16,
                Register::C => self.reg_c as u16,
                Register::D => self.reg_d as u16,
                Register::E => self.reg_e as u16,
                Register::H => self.reg_h as u16,
                Register::L => self.reg_l as u16,
                Register::AF => self.af(),
                Register::BC => self.bc(),
                Register::DE => self.de(),
                Register::HL => self.hl(),
                Register::SP => self.sp,
            },
            AddressingMode::Indirect(register) => match register {
                Register::BC => self.bus_read(self.bc()) as u16,
                Register::DE => self.bus_read(self.de()) as u16,
                Register::HL => self.bus_read(self.hl()) as u16,
                _ => unreachable!("Only BC, DE, HL is valid for RegisterIndirect"),
            },
            AddressingMode::PC1 => self.read_pc() as u16,
            AddressingMode::PC2 => self.read_pc2(),
        }
    }

    pub(crate) fn write_data(&mut self, am: &AddressingMode, address: u16, value: u16) {
        match am {
            AddressingMode::Direct(register) => match register {
                Register::A => self.reg_a = value as u8,
                Register::F => self.reg_f = value as u8,
                Register::B => self.reg_b = value as u8,
                Register::C => self.reg_c = value as u8,
                Register::D => self.reg_d = value as u8,
                Register::E => self.reg_e = value as u8,
                Register::H => self.reg_h = value as u8,
                Register::L => self.reg_l = value as u8,
                Register::BC => self.set_bc(value),
                Register::DE => self.set_de(value),
                Register::HL => self.set_hl(value),
                Register::SP => self.sp = value,
                _ => unreachable!(),
            },
            AddressingMode::Indirect(register) => match register {
                Register::BC => self.bus_write(self.bc(), value as u8),
                Register::DE => self.bus_write(self.de(), value as u8),
                Register::HL => self.bus_write(self.hl(), value as u8),
                _ => unreachable!("Only BC, DE, HL is valid for RegisterIndirect"),
            },
            AddressingMode::PC1 => {
                self.bus_write(address, value as u8);
            }
            AddressingMode::PC2 => {
                self.bus_write(address, value as u8);
                self.bus_write(address + 1, (value >> 8) as u8);
            }
        }
    }

    #[inline]
    pub fn af(&self) -> u16 {
        convert_u8_tuple_to_u16(self.reg_a, self.reg_f)
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

    fn set_flags(&mut self, z: Option<bool>, n: Option<bool>, h: Option<bool>, c: Option<bool>) {
        /// Turn on or off for specific bit.
        fn set_flag(value: u8, flag: Option<bool>, bit: u8) -> u8 {
            debug_assert!(bit <= 8 && bit >= 1);

            match flag {
                None => value,
                Some(true) => value | (1u8 << (bit - 1)),
                Some(false) => value & (!(1u8 << (bit - 1))),
            }
        }

        let v = set_flag(self.reg_f, c, 4);
        let v = set_flag(v, h, 5);
        let v = set_flag(v, n, 6);
        self.reg_f = set_flag(v, z, 7);
    }

    #[inline]
    fn flag_z(&self) -> bool {
        (self.reg_f & (1 << 6)) != 0
    }

    #[inline]
    fn flag_n(&self) -> bool {
        (self.reg_f & (1 << 5)) != 0
    }

    #[inline]
    fn flag_h(&self) -> bool {
        (self.reg_f & (1 << 4)) != 0
    }

    #[inline]
    fn flag_c(&self) -> bool {
        (self.reg_f & (1 << 3)) != 0
    }

    fn inc_pc(&mut self) -> u16 {
        let pc = self.pc;
        self.pc += 1;

        pc
    }

    #[inline]
    fn inc_hl(&mut self) {
        self.set_hl(self.hl().wrapping_add(1));
    }

    #[inline]
    fn dec_hl(&mut self) {
        self.set_hl(self.hl().wrapping_sub(1));
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

    fn stack_push(&mut self, value: u8) {
        self.sp -= 1;
        self.bus_write(self.sp, value);
    }

    fn stack_push2(&mut self, value: u16) {
        self.stack_push((value >> 8) as u8);
        self.stack_push(value as u8);
    }

    fn stack_pop(&mut self) -> u8 {
        let value = self.bus_read(self.sp);
        self.sp += 1;

        value
    }

    fn stack_pop2(&mut self) -> u16 {
        let lo = self.stack_pop();
        let hi = self.stack_pop();

        convert_u8_tuple_to_u16(hi, lo)
    }

    /// Interrupts are checked before fetching a new instruction.
    /// If any IF flag and the corresponding IE flag are both '1',
    /// and IME is set to '1' too, the CPU will push the current PC
    /// into the stack, will jump to the corresponding interrupt vector
    /// and set IME to '0'. If IME is '0', this won't happen. This flags
    /// are only cleared when the CPU jumps to an interrupt vector because
    /// of an interrupt(or IF is written manually).
    pub fn handle_interrupts(&mut self) {
        // todo!()
    }

    pub fn execute(&mut self) {
        let opcode = self.read_pc();
        let inst = get_instruction(opcode);
        debug!("{:?}", inst);

        match inst.ty {
            InstructionType::NOP => {
                //
            }
            InstructionType::LD => {
                proc_ld(self, inst);
            }
            InstructionType::INC => {
                proc_inc(self, inst);
            }
            InstructionType::DEC => {
                proc_dec(self, inst);
            }
            InstructionType::JP => {
                proc_jp(self, inst);
            }
            InstructionType::JR => {
                proc_jr(self, inst);
            }
            InstructionType::ADD => {
                proc_add(self, inst);
            }
            InstructionType::CALL => {
                proc_call(self, inst);
            }
            InstructionType::PUSH => {
                proc_push(self, inst);
            }
            InstructionType::POP => {
                proc_pop(self, inst);
            }
            InstructionType::RET => {
                proc_ret(self, inst);
            }
            InstructionType::RETI => {
                proc_reti(self, inst);
            }
            InstructionType::RST => {
                proc_rst(self, inst);
            }
            InstructionType::SUB => {
                proc_sub(self, inst);
            }
            InstructionType::AND => {
                proc_and(self, inst);
            }
            InstructionType::OR => {
                proc_or(self, inst);
            }
            InstructionType::XOR => {
                proc_xor(self, inst);
            }
            InstructionType::STOP => {
                proc_stop(self);
            }
            InstructionType::DI => {
                proc_di(self);
            }
            InstructionType::EI => {
                proc_ei(self);
            }
            InstructionType::HALT => {
                proc_halt(self);
            }
        }

        debug!("{:?}", &self);
    }
}
