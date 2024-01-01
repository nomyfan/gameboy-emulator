pub(crate) fn alu_dec_8(value: u8) -> (u8, bool, bool) {
    let ret = value.wrapping_sub(1);
    let z = ret == 0;
    let h = (ret & 0xF) == 0xF;

    (ret, z, h)
}

#[inline]
pub(crate) fn alu_dec_16(value: u16) -> u16 {
    value.wrapping_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dec_rr() {
        let ret = alu_dec_16(0);

        assert_eq!(ret, 0xFFFF);
    }

    #[test]
    fn dec_r() {
        let cases = [(0u8, (0xFF, false, true)), (1, (0, true, false)), (0x10, (0xF, false, true))];

        for (input, output) in cases.into_iter() {
            assert_eq!(alu_dec_8(input), output);
        }
    }
}
