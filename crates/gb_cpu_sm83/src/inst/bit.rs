use gb_shared::is_bit_set;

#[inline]
pub(crate) fn bit(value: u8, bit: u8) -> bool {
    is_bit_set!(value, bit)
}
