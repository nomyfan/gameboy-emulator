pub(crate) fn alu_inc_8(value: u8) -> (u8, bool, bool) {
    let ret = value.wrapping_add(1);

    (ret, ret == 0, (ret & 0xF) == 0)
}

#[inline]
pub(crate) fn alu_inc_16(value: u16) -> u16 {
    value.wrapping_add(1)
}
