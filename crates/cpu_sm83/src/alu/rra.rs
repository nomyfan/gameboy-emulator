pub(crate) fn alu_rra(value: u8, flag_c: bool) -> (u8, bool) {
    let mlb = value & 1;
    let msb = flag_c as u8;

    let ret = (value >> 1) | (msb << 7);
    let c = mlb == 1;

    (ret, c)
}
