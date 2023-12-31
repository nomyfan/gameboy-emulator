pub(crate) fn alu_rla(value: u8, flag_c: bool) -> (u8, bool) {
    let msb = (value >> 7) & 1;
    let mlb = flag_c as u8;

    let ret = (value << 1) | mlb;

    (ret, msb == 1)
}
