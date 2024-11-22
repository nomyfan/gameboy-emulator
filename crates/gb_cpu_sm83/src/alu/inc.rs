pub(crate) fn alu_inc_8(value: u8) -> (u8, bool, bool) {
    let ret = value.wrapping_add(1);

    let z = ret == 0;
    let h = (ret & 0xF) == 0;
    (ret, z, h)
}

#[inline]
pub(crate) fn alu_inc_16(value: u16) -> u16 {
    value.wrapping_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inc_rr() {
        let ret = alu_inc_16(0xFFFF);

        assert_eq!(ret, 0);
    }

    #[test]
    fn inc_r() {
        let cases = [(0xFFu8, (0, true, true)), (0xF, (0x10, false, true))];

        for (input, output) in cases.into_iter() {
            assert_eq!(alu_inc_8(input), output);
        }
    }
}
