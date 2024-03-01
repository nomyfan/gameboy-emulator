/// How many CPU clock cycles to produce a sample.
pub(crate) fn wave_channel_sample_period(period_value: u16) -> u32 {
    debug_assert!(period_value <= 2047);

    // CPU_FREQ / (2097152 / (2048 - period_value as u32))
    2 * (2048 - period_value as u32)
}

/// Convet frequency to clock cycles.
#[inline]
pub(crate) const fn freq_to_clock_cycles(freq: u32) -> u32 {
    gb_shared::CPU_FREQ / freq
}
