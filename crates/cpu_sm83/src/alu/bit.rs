use gb_shared::is_bit_set;

/// Returns true if the bit is unset, false otherwise.
#[inline]
pub(crate) fn alu_bit(value: u8, bit: u8) -> bool {
    !is_bit_set!(value, bit)
}
