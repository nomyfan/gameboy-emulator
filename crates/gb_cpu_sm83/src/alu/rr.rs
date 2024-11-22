pub(crate) fn alu_rr(value: u8, flag_c: bool) -> (u8, bool) {
    let mlb = value & 1;
    let msb = flag_c as u8;

    let ret = (value >> 1) | (msb << 7);
    let c = mlb == 1;

    (ret, c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rr() {
        let cases = [
            ((0b001_0001, true), (0b1000_1000, true)),
            ((0b001_0001, false), (0b0000_1000, true)),
            ((0b0001_0000, true), (0b1000_1000, false)),
            ((0b0001_0000, false), (0b0000_1000, false)),
        ];

        for ((in_a, in_flag_c), output) in cases.into_iter() {
            assert_eq!(alu_rr(in_a, in_flag_c), output);
        }
    }
}
