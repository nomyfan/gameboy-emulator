pub struct Cpu {
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

fn bus_read(addr: u16) -> u8 {
    todo!()
}

fn bus_write(addr: u16, value: u8) {
    todo!()
}

impl Cpu {
    pub fn new() -> Self {
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
        }
    }

    fn bc(&self) -> u16 {
        convert_u8_tuple_to_u16(self.reg_b, self.reg_c)
    }

    fn set_bc(&mut self, value: u16) {
        let (hi, lo) = convert_u16_to_u8_tuple(value);

        self.reg_b = hi;
        self.reg_c = lo;
    }

    fn de(&self) -> u16 {
        convert_u8_tuple_to_u16(self.reg_d, self.reg_e)
    }

    fn set_de(&mut self, value: u16) {
        let (hi, lo) = convert_u16_to_u8_tuple(value);

        self.reg_d = hi;
        self.reg_e = lo;
    }

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

    pub fn execute(&mut self) {
        let opcode = bus_read(self.inc_pc());

        match opcode {
            0x00 => {
                // NOP
            }
            0x01 => {
                // LD BC,D16
                let lo = bus_read(self.inc_pc());
                let hi = bus_read(self.inc_pc());
                self.set_bc(convert_u8_tuple_to_u16(hi, lo));
            }
            0x02 => {
                // LD (BC),A
                bus_write(self.bc(), self.reg_a);
            }
            0x06 => {
                // LD B,D8
                self.reg_b = bus_read(self.inc_pc());
            }
            0x08 => {
                // LD (A16),SP
                let lo = bus_read(self.inc_pc());
                let hi = bus_read(self.inc_pc());
                let addr = convert_u8_tuple_to_u16(hi, lo);
                bus_write(addr, (self.sp & 0x00FF) as u8);
                bus_write(addr + 1, ((self.sp & 0xFF00) >> 8) as u8);
            }
            0x0A => {
                // LD A,(BC)
                self.reg_a = bus_read(self.bc());
            }
            0x0E => {
                // LD C,D8
                self.reg_c = bus_read(self.inc_pc());
            }
            0x11 => {
                // LD DE,D16
                let lo = bus_read(self.inc_pc());
                let hi = bus_read(self.inc_pc());
                self.set_de(convert_u8_tuple_to_u16(hi, lo));
            }
            0x12 => {
                // LD (DE),A
                bus_write(self.de(), self.reg_a);
            }
            0x16 => {
                // LD D,D8
                self.reg_d = bus_read(self.inc_pc());
            }
            0x1A => {
                // LD A,(DE)
                self.reg_a = bus_read(self.de());
            }
            0x1E => {
                // LD E,D8
                self.reg_e = bus_read(self.inc_pc());
            }
            _ => {
                todo!()
            }
        }
    }
}
