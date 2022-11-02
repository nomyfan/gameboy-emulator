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
    // TODO ticks
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

impl Cpu {
    pub fn bc(&self) -> u16 {
        convert_u8_tuple_to_u16(self.reg_b, self.reg_c)
    }

    pub fn set_bc(&mut self, value: u16) {
        let (hi, lo) = convert_u16_to_u8_tuple(value);

        self.reg_b = hi;
        self.reg_c = lo;
    }

    pub fn de(&self) -> u16 {
        convert_u8_tuple_to_u16(self.reg_d, self.reg_e)
    }

    pub fn set_de(&mut self, value: u16) {
        let (hi, lo) = convert_u16_to_u8_tuple(value);

        self.reg_d = hi;
        self.reg_e = lo;
    }

    pub fn hl(&self) -> u16 {
        convert_u8_tuple_to_u16(self.reg_h, self.reg_l)
    }

    pub fn set_hl(&mut self, value: u16) {
        let (hi, lo) = convert_u16_to_u8_tuple(value);

        self.reg_h = hi;
        self.reg_l = lo;
    }

    pub fn set_flags(
        &mut self,
        z: Option<bool>,
        n: Option<bool>,
        h: Option<bool>,
        c: Option<bool>,
    ) {
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
}
