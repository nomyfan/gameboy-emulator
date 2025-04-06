pub(crate) fn add_8(lhs: u8, rhs: u8) -> (u8, bool, bool, bool) {
    let ret = lhs.wrapping_add(rhs);

    let z = ret == 0;
    let h = (lhs & 0xF) + (rhs & 0xF) > 0xF;
    let c = (lhs as u16) + (rhs as u16) > 0xFF;

    (ret, z, h, c)
}

pub(crate) fn add_16(lhs: u16, rhs: u16) -> (u16, bool, bool) {
    let ret = lhs.wrapping_add(rhs);
    let h = (lhs & 0xFFF) + (rhs & 0xFFF) > 0xFFF;
    let c = (lhs as u32) + (rhs as u32) > 0xFFFF;

    (ret, h, c)
}

pub(crate) fn add_r8(lhs: u16, rhs: i8) -> (u16, bool, bool) {
    let ret = lhs.wrapping_add_signed(rhs as i16);
    let h = (lhs & 0xF) + (rhs as u8 as u16 & 0xF) > 0xF;
    let c = (lhs & 0xFF) + rhs as u8 as u16 > 0xFF;

    (ret, h, c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sp_r8() {
        let ret = add_r8(1, -1);

        assert_eq!(ret, (0, true, true));
    }

    #[test]
    fn add_r() {
        let ret = add_8(1, 2);

        assert_eq!(ret, (3, false, false, false));
    }

    #[test]
    fn add_rr() {
        let ret = add_16(1, 2);

        assert_eq!(ret, (3, false, false));
    }

    #[test]
    fn add_rr_set_flags() {
        let cases = [
            // c
            ((0xFFFFu16, 1u16), (0u16, true, true)),
            // h
            ((0xFFF, 1), (0x1000, true, false)),
            //
            ((0x00, 0), (0, false, false)),
        ];

        for (input, output) in cases.into_iter() {
            let ret = add_16(input.0, input.1);

            assert_eq!(ret, output)
        }
    }

    #[test]
    fn add_r_set_flags() {
        let cases = [
            // c
            ((0xFFu8, 1u8), (0, true, true, true)),
            // h
            ((0xF, 1), (0x10, false, true, false)),
            // z
            ((0x00, 0), (0, true, false, false)),
        ];

        for (input, output) in cases.into_iter() {
            let ret = add_8(input.0, input.1);

            assert_eq!(ret, output)
        }
    }
}
