pub(crate) fn alu_sbc(lhs: u8, rhs: u8, flag_c: bool) -> (u8, bool, bool, bool) {
    let ret = lhs.wrapping_sub(rhs).wrapping_sub(flag_c as u8);

    let z = ret == 0;
    let h = ((lhs & 0xF) as i16) - ((rhs & 0xF) as i16) - (flag_c as i16) < 0;
    let c = (lhs as i16) - (rhs as i16) - (flag_c as i16) < 0;

    (ret, z, h, c)
}
