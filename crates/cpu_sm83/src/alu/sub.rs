pub(crate) fn alu_sub(lhs: u8, rhs: u8) -> (u8, bool, bool, bool) {
    let ret = lhs.wrapping_sub(rhs);

    let z = ret == 0;
    let h = (lhs & 0xF) < (rhs & 0xF);
    let c = lhs < rhs;

    (ret, z, h, c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub() {
        let cases = [
            (1, 1, (0, true, false, false)),
            (2, 1, (1, false, false, false)),
            (1, 3, (-2i8 as u8, false, true, true)),
            (0x11, 0x21, (0xF0, false, false, true)),
        ];

        for (a, v, output) in cases.into_iter() {
            assert_eq!(alu_sub(a, v), output);
        }
    }
}
