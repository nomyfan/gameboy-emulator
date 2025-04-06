pub(crate) fn sla(value: u8) -> (u8, bool) {
    let msb = (value >> 7) & 1;
    let ret = value << 1;
    let c = msb == 1;

    (ret, c)
}
