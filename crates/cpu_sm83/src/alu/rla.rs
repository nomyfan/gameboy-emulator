pub(crate) fn alu_rla(value: u8, flag_c: bool) -> (u8, bool) {
    let msb = (value >> 7) & 1;
    let mlb = flag_c as u8;

    let ret = (value << 1) | mlb;
    let c = msb == 1;

    (ret, c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rla() {
        let cases = [
            ((0b1000_1000, true), (0b0001_0001, true)),
            ((0b1000_1000, false), (0b0001_0000, true)),
            ((0b0000_1000, true), (0b0001_0001, false)),
            ((0b0000_1000, false), (0b0001_0000, false)),
        ];

        for ((in_value, in_flag_c), output) in cases.into_iter() {
            assert_eq!(alu_rla(in_value, in_flag_c), output);
        }
    }
}
