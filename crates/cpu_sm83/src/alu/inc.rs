pub(crate) fn alu_inc_8(value: u8) -> (u8, bool, bool) {
    let ret = value.wrapping_add(1);

    let z = ret == 0;
    let h = (ret & 0xF) == 0;
    (ret, z, h)
}

#[inline]
pub(crate) fn alu_inc_16(value: u16) -> u16 {
    value.wrapping_add(1)
}
