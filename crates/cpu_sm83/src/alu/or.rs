pub(crate) fn alu_or(lhs: u8, rhs: u8) -> (u8, bool) {
    let ret = lhs | rhs;
    let z = ret == 0;
    (ret, z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn or() {
        let cases = [
            //
            ((1, 0), (1, false)),
            ((0, 1), (1, false)),
            ((1, 1), (1, false)),
            ((0, 0), (0, true)),
        ];

        for ((lhs, rhs), output) in cases.into_iter() {
            assert_eq!(alu_or(lhs, rhs), output);
        }
    }
}
