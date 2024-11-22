pub(crate) fn alu_sbc(lhs: u8, rhs: u8, flag_c: bool) -> (u8, bool, bool, bool) {
    let ret = lhs.wrapping_sub(rhs).wrapping_sub(flag_c as u8);

    let z = ret == 0;
    let h = ((lhs & 0xF) as i16) - ((rhs & 0xF) as i16) - (flag_c as i16) < 0;
    let c = (lhs as i16) - (rhs as i16) - (flag_c as i16) < 0;

    (ret, z, h, c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sbc_carry() {
        let cases =
            [((2, 1, true), (0, true, false, false)), ((2, 1, false), (1, false, false, false))];

        for ((a, v, cv), output) in cases.into_iter() {
            let ret = alu_sbc(a, v, cv);

            assert_eq!(ret, output);
        }
    }

    #[test]
    fn sbc_set_flags() {
        let cases = [
            // z
            ((2, 2, false), (0, true, false, false)),
            // h
            ((0xFE, 0xEF, false), (0xF, false, true, false)),
            // c
            ((0xEF, 0xFF, false), (0xF0, false, false, true)),
        ];

        for ((a, v, cv), output) in cases.into_iter() {
            let ret = alu_sbc(a, v, cv);

            assert_eq!(ret, output);
        }
    }
}
