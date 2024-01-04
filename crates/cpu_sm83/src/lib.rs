mod alu;
mod cpu16;
mod interrupt;
mod proc;

use cpu16::{Cpu16, Register16, Register8};
use gb_shared::{is_bit_set, set_bits, unset_bits};
use interrupt::INTERRUPTS;

pub const CPU_PERIOD_NANOS: f64 = 238.418579;

impl<BUS> Cpu16 for Cpu<BUS>
where
    BUS: gb_shared::Memory + gb_shared::Component,
{
    fn adv_cycles(&mut self, cycles: u8) {
        self.cycles = self.cycles.wrapping_add(cycles);
        self.bus.step(cycles);
    }

    fn bus_read(&mut self, addr: u16) -> u8 {
        let value = self.bus.read(addr);
        self.adv_cycles(4);

        value
    }

    fn bus_write(&mut self, addr: u16, value: u8) {
        self.bus.write(addr, value);

        self.adv_cycles(4)
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
        self.stack_push2(self.pc);
    }

    fn jp(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn jr(&mut self, r8: i8) {
        self.pc = self.pc.wrapping_add_signed(r8 as i16);
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

    fn read_r8(&self, reg: Register8) -> u8 {
        match reg {
            Register8::A => self.reg_a,
            Register8::B => self.reg_b,
            Register8::C => self.reg_c,
            Register8::D => self.reg_d,
            Register8::E => self.reg_e,
            Register8::H => self.reg_h,
            Register8::L => self.reg_l,
        }
    }

    fn write_r8(&mut self, reg: Register8, value: u8) {
        match reg {
            Register8::A => self.reg_a = value,
            Register8::B => self.reg_b = value,
            Register8::C => self.reg_c = value,
            Register8::D => self.reg_d = value,
            Register8::E => self.reg_e = value,
            Register8::H => self.reg_h = value,
            Register8::L => self.reg_l = value,
        }
    }

    fn read_r16(&self, reg: Register16) -> u16 {
        match reg {
            Register16::AF => self.af(),
            Register16::BC => self.bc(),
            Register16::DE => self.de(),
            Register16::HL => self.hl(),
            Register16::SP => self.sp,
        }
    }

    fn write_r16(&mut self, reg: Register16, value: u16) {
        match reg {
            Register16::AF => self.set_af(value),
            Register16::BC => self.set_bc(value),
            Register16::DE => self.set_de(value),
            Register16::HL => self.set_hl(value),
            Register16::SP => self.sp = value,
        }
    }

    fn read_pc(&mut self) -> u8 {
        let value = self.bus_read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        value
    }
}

pub struct Cpu<BUS>
where
    BUS: gb_shared::Memory + gb_shared::Component,
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
    pub enabling_ime: bool,
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

    pub cycles: u8,

    bus: BUS,
    // TODO
}

impl<BUS> core::fmt::Debug for Cpu<BUS>
where
    BUS: gb_shared::Memory + gb_shared::Component,
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
    BUS: gb_shared::Memory + gb_shared::Component,
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
            enabling_ime: false,
            halted: false,
            stopped: false,
            cycles: 0,
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
        self.reg_f = lo & 0xF0;
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

    #[inline]
    fn read_pc2(&mut self) -> u16 {
        let lo = self.read_pc();
        let hi = self.read_pc();

        convert_u8_tuple_to_u16(hi, lo)
    }

    #[inline]
    fn bus_write_16(&mut self, addr: u16, value: u16) {
        self.bus_write(addr, value as u8);
        self.bus_write(addr.wrapping_add(1), (value >> 8) as u8);
    }

    pub fn finish_cycles(&mut self) -> u8 {
        let cycles = self.cycles;
        self.cycles = 0;

        cycles
    }

    fn read_reg(&mut self, loc: u8) -> u8 {
        match loc {
            0 => self.reg_b,
            1 => self.reg_c,
            2 => self.reg_d,
            3 => self.reg_e,
            4 => self.reg_h,
            5 => self.reg_l,
            // (HL)
            6 => self.bus_read(self.hl()),
            7 => self.reg_a,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn inc_hl(&mut self) {
        self.set_hl(self.hl().wrapping_add(1));
    }

    #[inline]
    fn dec_hl(&mut self) {
        self.set_hl(self.hl().wrapping_sub(1));
    }

    /// Push current PC to stack, and jump to corresponding
    /// interrupt handler address.
    pub fn handle_interrupts(&mut self) {
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

    pub fn step(&mut self) {
        let opcode = self.read_pc();

        match opcode {
            0x00 => {
                // NOP
            }
            0x01 => {
                // LD BC,d16
                let d16 = self.read_pc2();
                self.set_bc(d16);
            }
            0x02 => {
                // LD (BC),A
                self.bus_write(self.bc(), self.reg_a);
            }
            0x03 => {
                // INC BC
                self.set_bc(alu::inc::alu_inc_16(self.bc()));

                self.adv_cycles(4);
            }
            0x04 => {
                // INC B
                let (value, z, h) = alu::inc::alu_inc_8(self.reg_b);
                self.reg_b = value;
                self.set_flags(Some(z), Some(false), Some(h), None);
            }
            0x05 => {
                // DEC B
                let (value, z, h) = alu::dec::alu_dec_8(self.reg_b);
                self.reg_b = value;
                self.set_flags(Some(z), Some(true), Some(h), None);
            }
            0x06 => {
                // LD B,d8
                self.reg_b = self.read_pc();
            }
            0x07 => {
                // RLCA
                let (value, c) = alu::rlca::alu_rlca(self.reg_a);
                self.reg_a = value;
                self.set_flags(Some(false), Some(false), Some(false), Some(c));
            }
            0x08 => {
                // LD (a16),SP
                let addr = self.read_pc2();
                self.bus_write_16(addr, self.sp);
            }
            0x09 => {
                // ADD HL,BC
                let (value, h, c) = alu::add::alu_add_16(self.hl(), self.bc());
                self.set_hl(value);
                self.set_flags(None, Some(false), Some(h), Some(c));

                self.adv_cycles(4);
            }
            0x0A => {
                // LD A,(BC)
                self.reg_a = self.bus_read(self.bc());
            }
            0x0B => {
                // DEC BC
                self.set_bc(alu::dec::alu_dec_16(self.bc()));

                self.adv_cycles(4);
            }
            0x0C => {
                // INC C
                let (value, z, h) = alu::inc::alu_inc_8(self.reg_c);
                self.reg_c = value;
                self.set_flags(Some(z), Some(false), Some(h), None);
            }
            0x0D => {
                // DEC C
                let (value, z, h) = alu::dec::alu_dec_8(self.reg_c);
                self.reg_c = value;
                self.set_flags(Some(z), Some(true), Some(h), None);
            }
            0x0E => {
                // LD C,d8
                self.reg_c = self.read_pc();
            }
            0x0F => {
                // RRCA
                let (value, c) = alu::rrca::alu_rrca(self.reg_a);
                self.reg_a = value;
                self.set_flags(Some(false), Some(false), Some(false), Some(c));
            }
            0x10 => {
                // STOP
                self.stop();
            }
            0x11 => {
                // LD DE,d16
                let d16 = self.read_pc2();
                self.set_de(d16);
            }
            0x12 => {
                // LD (DE),A
                self.bus_write(self.de(), self.reg_a);
            }
            0x13 => {
                // INC DE
                self.set_de(alu::inc::alu_inc_16(self.de()));
                self.adv_cycles(4);
            }
            0x14 => {
                // INC D
                let (value, z, h) = alu::inc::alu_inc_8(self.reg_d);
                self.reg_d = value;
                self.set_flags(Some(z), Some(false), Some(h), None);
            }
            0x15 => {
                // DEC D
                let (value, z, h) = alu::dec::alu_dec_8(self.reg_d);
                self.reg_d = value;
                self.set_flags(Some(z), Some(true), Some(h), None);
            }
            0x16 => {
                // LD D,d8
                self.reg_d = self.read_pc();
            }
            0x17 => {
                // RLA
                let (value, c) = alu::rla::alu_rla(self.reg_a, self.flag_c());
                self.reg_a = value;
                self.set_flags(Some(false), Some(false), Some(false), Some(c));
            }
            0x18 => {
                // JR r8
                let r8 = self.read_pc() as i8;
                self.jr(r8);
                self.adv_cycles(4);
            }
            0x19 => {
                // ADD HL,DE
                let (value, h, c) = alu::add::alu_add_16(self.hl(), self.de());
                self.set_hl(value);
                self.set_flags(None, Some(false), Some(h), Some(c));
                self.adv_cycles(4);
            }
            0x1A => {
                // LD A,(DE)
                self.reg_a = self.bus_read(self.de());
            }
            0x1B => {
                // DEC DE
                self.set_de(alu::dec::alu_dec_16(self.de()));
                self.adv_cycles(4);
            }
            0x1C => {
                // INC E
                let (value, z, h) = alu::inc::alu_inc_8(self.reg_e);
                self.reg_e = value;
                self.set_flags(Some(z), Some(false), Some(h), None);
            }
            0x1D => {
                // DEC E
                let (value, z, h) = alu::dec::alu_dec_8(self.reg_e);
                self.reg_e = value;
                self.set_flags(Some(z), Some(true), Some(h), None);
            }
            0x1E => {
                // LD E,d8
                self.reg_e = self.read_pc();
            }
            0x1F => {
                // RRA
                let (value, c) = alu::rra::alu_rra(self.reg_a, self.flag_c());
                self.reg_a = value;
                self.set_flags(Some(false), Some(false), Some(false), Some(c));
            }
            0x20 => {
                // JR NZ,r8
                let r8 = self.read_pc() as i8;
                if !self.flag_z() {
                    self.jr(r8);
                    self.adv_cycles(4);
                }
            }
            0x21 => {
                // LD HL,d16
                let d16 = self.read_pc2();
                self.set_hl(d16);
            }
            0x22 => {
                // LD (HL+),A
                self.bus_write(self.hl(), self.reg_a);
                self.inc_hl();
            }
            0x23 => {
                // INC HL
                self.set_hl(alu::inc::alu_inc_16(self.hl()));
                self.adv_cycles(4);
            }
            0x24 => {
                // INC H
                let (value, z, h) = alu::inc::alu_inc_8(self.reg_h);
                self.reg_h = value;
                self.set_flags(Some(z), Some(false), Some(h), None);
            }
            0x25 => {
                // DEC H
                let (value, z, h) = alu::dec::alu_dec_8(self.reg_h);
                self.reg_h = value;
                self.set_flags(Some(z), Some(true), Some(h), None);
            }
            0x26 => {
                // LD H,d8
                self.reg_h = self.read_pc();
            }
            0x27 => {
                // DAA
                let (value, z, c) =
                    alu::daa::alu_daa(self.reg_a, self.flag_n(), self.flag_h(), self.flag_c());
                self.reg_a = value;
                self.set_flags(Some(z), None, Some(false), Some(c));
            }
            0x28 => {
                // JR Z,r8
                let r8 = self.read_pc() as i8;
                if self.flag_z() {
                    self.jr(r8);
                    self.adv_cycles(4);
                }
            }
            0x29 => {
                // ADD HL,HL
                let (value, h, c) = alu::add::alu_add_16(self.hl(), self.hl());
                self.set_hl(value);
                self.set_flags(None, Some(false), Some(h), Some(c));
                self.adv_cycles(4);
            }
            0x2A => {
                // LD A,(HL+)
                self.reg_a = self.bus_read(self.hl());
                self.inc_hl();
            }
            0x2B => {
                // DEC HL
                self.set_hl(alu::dec::alu_dec_16(self.hl()));
                self.adv_cycles(4);
            }
            0x2C => {
                // INC L
                let (value, z, h) = alu::inc::alu_inc_8(self.reg_l);
                self.reg_l = value;
                self.set_flags(Some(z), Some(false), Some(h), None);
            }
            0x2D => {
                // DEC L
                let (value, z, h) = alu::dec::alu_dec_8(self.reg_l);
                self.reg_l = value;
                self.set_flags(Some(z), Some(true), Some(h), None);
            }
            0x2E => {
                // LD L,d8
                self.reg_l = self.read_pc();
            }
            0x2F => {
                // CPL
                self.reg_a = !self.reg_a;
                self.set_flags(None, Some(true), Some(true), None);
            }
            0x30 => {
                // JR NC,r8
                let r8 = self.read_pc() as i8;
                if !self.flag_c() {
                    self.jr(r8);
                    self.adv_cycles(4);
                }
            }
            0x31 => {
                // LD SP,d16
                self.sp = self.read_pc2();
            }
            0x32 => {
                // LD (HL-),A
                self.bus_write(self.hl(), self.reg_a);
                self.dec_hl();
            }
            0x33 => {
                // INC SP
                self.sp = alu::inc::alu_inc_16(self.sp);
                self.adv_cycles(4);
            }
            0x34 => {
                // INC (HL)
                let (value, z, h) = alu::inc::alu_inc_8(self.bus_read(self.hl()));
                self.bus_write(self.hl(), value);
                self.set_flags(Some(z), Some(false), Some(h), None);
            }
            0x35 => {
                // DEC (HL)
                let (value, z, h) = alu::dec::alu_dec_8(self.bus_read(self.hl()));
                self.bus_write(self.hl(), value);
                self.set_flags(Some(z), Some(true), Some(h), None);
            }
            0x36 => {
                // LD (HL),d8
                let d8 = self.read_pc();
                self.bus_write(self.hl(), d8);
            }
            0x37 => {
                // SCF
                self.set_flags(None, Some(false), Some(false), Some(true));
            }
            0x38 => {
                // JR C,r8
                let r8 = self.read_pc() as i8;
                if self.flag_c() {
                    self.jr(r8);
                    self.adv_cycles(4);
                }
            }
            0x39 => {
                // ADD HL,SP
                let (value, h, c) = alu::add::alu_add_16(self.hl(), self.sp);
                self.set_hl(value);
                self.set_flags(None, Some(false), Some(h), Some(c));
                self.adv_cycles(4);
            }
            0x3A => {
                // LD A,(HL-)
                self.reg_a = self.bus_read(self.hl());
                self.dec_hl();
            }
            0x3B => {
                // DEC SP
                self.sp = alu::dec::alu_dec_16(self.sp);
                self.adv_cycles(4);
            }
            0x3C => {
                // INC A
                let (value, z, h) = alu::inc::alu_inc_8(self.reg_a);
                self.reg_a = value;
                self.set_flags(Some(z), Some(false), Some(h), None);
            }
            0x3D => {
                // DEC A
                let (value, z, h) = alu::dec::alu_dec_8(self.reg_a);
                self.reg_a = value;
                self.set_flags(Some(z), Some(true), Some(h), None);
            }
            0x3E => {
                // LD A,d8
                self.reg_a = self.read_pc();
            }
            0x3F => {
                // CCF
                self.set_flags(None, Some(false), Some(false), Some(!self.flag_c()));
            }
            0x40..=0x47 => {
                // LD B,B..LD B,A
                self.reg_b = self.read_reg(opcode & 0x07);
            }
            0x48..=0x4F => {
                // LD C,B..LD C,A
                self.reg_c = self.read_reg(opcode & 0x07);
            }
            0x50..=0x57 => {
                // LD D,B..LD D,A
                self.reg_d = self.read_reg(opcode & 0x07);
            }
            0x58..=0x5F => {
                // LD E,B..LD E,A
                self.reg_e = self.read_reg(opcode & 0x07);
            }
            0x60..=0x67 => {
                // LD H,B..LD H,A
                self.reg_h = self.read_reg(opcode & 0x07);
            }
            0x68..=0x6F => {
                // LD L,B..LD L,A
                self.reg_l = self.read_reg(opcode & 0x07);
            }
            0x76 => {
                // HALT
                self.set_halt(true);
            }
            0x70..=0x77 => {
                // LD (HL),B..LD (HL),A
                let value = self.read_reg(opcode & 0x07);
                self.bus_write(self.hl(), value);
            }
            0x78..=0x7F => {
                // LD A,B..LD A,A
                self.reg_a = self.read_reg(opcode & 0x07);
            }
            0x80..=0x87 => {
                // ADD A,B..ADD A,A
                let (value, z, h, c) =
                    alu::add::alu_add_8(self.reg_a, self.read_reg(opcode & 0x07));
                self.reg_a = value;
                self.set_flags(Some(z), Some(false), Some(h), Some(c));
            }
            0x88..=0x8F => {
                // ADC A,B..ADC A,A
                let (value, z, h, c) =
                    alu::adc::alu_adc(self.reg_a, self.read_reg(opcode & 0x07), self.flag_c());
                self.reg_a = value;
                self.set_flags(Some(z), Some(false), Some(h), Some(c));
            }
            0x90..=0x97 => {
                // SUB A,B..SUB A,A
                let (value, z, h, c) = alu::sub::alu_sub(self.reg_a, self.read_reg(opcode & 0x07));
                self.reg_a = value;
                self.set_flags(Some(z), Some(true), Some(h), Some(c));
            }
            0x98..=0x9F => {
                // SBC A,B..SBC A,A
                let (value, z, h, c) =
                    alu::sbc::alu_sbc(self.reg_a, self.read_reg(opcode & 0x07), self.flag_c());
                self.reg_a = value;
                self.set_flags(Some(z), Some(true), Some(h), Some(c));
            }
            0xA0..=0xA7 => {
                // AND A,B..AND A,A
                let (value, z) = alu::and::alu_and(self.reg_a, self.read_reg(opcode & 0x07));
                self.reg_a = value;
                self.set_flags(Some(z), Some(false), Some(true), Some(false));
            }
            0xA8..=0xAF => {
                // XOR A,B..XOR A,A
                let (value, z) = alu::xor::alu_xor(self.reg_a, self.read_reg(opcode & 0x07));
                self.reg_a = value;
                self.set_flags(Some(z), Some(false), Some(false), Some(false));
            }
            0xB0..=0xB7 => {
                // OR A,B..OR A,A
                let (value, z) = alu::or::alu_or(self.reg_a, self.read_reg(opcode & 0x07));
                self.reg_a = value;
                self.set_flags(Some(z), Some(false), Some(false), Some(false));
            }
            0xB8..=0xBF => {
                // CP A,B..CP A,A
                let (z, h, c) = alu::cp::alu_cp(self.reg_a, self.read_reg(opcode & 0x07));
                self.set_flags(Some(z), Some(true), Some(h), Some(c));
            }
            0xC0 => {
                // RET NZ
                self.adv_cycles(4);
                if !self.flag_z() {
                    self.pc = self.stack_pop2();
                    self.adv_cycles(4);
                }
            }
            0xC1 => {
                // POP BC
                let value = self.stack_pop2();
                self.set_bc(value);
            }
            0xC2 => {
                // JP NZ,a16
                let addr = self.read_pc2();
                if !self.flag_z() {
                    self.jp(addr);
                    self.adv_cycles(4);
                }
            }
            0xC3 => {
                // JP a16
                let addr = self.read_pc2();
                self.jp(addr);
                self.adv_cycles(4);
            }
            0xC4 => {
                // CALL NZ,a16
                let addr = self.read_pc2();
                if !self.flag_z() {
                    self.stack_push_pc();
                    self.jp(addr);
                    self.adv_cycles(4);
                }
            }
            0xC5 => {
                // PUSH BC
                self.stack_push2(self.bc());
                self.adv_cycles(4);
            }
            0xC6 => {
                // ADD A,d8
                let (value, z, h, c) = alu::add::alu_add_8(self.reg_a, self.read_pc());
                self.reg_a = value;
                self.set_flags(Some(z), Some(false), Some(h), Some(c));
            }
            0xC7 => {
                // RST 00H
                self.stack_push_pc();
                self.jp(0x00);
                self.adv_cycles(4);
            }
            0xC8 => {
                // RET Z
                self.adv_cycles(4);
                if self.flag_z() {
                    let addr = self.stack_pop2();
                    self.jp(addr);
                    self.adv_cycles(4);
                }
            }
            0xC9 => {
                // RET
                let addr = self.stack_pop2();
                self.jp(addr);
                self.adv_cycles(4);
            }
            0xCA => {
                // JP Z,a16
                let addr = self.read_pc2();
                if self.flag_z() {
                    self.jp(addr);
                    self.adv_cycles(4);
                }
            }
            0xCB => {
                // CB
                proc::proc_cb(self);
            }
            0xCC => {
                // CALL Z,a16
                let addr = self.read_pc2();
                if self.flag_z() {
                    self.stack_push_pc();
                    self.jp(addr);
                    self.adv_cycles(4);
                }
            }
            0xCD => {
                // CALL a16
                let addr = self.read_pc2();
                self.stack_push_pc();
                self.jp(addr);
                self.adv_cycles(4);
            }
            0xCE => {
                // ADC A,d8
                let (value, z, h, c) = alu::adc::alu_adc(self.reg_a, self.read_pc(), self.flag_c());
                self.reg_a = value;
                self.set_flags(Some(z), Some(false), Some(h), Some(c));
            }
            0xCF => {
                // RST 08H
                self.stack_push_pc();
                self.jp(0x08);
                self.adv_cycles(4);
            }
            0xD0 => {
                // RET NC
                self.adv_cycles(4);
                if !self.flag_c() {
                    let addr = self.stack_pop2();
                    self.jp(addr);
                    self.adv_cycles(4);
                }
            }
            0xD1 => {
                // POP DE
                let value = self.stack_pop2();
                self.set_de(value);
            }
            0xD2 => {
                // JP NC,a16
                let addr = self.read_pc2();
                if !self.flag_c() {
                    self.jp(addr);
                    self.adv_cycles(4);
                }
            }
            0xD4 => {
                // CALL NC,a16
                let addr = self.read_pc2();
                if !self.flag_c() {
                    self.stack_push_pc();
                    self.jp(addr);
                    self.adv_cycles(4);
                }
            }
            0xD5 => {
                // PUSH DE
                self.stack_push2(self.de());
                self.adv_cycles(4);
            }
            0xD6 => {
                // SUB A,d8
                let (value, z, h, c) = alu::sub::alu_sub(self.reg_a, self.read_pc());
                self.reg_a = value;
                self.set_flags(Some(z), Some(true), Some(h), Some(c));
            }
            0xD7 => {
                // RST 10H
                self.stack_push_pc();
                self.jp(0x10);
                self.adv_cycles(4);
            }
            0xD8 => {
                // RET C
                self.adv_cycles(4);
                if self.flag_c() {
                    let addr = self.stack_pop2();
                    self.jp(addr);
                    self.adv_cycles(4);
                }
            }
            0xD9 => {
                // RETI
                self.set_ime(true);
                let addr = self.stack_pop2();
                self.jp(addr);
                self.adv_cycles(4);
            }
            0xDA => {
                // JP C,a16
                let addr = self.read_pc2();
                if self.flag_c() {
                    self.jp(addr);
                    self.adv_cycles(4);
                }
            }
            0xDC => {
                // CALL C,a16
                let addr = self.read_pc2();
                if self.flag_c() {
                    self.stack_push_pc();
                    self.jp(addr);
                    self.adv_cycles(4);
                }
            }
            0xDE => {
                // SBC A,d8
                let (value, z, h, c) = alu::sbc::alu_sbc(self.reg_a, self.read_pc(), self.flag_c());
                self.reg_a = value;
                self.set_flags(Some(z), Some(true), Some(h), Some(c));
            }
            0xDF => {
                // RST 18H
                self.stack_push_pc();
                self.jp(0x18);
                self.adv_cycles(4);
            }
            0xE0 => {
                // LDH (a8),A
                let addr = 0xFF00 | (self.read_pc() as u16);
                self.bus_write(addr, self.reg_a);
            }
            0xE1 => {
                // POP HL
                let value = self.stack_pop2();
                self.set_hl(value);
            }
            0xE2 => {
                // LD (C),A
                self.bus_write(0xFF00 | (self.reg_c as u16), self.reg_a);
            }
            0xE5 => {
                // PUSH HL
                self.stack_push2(self.hl());
                self.adv_cycles(4);
            }
            0xE6 => {
                // AND A,d8
                let (value, z) = alu::and::alu_and(self.reg_a, self.read_pc());
                self.reg_a = value;
                self.set_flags(Some(z), Some(false), Some(true), Some(false));
            }
            0xE7 => {
                // RST 20H
                self.stack_push_pc();
                self.jp(0x20);
                self.adv_cycles(4);
            }
            0xE8 => {
                // ADD SP,r8
                let (value, h, c) = alu::add::alu_add_r8(self.sp, self.read_pc() as i8);
                self.sp = value;
                self.set_flags(Some(false), Some(false), Some(h), Some(c));
                self.adv_cycles(8);
            }
            0xE9 => {
                // JP (HL)
                self.jp(self.hl());
            }
            0xEA => {
                // LD (a16),A
                let addr = self.read_pc2();
                self.bus_write(addr, self.reg_a);
            }
            0xEE => {
                // XOR A,d8
                let (value, z) = alu::xor::alu_xor(self.reg_a, self.read_pc());
                self.reg_a = value;
                self.set_flags(Some(z), Some(false), Some(false), Some(false));
            }
            0xEF => {
                // RST 28H
                self.stack_push_pc();
                self.jp(0x28);
                self.adv_cycles(4);
            }
            0xF0 => {
                // LDH A,(a8)
                let addr = 0xFF00 | (self.read_pc() as u16);
                self.reg_a = self.bus_read(addr);
            }
            0xF1 => {
                // POP AF
                let value = self.stack_pop2();
                self.set_af(value);
            }
            0xF2 => {
                // LD A,(C)
                self.reg_a = self.bus_read(0xFF00 | (self.reg_c as u16));
            }
            0xF3 => {
                // DI
                self.set_ime(false);
            }
            0xF5 => {
                // PUSH AF
                self.stack_push2(self.af());
                self.adv_cycles(4);
            }
            0xF6 => {
                // OR A,d8
                let (value, z) = alu::or::alu_or(self.reg_a, self.read_pc());
                self.reg_a = value;
                self.set_flags(Some(z), Some(false), Some(false), Some(false));
            }
            0xF7 => {
                // RST 30H
                self.stack_push_pc();
                self.jp(0x30);
                self.adv_cycles(4);
            }
            0xF8 => {
                // LD HL,SP+r8
                let (value, h, c) = alu::add::alu_add_r8(self.sp, self.read_pc() as i8);
                self.set_hl(value);
                self.set_flags(Some(false), Some(false), Some(h), Some(c));
                self.adv_cycles(4);
            }
            0xF9 => {
                // LD SP,HL
                self.sp = self.hl();
                self.adv_cycles(4);
            }
            0xFA => {
                // LD A,(a16)
                let addr = self.read_pc2();
                self.reg_a = self.bus_read(addr);
            }
            0xFB => {
                // EI
                self.enabling_ime = true;
            }
            0xFE => {
                // CP A,d8
                let (z, h, c) = alu::cp::alu_cp(self.reg_a, self.read_pc());
                self.set_flags(Some(z), Some(true), Some(h), Some(c));
            }
            0xFF => {
                // RST 38H
                self.stack_push_pc();
                self.jp(0x38);
                self.adv_cycles(4);
            }
            _ => {
                panic!("No such instruction, opcode:{:#02X}", opcode);
            }
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

    impl gb_shared::Component for Bus {
        fn step(&mut self, cycles: u8) {
            // Noop
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
            assert_eq!(cpu.reg_f, 0x30);
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

            cpu.inc_hl();
            assert_eq!(cpu.hl(), 0x1235);

            cpu.dec_hl();
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
}
