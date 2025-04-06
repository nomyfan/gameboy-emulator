pub(crate) fn srl(value: u8) -> (u8, bool) {
    let mlb = value & 1;
    let ret = value >> 1;
    let c = mlb == 1;

    (ret, c)
}
