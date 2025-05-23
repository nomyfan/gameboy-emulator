pub(crate) fn rlc(value: u8) -> (u8, bool) {
    let msb = (value >> 7) & 1;

    // Move the MSB to LSB.
    let ret = (value << 1) | msb;
    let c = msb == 1;

    (ret, c)
}

#[cfg(test)]
mod tests {
    #[test]
    fn rlc() {
        let cases = [
            //
            ((0b1001_0000), (0b0010_0001, true)),
            ((0b0001_0001), (0b0010_0010, false)),
        ];

        for (in_a, output) in cases.into_iter() {
            assert_eq!(super::rlc(in_a), output);
        }
    }
}
