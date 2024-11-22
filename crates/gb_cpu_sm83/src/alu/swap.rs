#[inline]
pub(crate) fn alu_swap(value: u8) -> u8 {
    ((value & 0xF0) >> 4) | ((value & 0x0F) << 4)
}
