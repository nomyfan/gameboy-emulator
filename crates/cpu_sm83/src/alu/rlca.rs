pub(crate) fn alu_rlca(value: u8) -> (u8, bool) {
    let msb = (value >> 7) & 1;

    // Move the MSB to LSB.
    let ret = (value << 1) | msb;

    (ret, msb == 1)
}
