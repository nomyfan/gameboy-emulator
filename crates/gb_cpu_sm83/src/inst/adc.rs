pub(crate) fn adc(lhs: u8, rhs: u8, flag_c: bool) -> (u8, bool, bool, bool) {
    let ret = lhs.wrapping_add(rhs).wrapping_add(flag_c as u8);

    let z = ret == 0;
    let h = (lhs & 0xF) + (rhs & 0xF) + flag_c as u8 > 0xF;
    let c = (lhs as u16) + (rhs as u16) + flag_c as u16 > 0xFF;

    (ret, z, h, c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adc_flag_c_set_then_add_carry() {
        let ret = adc(1, 1, true);

        assert_eq!(ret, (3, false, false, false));
    }

    #[test]
    fn adc_set_flag_c() {
        let ret = adc(0x11, 0xF0, false);

        assert_eq!(ret, (0x01, false, false, true));
    }

    #[test]
    fn adc_set_flag_h() {
        let ret = adc(0x1, 0xF, false);

        assert_eq!(ret, (0x10, false, true, false));
    }

    #[test]
    fn adc_set_flag_z() {
        let ret = adc(127, (-127i8) as u8, false);

        assert_eq!(ret, (0, true, true, true));
    }
}
