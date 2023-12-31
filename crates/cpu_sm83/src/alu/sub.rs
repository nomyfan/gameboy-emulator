pub(crate) fn alu_sub(lhs: u8, rhs: u8) -> (u8, bool, bool, bool) {
    let ret = lhs.wrapping_sub(rhs);

    let z = ret == 0;
    let h = (lhs & 0xF) < (rhs & 0xF);
    let c = lhs < rhs;

    (ret, z, h, c)
}
