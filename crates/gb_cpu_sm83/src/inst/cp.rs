#[inline]
pub(crate) fn cp(lhs: u8, rhs: u8) -> (bool, bool, bool) {
    let z = lhs == rhs;
    let h = (lhs & 0xF) < (rhs & 0xF);
    let c = lhs < rhs;

    (z, h, c)
}

#[cfg(test)]
mod tests {

    #[test]
    fn cp() {
        let cases = [
            ((11, 21), (false, false, true)),
            ((1, 1), (true, false, false)),
            ((2, 1), (false, false, false)),
            ((1, 3), (false, true, true)),
        ];

        for (input, output) in cases.into_iter() {
            assert_eq!(super::cp(input.0, input.1), output);
        }
    }
}
