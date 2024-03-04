/// Convet frequency to clock cycles.
#[inline]
pub(crate) const fn freq_to_clock_cycles(freq: u32) -> u32 {
    gb_shared::CPU_FREQ / freq
}
