pub(crate) fn alu_rrca(value: u8) -> (u8, bool) {
    let mlb = value & 1;

    // Move the LSB to MSB.
    let ret = (value >> 1) | (mlb << 7);
    let c = mlb == 1;

    (ret, c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rrca() {
        let cases = [
            //
            ((0b0001_0001), (0b1000_1000, true)),
            ((0b1001_0000), (0b0100_1000, false)),
        ];

        for (in_a, output) in cases.into_iter() {
            assert_eq!(alu_rrca(in_a), output);
        }
    }
}
