pub(crate) fn alu_add_8(lhs: u8, rhs: u8) -> (u8, bool, bool, bool) {
    let ret = lhs.wrapping_add(rhs);
    let h = (lhs & 0xF) + (rhs & 0xF) > 0xF;
    let c = (lhs as u16) + (rhs as u16) > 0xFF;

    (ret, ret == 0, h, c)
}

pub(crate) fn alu_add_16(lhs: u16, rhs: u16) -> (u16, bool, bool) {
    let ret = lhs.wrapping_add(rhs);
    let h = (lhs & 0xFFF) + (rhs & 0xFFF) > 0xFFF;
    let c = (lhs as u32) + (rhs as u32) > 0xFFFF;

    (ret, h, c)
}

pub(crate) fn alu_add_sp_r8(lhs: u16, rhs: i8) -> (u16, bool, bool) {
    let ret = lhs.wrapping_add_signed(rhs as i16);
    let h = (lhs & 0xF) + (rhs as u16 & 0xF) > 0xF;
    let c = (lhs & 0xFF) as i16 + rhs as u8 as i16 > 0xFF;

    (ret, h, c)
}
