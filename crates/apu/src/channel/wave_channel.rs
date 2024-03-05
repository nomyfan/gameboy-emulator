use gb_shared::{is_bit_set, Memory};

use crate::{blipbuf, clock::Clock, length_timer::LengthTimer};

enum OutputLevel {
    Mute,
    Full,
    Half,
    Quarter,
}

pub(crate) struct WaveRam {
    ram: [u8; 16],
    index: usize,
}

impl WaveRam {
    fn new() -> Self {
        Self { ram: Default::default(), index: 1 }
    }

    fn reset(&mut self) {
        self.index = 1;
    }

    fn next(&mut self) -> u8 {
        let value = self.ram[self.index / 2];
        let value = if self.index % 2 == 0 { value >> 4 } else { value & 0x0F };

        self.index = (self.index + 1) % 32;
        value
    }
}

impl Memory for WaveRam {
    #[inline]
    fn write(&mut self, addr: u16, value: u8) {
        self.ram[(addr - 0xFF30) as usize] = value;
    }

    #[inline]
    fn read(&self, addr: u16) -> u8 {
        self.ram[(addr - 0xFF30) as usize]
    }
}

pub struct WaveChannel {
    blipbuf: blipbuf::BlipBuf,
    /// DAC enable.
    /// Bit 7: On/Off.
    nrx0: u8,
    /// Length timer.
    nrx1: u8,
    /// Output level.
    /// Bit 6..=5: Output level.
    /// 00: Mute.
    /// 01: 100%.
    /// 10: 50%.
    /// 11: 25%.
    nrx2: u8,
    /// Period low.
    /// The low 8 bits of the period value.
    nrx3: u8,
    /// Period hi and control.
    /// Bit 7: Trigger.
    /// Bit 6: Length enable.
    /// Bit 2..=0: The upper 3 bits of the period value.
    nrx4: u8,
    pub(crate) wave_ram: WaveRam,
    length_timer: Option<LengthTimer>,
    channel_clock: Clock,
}

impl WaveChannel {
    pub(crate) fn from_nrxs(
        (nrx0, nrx1, nrx2, nrx3, nrx4): (u8, u8, u8, u8, u8),
        frequency: u32,
        sample_rate: u32,
    ) -> Self {
        let channel_clock = Self::new_channel_clock(nrx3, nrx4);
        Self {
            blipbuf: blipbuf::BlipBuf::new(frequency, sample_rate, 0),
            nrx0,
            nrx1,
            nrx2,
            nrx3,
            nrx4,
            wave_ram: WaveRam::new(),
            length_timer: None,
            channel_clock,
        }
    }

    pub(crate) fn new(frequency: u32, sample_rate: u32) -> Self {
        Self::from_nrxs((0, 0, 0, 0, 0), frequency, sample_rate)
    }

    fn new_channel_clock(nrx3: u8, nrx4: u8) -> Clock {
        let period_value = (((nrx4 & 0b111) as u16) << 8) | (nrx3 as u16);

        // CPU_FREQ / (2097152 / (2048 - period_value as u32))
        let div = 2 * (2048 - period_value as u32);
        Clock::new(div)
    }

    pub(crate) fn active(&self) -> bool {
        let length_timer_expired = self.length_timer.as_ref().map_or(false, |lt| lt.expired());

        !self.dac_off() && !length_timer_expired
    }

    fn output_level(&self) -> OutputLevel {
        match (self.nrx2 >> 5) & 0b11 {
            0b00 => OutputLevel::Mute,
            0b01 => OutputLevel::Full,
            0b10 => OutputLevel::Half,
            0b11 => OutputLevel::Quarter,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn dac_off(&self) -> bool {
        !is_bit_set!(self.nrx0, 7)
    }

    #[inline]
    fn triggered(&self) -> bool {
        is_bit_set!(self.nrx4, 7)
    }

    pub(crate) fn next(&mut self) {
        if self.channel_clock.next() {
            if self.active() {
                let volume = self.wave_ram.next();
                let volume = match self.output_level() {
                    OutputLevel::Mute => 0,
                    OutputLevel::Full => volume,
                    OutputLevel::Half => volume >> 1,
                    OutputLevel::Quarter => volume >> 2,
                };
                self.blipbuf.add_delta(self.channel_clock.div(), volume as i32);
            } else {
                self.blipbuf.add_delta(self.channel_clock.div(), 0);
            }
        }

        if let Some(length_timer) = self.length_timer.as_mut() {
            length_timer.next();
        }
    }

    pub(crate) fn read_samples(&mut self, buffer: &mut [i16], duration: u32) {
        self.blipbuf.end(buffer, duration)
    }
}

impl WaveChannel {
    #[inline]
    pub(crate) fn nrx0(&self) -> u8 {
        self.nrx0
    }

    #[inline]
    pub(crate) fn nrx1(&self) -> u8 {
        self.nrx1
    }

    #[inline]
    pub(crate) fn nrx2(&self) -> u8 {
        self.nrx2
    }

    #[inline]
    pub(crate) fn nrx3(&self) -> u8 {
        self.nrx3
    }

    #[inline]
    pub(crate) fn nrx4(&self) -> u8 {
        self.nrx4
    }

    #[inline]
    pub(crate) fn set_nrx0(&mut self, value: u8) {
        self.nrx0 = value;
    }

    #[inline]
    pub(crate) fn set_nrx1(&mut self, value: u8) {
        self.nrx1 = value;
    }

    #[inline]
    pub(crate) fn set_nrx2(&mut self, value: u8) {
        self.nrx2 = value;
    }

    pub(crate) fn set_nrx3(&mut self, value: u8) {
        self.nrx3 = value;
        self.channel_clock = Self::new_channel_clock(self.nrx3, self.nrx4);
    }

    pub(crate) fn set_nrx4(&mut self, value: u8) {
        self.nrx4 = value;
        self.channel_clock = Self::new_channel_clock(self.nrx3, self.nrx4);

        if self.triggered() && !self.dac_off() {
            self.length_timer =
                if is_bit_set!(self.nrx4, 6) { Some(LengthTimer::new(self.nrx1)) } else { None };

            self.wave_ram.reset();
            self.channel_clock = Self::new_channel_clock(self.nrx3, self.nrx4);
        }
    }
}
