use log::debug;

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

    pub interrupt_master_enable: bool,
    pub interrupt_enable: bool,
    pub interrupt_flags: u8,

    bus: BUS,
    // TODO
}

#[inline]
fn convert_u16_to_u8_tuple(value: u16) -> (u8, u8) {
    let hi = value & 0xFF00;
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

            interrupt_enable: false,
            interrupt_flags: 0,
            interrupt_master_enable: false,
            bus,
        }
    }

    fn bus_read(&self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    fn bus_write(&mut self, addr: u16, value: u8) {
        self.bus.write(addr, value);
    }

    #[inline]
    fn bc(&self) -> u16 {
        convert_u8_tuple_to_u16(self.reg_b, self.reg_c)
    }

    fn set_bc(&mut self, value: u16) {
        let (hi, lo) = convert_u16_to_u8_tuple(value);

        self.reg_b = hi;
        self.reg_c = lo;
    }

    #[inline]
    fn de(&self) -> u16 {
        convert_u8_tuple_to_u16(self.reg_d, self.reg_e)
    }

    fn set_de(&mut self, value: u16) {
        let (hi, lo) = convert_u16_to_u8_tuple(value);

        self.reg_d = hi;
        self.reg_e = lo;
    }

    #[inline]
    fn hl(&self) -> u16 {
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

    fn inc_pc(&mut self) -> u16 {
        let pc = self.pc;
        self.pc += 1;

        pc
    }

    #[inline]
    fn inc_hl(&mut self) {
        self.set_hl(self.hl() + 1);
    }

    #[inline]
    fn dec_hl(&mut self) {
        self.set_hl(self.hl() - 1);
    }

    #[inline]
    fn read_pc(&mut self) -> u8 {
        let addr = self.inc_pc();
        self.bus_read(addr)
    }

    pub fn execute(&mut self) {
        let opcode = self.read_pc();
        debug!("opcode 0x{opcode:02X}");

        match opcode {
            0x00 => {
                // NOP
            }
            0x01 => {
                // LD BC,D16
                let lo = self.read_pc();
                let hi = self.read_pc();
                self.set_bc(convert_u8_tuple_to_u16(hi, lo));
            }
            0x02 => {
                // LD (BC),A
                self.bus_write(self.bc(), self.reg_a);
            }
            0x06 => {
                // LD B,D8
                self.reg_b = self.read_pc();
            }
            0x08 => {
                // LD (A16),SP
                let lo = self.read_pc();
                let hi = self.read_pc();
                let addr = convert_u8_tuple_to_u16(hi, lo);
                self.bus_write(addr, (self.sp & 0x00FF) as u8);
                self.bus_write(addr + 1, ((self.sp & 0xFF00) >> 8) as u8);
            }
            0x0A => {
                // LD A,(BC)
                self.reg_a = self.bus_read(self.bc());
            }
            0x0E => {
                // LD C,D8
                let addr = self.inc_pc();
                self.reg_c = self.bus_read(addr);
            }
            0x11 => {
                // LD DE,D16
                let lo = self.read_pc();
                let hi = self.read_pc();
                self.set_de(convert_u8_tuple_to_u16(hi, lo));
            }
            0x12 => {
                // LD (DE),A
                self.bus_write(self.de(), self.reg_a);
            }
            0x16 => {
                // LD D,D8
                self.reg_d = self.read_pc();
            }
            0x1A => {
                // LD A,(DE)
                self.reg_a = self.bus_read(self.de());
            }
            0x1E => {
                // LD E,D8
                self.reg_e = self.read_pc();
            }
            0x21 => {
                // LD HL,D16
                let lo = self.read_pc();
                let hi = self.read_pc();
                self.set_hl(convert_u8_tuple_to_u16(hi, lo));
            }
            0x22 => {
                // LD (HL+),A
                self.bus_write(self.hl(), self.reg_a);
                self.inc_hl();
            }
            0x26 => {
                // LD H,D8
                self.reg_h = self.read_pc();
            }
            0x2A => {
                // LD A,(HL+)
                self.reg_a = self.bus_read(self.hl());
                self.inc_hl();
            }
            0x2E => {
                // LD L,D8
                self.reg_l = self.read_pc();
            }
            0x31 => {
                // LD SP,D16
                let lo = self.read_pc();
                let hi = self.read_pc();
                self.sp = convert_u8_tuple_to_u16(hi, lo);
            }
            0x32 => {
                // LD (HL-),A
                self.bus_write(self.hl(), self.reg_a);
                self.dec_hl();
            }
            0x36 => {
                // LD (HL),D8
                let addr = self.inc_pc();
                self.bus_write(self.hl(), self.bus_read(addr));
            }
            0x3A => {
                // LD A,(HL-)
                self.reg_a = self.bus_read(self.hl());
                self.dec_hl();
            }
            0x3E => {
                // LD A,D8
                self.reg_a = self.read_pc();
            }
            0x40 => {
                // LD B,B
            }
            0x41 => {
                // LD B,C
                self.reg_b = self.reg_c;
            }
            0x42 => {
                // LD B,D
                self.reg_b = self.reg_d;
            }
            0x43 => {
                // LD B,E
                self.reg_b = self.reg_e;
            }
            0x44 => {
                // LD B,H
                self.reg_b = self.reg_h;
            }
            0x45 => {
                // LD B,L
                self.reg_b = self.reg_l;
            }
            0x46 => {
                // LD B,(HL)
                self.reg_b = self.bus_read(self.hl());
            }
            0x47 => {
                // LD B,A
                self.reg_b = self.reg_a;
            }
            0x48 => {
                // LD C,B
                self.reg_c = self.reg_b;
            }
            0x49 => {
                // LD C,C
            }
            0x4A => {
                // LD C,D
                self.reg_c = self.reg_d;
            }
            0x4B => {
                // LD C,E
                self.reg_c = self.reg_e;
            }
            0x4C => {
                // LD C,H
                self.reg_c = self.reg_h;
            }
            0x4D => {
                // LD C,L
                self.reg_c = self.reg_l;
            }
            0x4E => {
                // LD C,(HL)
                self.reg_c = self.bus_read(self.hl());
            }
            0x4F => {
                // LD C,A
                self.reg_c = self.reg_a;
            }
            0x50 => {
                // LD D,B
                self.reg_d = self.reg_b;
            }
            0x51 => {
                // LD D,C
                self.reg_d = self.reg_c;
            }
            0x52 => {
                // LD D,D
            }
            0x53 => {
                // LD D,E
                self.reg_d = self.reg_e;
            }
            0x54 => {
                // LD D,H
                self.reg_d = self.reg_h;
            }
            0x55 => {
                // LD D,L
                self.reg_d = self.reg_l;
            }
            0x56 => {
                // LD D,(HL)
                self.reg_d = self.bus_read(self.hl());
            }
            0x57 => {
                // LD D,A
                self.reg_d = self.reg_a;
            }
            0x58 => {
                // LD E,B
                self.reg_e = self.reg_b;
            }
            0x59 => {
                // LD E,C
                self.reg_e = self.reg_c;
            }
            0x5A => {
                // LD E,D
                self.reg_e = self.reg_d;
            }
            0x5B => {
                // LD E,E
            }
            0x5C => {
                // LD E,H
                self.reg_e = self.reg_h;
            }
            0x5D => {
                // LD E,L
                self.reg_e = self.reg_l;
            }
            0x5E => {
                // LD E,(HL)
                self.reg_e = self.bus_read(self.hl());
            }
            0x5F => {
                // LD E,A
                self.reg_e = self.reg_a;
            }
            0x60 => {
                // LD H,B
                self.reg_h = self.reg_b;
            }
            0x61 => {
                // LD H,C
                self.reg_h = self.reg_c;
            }
            0x62 => {
                // LD H,D
                self.reg_h = self.reg_d;
            }
            0x63 => {
                // LD H,E
                self.reg_h = self.reg_e;
            }
            0x64 => {
                // LD H,H
            }
            0x65 => {
                // LD H,L
                self.reg_h = self.reg_l;
            }
            0x66 => {
                // LD H,(HL)
                self.reg_h = self.bus_read(self.hl());
            }
            0x67 => {
                // LD H,A
                self.reg_h = self.reg_a;
            }
            0x68 => {
                // LD L,B
                self.reg_l = self.reg_b;
            }
            0x69 => {
                // LD L,C
                self.reg_l = self.reg_c;
            }
            0x6A => {
                // LD L,D
                self.reg_l = self.reg_d;
            }
            0x6B => {
                // LD L,E
                self.reg_l = self.reg_e;
            }
            0x6C => {
                // LD L,H
                self.reg_l = self.reg_h;
            }
            0x6D => {
                // LD L,L
            }
            0x6E => {
                // LD L,(HL)
                self.reg_l = self.bus_read(self.hl());
            }
            0x6F => {
                // LD L,A
                self.reg_l = self.reg_a;
            }
            0x70 => {
                // LD (HL),B
                self.bus_write(self.hl(), self.reg_b);
            }
            0x71 => {
                // LD (HL),C
                self.bus_write(self.hl(), self.reg_c);
            }
            0x72 => {
                // LD (HL),D
                self.bus_write(self.hl(), self.reg_d);
            }
            0x73 => {
                // LD (HL),E
                self.bus_write(self.hl(), self.reg_e);
            }
            0x74 => {
                // LD (HL),H
                self.bus_write(self.hl(), self.reg_h);
            }
            0x75 => {
                // LD (HL),L
                self.bus_write(self.hl(), self.reg_l);
            }
            0x77 => {
                // LD (HL),A
                self.bus_write(self.hl(), self.reg_a);
            }
            0x78 => {
                // LD A,B
                self.reg_a = self.reg_b;
            }
            0x79 => {
                // LD A,C
                self.reg_a = self.reg_c;
            }
            0x7A => {
                // LD A,D
                self.reg_a = self.reg_d;
            }
            0x7B => {
                // LD A,E
                self.reg_a = self.reg_e;
            }
            0x7C => {
                // LD A,H
                self.reg_a = self.reg_h;
            }
            0x7D => {
                // LD A,L
                self.reg_a = self.reg_l;
            }
            0x7E => {
                // LD A,(HL)
                self.reg_a = self.bus_read(self.hl());
            }
            0x7F => {
                // LD A,A
            }
            0xE2 => {
                // LD (C),A
                self.bus_write(convert_u8_tuple_to_u16(0xFF, self.reg_c), self.reg_a);
            }
            0xEA => {
                // LD (A16),A
                let lo = self.read_pc();
                let hi = self.read_pc();
                self.bus_write(convert_u8_tuple_to_u16(hi, lo), self.reg_a);
            }
            0xF2 => {
                // LD A,(C)
                self.reg_a = self.bus_read(convert_u8_tuple_to_u16(0xFF, self.reg_c));
            }
            0xF8 => {
                // LD HL,SP+R8
                let r8 = self.read_pc();
                self.set_hl(self.sp + r8 as u16);
            }
            0xF9 => {
                // LD SP,HL
                self.sp = self.hl();
            }
            0xFA => {
                // LD A,(A16)
                let lo = self.read_pc();
                let hi = self.read_pc();
                let addr = convert_u8_tuple_to_u16(hi, lo);
                self.reg_a = self.bus_read(addr);
            }
            _ => {
                todo!()
            }
        }
    }
}
