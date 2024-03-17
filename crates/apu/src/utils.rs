#[inline]
pub(crate) const fn freq_to_period(freq: u32) -> u32 {
    debug_assert!(freq <= gb_shared::CPU_FREQ);

    gb_shared::CPU_FREQ / freq
}
