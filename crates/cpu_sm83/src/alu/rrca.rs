pub(crate) fn alu_rrca(value: u8) -> (u8, bool) {
    let mlb = value & 1;

    // Move the LSB to MSB.
    let ret = (value >> 1) | (mlb << 7);

    (ret, mlb == 1)
}
