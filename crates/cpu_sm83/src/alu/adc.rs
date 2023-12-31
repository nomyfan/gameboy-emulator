pub(crate) fn alu_adc(lhs: u8, rhs: u8, flag_c: bool) -> (u8, bool, bool, bool) {
    let ret = lhs.wrapping_add(rhs).wrapping_add(flag_c as u8);

    let z = ret == 0;
    let h = (lhs & 0xF) + (rhs & 0xF) + flag_c as u8 > 0xF;
    let c = (lhs as u16) + (rhs as u16) + flag_c as u16 > 0xFF;

    (ret, z, h, c)
}
