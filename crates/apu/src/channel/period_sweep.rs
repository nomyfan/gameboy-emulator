use gb_shared::is_bit_set;

use crate::frame_sequencer::FrameSequencer;

pub(crate) trait PeriodSweep: std::fmt::Debug {
    fn new(nrx0: u8, nrx3: u8, nrx4: u8) -> Self;
    fn step(&mut self) -> Option<()>;
    fn trigger(&mut self);
    fn active(&self) -> bool;
    fn set_nrx0(&mut self, nrx0: u8);
    fn set_nrx3(&mut self, nrx3: u8);
    fn set_nrx4(&mut self, nrx4: u8);
    fn period_value(&self) -> u16;
}

pub(crate) struct SomePeriodSweep {
    fs: FrameSequencer,
    /// Complete one iteration when it reaches zero.
    /// Initialized and reset with `pace`.
    steps: u8,
    /// Control period sweep clock frequency.
    pace: u8,
    /// Used to calculate next period value.
    dir_decrease: bool,
    /// Used to calculate next period value.
    shift: u8,
    shadow_period_value: u16,
    /// Writes to NR13 and NR14 will be stored here and
    /// will be used to override `shadow_period_value` on
    /// triggered. And it can be overriden by `period_value`
    /// on next period sweep update.
    nrx34: u16,
    /// Enabled if `pace` != 8 or `shift` != 0.
    /// Update on triggered.
    enabled: bool,
    /// Control channel.
    active: bool,
    /// Set as `false` when writing to NR10.
    /// Set as `true` when at least one calculation has been made.
    occurred: bool,
}

impl std::fmt::Debug for SomePeriodSweep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SomePeriodSweep")
            .field("fs", &self.fs.current_step())
            .field("steps", &self.steps)
            .field("pace", &self.pace)
            .field("dir", &if self.dir_decrease { "-" } else { "+" })
            .field("shift", &self.shift)
            .field("shadow_period_value", &format!("{:#X}", self.shadow_period_value))
            .field("nrx34", &format!("{:#X}", self.nrx34))
            .field("enabled", &self.enabled)
            .field("active", &self.active)
            .finish()
    }
}

impl SomePeriodSweep {
    fn parse_nrx0(nrx0: u8) -> (u8, bool, u8) {
        let pace = {
            let mut pace = (nrx0 >> 4) & 0b111;
            if pace == 0 {
                pace = 8;
            }
            pace
        };
        let dir_decrease = is_bit_set!(nrx0, 3);
        let shift = nrx0 & 0b111;
        (pace, dir_decrease, shift)
    }

    fn calculate_next_period_value(period_value: u16, dir_decrease: bool, shift: u8) -> u16 {
        if period_value == 0 {
            return 0;
        }

        let delta = period_value >> shift;
        if dir_decrease {
            // delta won't > shadow_period_value,
            // so if it's subtracting, it won't underflow 0.
            period_value.saturating_sub(delta)
        } else {
            period_value.wrapping_add(delta)
        }
    }
}

#[inline]
fn period_value(nrx3: u8, nrx4: u8) -> u16 {
    ((nrx4 as u16 & 0b111) << 8) | (nrx3 as u16)
}

impl PeriodSweep for SomePeriodSweep {
    fn new(nrx0: u8, nrx3: u8, nrx4: u8) -> Self {
        let (pace, dir_decrease, shift) = Self::parse_nrx0(nrx0);

        let period_value = period_value(nrx3, nrx4);
        Self {
            fs: FrameSequencer::new(),
            steps: pace,
            pace,
            dir_decrease,
            shift,
            shadow_period_value: period_value,
            nrx34: period_value,
            enabled: pace != 8 || shift != 0,
            active: true,
            occurred: false,
        }
    }

    fn step(&mut self) -> Option<()> {
        if let Some(step) = self.fs.step() {
            if self.enabled && (step == 2 || step == 6) {
                self.steps = self.steps.saturating_sub(1);
                if self.steps == 0 {
                    self.steps = self.pace;
                    if self.pace != 8 {
                        let new_period_value = Self::calculate_next_period_value(
                            self.shadow_period_value,
                            self.dir_decrease,
                            self.shift,
                        );
                        self.active = new_period_value <= 2047;
                        if self.active && self.shift != 0 {
                            self.shadow_period_value = new_period_value;
                            self.nrx34 = new_period_value;

                            // AGAIN
                            let new_period_value = Self::calculate_next_period_value(
                                new_period_value,
                                self.dir_decrease,
                                self.shift,
                            );
                            self.active = new_period_value <= 2047;
                            log::debug!("reloaded {:?}", self);
                            return Some(());
                        }

                        self.occurred = true;
                    }
                }
            }
        }

        None
    }

    fn trigger(&mut self) {
        self.shadow_period_value = self.nrx34;
        self.steps = self.pace;
        self.enabled = self.pace != 8 || self.shift != 0;
        self.active = true;

        if self.shift != 0 {
            let new_period_value = Self::calculate_next_period_value(
                self.shadow_period_value,
                self.dir_decrease,
                self.shift,
            );

            self.active = new_period_value <= 2047;
            self.occurred = true;
        }
    }

    #[inline]
    fn active(&self) -> bool {
        self.active
    }

    fn set_nrx0(&mut self, nrx0: u8) {
        let (pace, dir_decrease, shift) = Self::parse_nrx0(nrx0);
        if self.dir_decrease && !dir_decrease && self.occurred {
            self.active = false;
        }

        self.pace = pace;
        self.dir_decrease = dir_decrease;
        self.shift = shift;
        self.occurred = false;
    }

    fn set_nrx3(&mut self, nrx3: u8) {
        let lo = nrx3 as u16;
        self.nrx34 = (self.nrx34 & 0x700) | lo;
    }

    fn set_nrx4(&mut self, nrx4: u8) {
        let hi = (nrx4 as u16 & 0x7) << 8;
        self.nrx34 = (self.nrx34 & 0xFF) | hi;
    }

    fn period_value(&self) -> u16 {
        self.shadow_period_value
    }
}

pub(crate) struct NonePeriodSweep {
    nrx3: u8,
    nrx4: u8,
}

impl std::fmt::Debug for NonePeriodSweep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NonePeriodSweep").finish()
    }
}

impl PeriodSweep for NonePeriodSweep {
    fn new(_nrx0: u8, nrx3: u8, nrx4: u8) -> Self {
        Self { nrx3, nrx4 }
    }

    fn step(&mut self) -> Option<()> {
        None
    }

    fn trigger(&mut self) {}

    fn active(&self) -> bool {
        true
    }

    fn set_nrx0(&mut self, _nrx0: u8) {}

    fn set_nrx3(&mut self, nrx3: u8) {
        self.nrx3 = nrx3;
    }

    fn set_nrx4(&mut self, nrx4: u8) {
        self.nrx4 = nrx4;
    }

    fn period_value(&self) -> u16 {
        period_value(self.nrx3, self.nrx4)
    }
}
