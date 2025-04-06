pub(crate) fn and(lhs: u8, rhs: u8) -> (u8, bool) {
    let ret = lhs & rhs;

    let z = ret == 0;
    (ret, z)
}

#[cfg(test)]
mod tests {

    #[test]
    fn and() {
        let cases = [
            ((1u8, 0u8), (0u8, true)),
            ((0, 1), (0, true)),
            ((1, 1), (1, false)),
            ((0, 0), (0, true)),
        ];

        for (input, output) in cases.into_iter() {
            assert_eq!(super::and(input.0, input.1), output);
        }
    }
}
