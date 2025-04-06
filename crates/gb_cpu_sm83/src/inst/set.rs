use gb_shared::set_bits;

#[inline]
pub(crate) fn set(value: u8, bit: u8) -> u8 {
    set_bits!(value, bit)
}
