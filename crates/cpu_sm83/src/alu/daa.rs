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

    (ret, ret == 0, c)
}
