pub(crate) fn alu_sra(value: u8) -> (u8, bool) {
    let mlb = value & 1;
    let ret = ((value as i8) >> 1) as u8;
    let c = mlb == 1;

    (ret, c)
}
