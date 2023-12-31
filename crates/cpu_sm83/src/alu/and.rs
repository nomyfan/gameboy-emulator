pub(crate) fn alu_and(lhs: u8, rhs: u8) -> (u8, bool) {
    let ret = lhs & rhs;

    (ret, ret == 0)
}
