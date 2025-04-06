pub(crate) fn xor(lhs: u8, rhs: u8) -> (u8, bool) {
    let ret = lhs ^ rhs;
    let z = ret == 0;

    (ret, z)
}

#[cfg(test)]
mod tests {
    #[test]
    fn xor() {
        let cases = [
            //
            ((1, 0), (1, false)),
            ((0, 1), (1, false)),
            ((1, 1), (0, true)),
            ((0, 0), (0, true)),
        ];

        for ((a, v), output) in cases.into_iter() {
            assert_eq!(super::xor(a, v), output);
        }
    }
}
