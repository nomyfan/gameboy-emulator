/// How many CPU clock cycles to produce a sample.
pub(crate) fn pulse_channel_sample_period(period_value: u16) -> u32 {
    debug_assert!(period_value <= 2047);

    // CPU_FREQ / (1048576 / (2048 - period_value as u32))
    4 * (2048 - period_value as u32)
}

/// How many CPU clock cycles to produce a sample.
pub(crate) fn wave_channel_sample_period(period_value: u16) -> u32 {
    debug_assert!(period_value <= 2047);

    // CPU_FREQ / (2097152 / (2048 - period_value as u32))
    2 * (2048 - period_value as u32)
}

pub(crate) fn pulse_period_sweep_period(nrx0: u8) -> u32 {
    let pace = (nrx0 >> 4) & 0b111;

    32768 * (pace as u32)
}

pub(crate) fn pulse_channel_period_sweep(period_value: u16, nrx0: u8) -> u16 {
    let direction = (nrx0 >> 3) & 0b1;
    let step = nrx0 & 0b111;

    if direction == 1 {
        ((period_value as i16) - ((period_value / 2u16.pow(step as u32)) as i16)) as u16
    } else {
        period_value + (period_value / 2u16.pow(step as u32))
    }
}
