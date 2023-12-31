pub(crate) fn alu_cp(lhs: u8, rhs: u8) -> (bool, bool, bool) {
    (lhs == rhs, (lhs & 0xF) < (rhs & 0xF), lhs < rhs)
}
