pub(crate) fn alu_dec_8(value: u8) -> (u8, bool, bool) {
    let ret = value.wrapping_sub(1);

    (ret, ret == 0, (ret & 0xF) == 0xF)
}

#[inline]
pub(crate) fn alu_dec_16(value: u16) -> u16 {
    value.wrapping_sub(1)
}
