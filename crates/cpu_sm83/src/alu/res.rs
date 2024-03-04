use gb_shared::{unset_bits};

#[inline]
pub(crate) fn alu_res(value: u8, bit: u8) -> u8 {
    unset_bits!(value, bit)
}
