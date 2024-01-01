pub(crate) fn alu_daa(value: u8, flag_n: bool, flag_h: bool, flag_c: bool) -> (u8, bool, bool) {
    let mut acc = 0;
    let mut c = false;

    if flag_h || (!flag_n && (value & 0xF) > 9) {
        acc += 0x06;
    }

    if flag_c || (!flag_n && value > 0x99) {
        acc += 0x60;
        c = true;
    }

    let ret = if flag_n { value.wrapping_sub(acc) } else { value.wrapping_add(acc) };
    let z = ret == 0;

    (ret, z, c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daa_flag_h_set_then_add_0x6() {
        let ret = alu_daa(0, false, true, false);

        assert_eq!(ret, (0x6, false, false));
    }

    #[test]
    fn daa_reg_a_over_0x9_then_add_0x6() {
        let ret = alu_daa(0xA, false, false, false);

        assert_eq!(ret, (0x10, false, false));
    }

    #[test]
    fn daa_flag_c_set_then_add_0x60() {
        let ret = alu_daa(0, false, false, true);

        assert_eq!(ret, (0x60, false, true));
    }

    #[test]
    fn daa_reg_a_over_0x90_then_add_0x60() {
        let ret = alu_daa(0xB0, false, false, false);

        assert_eq!(ret, (0x10, false, true));
    }

    #[test]
    fn daa_flag_n_set_then_subtract() {
        let ret = alu_daa(0xAA, true, true, true);

        assert_eq!(ret, (0x44, false, true));
    }

    #[test]
    fn daa_set_flag_z_false() {
        let cases = [
            ((0x66, (true, true, true)), (true)),
            (((-0x66i8) as u8, (false, true, true)), (true)),
            ((0, (false, false, false)), (false)),
            ((0, (true, false, false)), (false)),
        ];

        for ((in_a, (in_n, in_h, in_c)), out_c) in cases.into_iter() {
            let ret = alu_daa(in_a, in_n, in_h, in_c);

            assert_eq!(ret, (0, true, out_c));
        }
    }
}
