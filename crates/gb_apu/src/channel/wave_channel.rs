use gb_shared::{is_bit_set, Memory, Snapshot};
use serde::{Deserialize, Serialize};

use crate::{blipbuf, clock::Clock};

use super::{Frame, WaveChannelLengthCounter as LengthCounter};

enum OutputLevel {
    Mute,
    Full,
    Half,
    Quarter,
}

pub(crate) struct WaveRam {
    ram: [u8; 16],
    index: u8,
}

impl std::fmt::Debug for WaveRam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WaveRam").field("index", &self.index).finish()
    }
}

impl WaveRam {
    fn new() -> Self {
        Self { ram: Default::default(), index: 1 }
    }

    fn reset_position(&mut self) {
        self.index = 1;
    }

    fn next_position(&mut self) -> u8 {
        let value = self.ram[self.index as usize / 2];
        let value = if self.index % 2 == 0 { value >> 4 } else { value & 0x0F };

        self.index = (self.index + 1) % 32;
        value
    }

    // fn step(&mut self) {
    //     // TODO: https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#Frequency_Sweep:~:text=if%20the%20wave%20channel%20is%20enabled%2C%20accessing
    // }
}

impl Memory for WaveRam {
    #[inline]
    fn write(&mut self, addr: u16, value: u8) {
        // TODO: https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#Frequency_Sweep:~:text=if%20the%20wave%20channel%20is%20enabled%2C%20accessing
        log::trace!("Wave RAM write: {:#X} = {:#X}", addr, value);
        self.ram[(addr - 0xFF30) as usize] = value;
    }

    #[inline]
    fn read(&self, addr: u16) -> u8 {
        // TODO: https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#Frequency_Sweep:~:text=if%20the%20wave%20channel%20is%20enabled%2C%20accessing
        let value = self.ram[(addr - 0xFF30) as usize];
        log::trace!("Wave RAM read: {:#X} = {:#X}", addr, value);

        value
    }
}

pub(crate) struct WaveChannel {
    /// DAC enable.
    /// Bit7, 1: On, 0: Off
    nrx0: u8,
    /// Length timer. Write-only.
    nrx1: u8,
    /// Output level.
    /// Bit5..=6: Output level.
    /// 00: Mute.
    /// 01: 100%.
    /// 10: 50%.
    /// 11: 25%.
    nrx2: u8,
    /// Period low.
    /// The low 8 bits of the period value.
    ///
    /// Period changes (written to NR33 or NR34) only take effect after the following time wave RAM is read.
    /// @see https://gbdev.io/pandocs/Audio_Registers.html#ff11--nr11-channel-1-length-timer--duty-cycle:~:text=only%20take%20effect%20after%20the%20following%20time%20wave%20ram%20is%20read
    nrx3: u8,
    /// Period hi and control.
    ///
    /// Period changes (written to NR33 or NR34) only take effect after the following time wave RAM is read.
    /// @see https://gbdev.io/pandocs/Audio_Registers.html#ff11--nr11-channel-1-length-timer--duty-cycle:~:text=only%20take%20effect%20after%20the%20following%20time%20wave%20ram%20is%20read
    ///
    /// Bit 7: Trigger.
    /// Bit 6: Length enable.
    /// Bit 2..=0: The upper 3 bits of the period value.
    nrx4: u8,
    blipbuf: Option<blipbuf::BlipBuf>,
    pub(crate) wave_ram: WaveRam,
    length_counter: LengthCounter,
    channel_clock: Clock,
    active: bool,
}

impl std::fmt::Debug for WaveChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WaveChannel")
            .field("wave_ram", &self.wave_ram)
            .field("length_counter", &self.length_counter)
            .field("active", &self.active)
            .finish()
    }
}

impl WaveChannel {
    fn new_channel_clock(nrx3: u8, nrx4: u8) -> Clock {
        let period_value = (((nrx4 & 0b111) as u16) << 8) | (nrx3 as u16);

        // CPU_FREQ / (2097152 / (2048 - period_value as u32))
        Clock::new(2 * (2048 - period_value as u32))
    }
}

impl WaveChannel {
    pub(crate) fn new(frequency: u32, sample_rate: Option<u32>) -> Self {
        let nrx0 = 0;
        let nrx1 = 0;
        let nrx2 = 0;
        let nrx3 = 0;
        let nrx4 = 0;

        Self {
            blipbuf: sample_rate
                .map(|sample_rate| blipbuf::BlipBuf::new(frequency, sample_rate, 0)),
            nrx0,
            nrx1,
            nrx2,
            nrx3,
            nrx4,
            wave_ram: WaveRam::new(),
            length_counter: LengthCounter::new_expired(),
            channel_clock: Self::new_channel_clock(nrx3, nrx4),
            active: false,
        }
    }

    #[inline]
    pub(crate) fn active(&self) -> bool {
        self.active
    }

    #[inline]
    fn dac_on(&self) -> bool {
        is_bit_set!(self.nrx0, 7)
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

    pub(crate) fn step(&mut self, frame: Option<Frame>) {
        if self.channel_clock.step() {
            let volume = if self.active() {
                let volume = self.wave_ram.next_position();
                let volume = match self.output_level() {
                    OutputLevel::Mute => 0,
                    OutputLevel::Full => volume,
                    OutputLevel::Half => volume >> 1,
                    OutputLevel::Quarter => volume >> 2,
                };

                volume as i32
            } else {
                0
            };
            if let Some(blipbuf) = &mut self.blipbuf {
                blipbuf.add_delta(self.channel_clock.div(), volume);
            }
        }

        if let Some(frame) = frame {
            self.length_counter.step(frame);
        }

        self.active &= self.length_counter.active();
    }

    pub(crate) fn read_samples(&mut self, buffer: &mut [i16], duration: u32) -> usize {
        self.blipbuf.as_mut().map_or(0, |blipbuf| blipbuf.end(buffer, duration))
    }

    pub(crate) fn power_off(&mut self) {
        self.write(0, 0);
        // On DMG, length counter are unaffected by power and can still be written while off.
        self.nrx1 = 0;
        self.write(2, 0);
        self.write(3, 0);
        self.write(4, 0);

        self.length_counter.frame = Default::default();
    }

    pub(crate) fn set_length_counter(&mut self, value: u8) {
        self.length_counter.set_len(value);
    }
}

impl Memory for WaveChannel {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0 => {
                self.nrx0 = value;
                self.active &= self.dac_on();
            }
            1 => {
                self.length_counter.set_len(value);
                self.nrx1 = value;
            }
            2 => {
                self.nrx2 = value;
            }
            3 => {
                self.channel_clock = Self::new_channel_clock(value, self.nrx4);
                self.nrx3 = value;
            }
            4 => {
                self.channel_clock = Self::new_channel_clock(self.nrx3, value);
                self.length_counter.set_enabled(value);

                // Trigger the channel
                if is_bit_set!(value, 7) {
                    self.length_counter.trigger();
                    self.wave_ram.reset_position();
                }

                self.active = self.length_counter.active();
                self.active &= self.dac_on();

                self.nrx4 = value;
            }
            _ => unreachable!("Invalid address for WaveChannel: {:#X}", addr),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0 => self.nrx0,
            1 => self.nrx1,
            2 => self.nrx2,
            3 => self.nrx3,
            4 => self.nrx4,
            _ => unreachable!("Invalid address for WaveChannel: {:#X}", addr),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct WaveChannelSnapshot {
    nrx0: u8,
    nrx1: u8,
    nrx2: u8,
    nrx3: u8,
    nrx4: u8,
    wave_ram: [u8; 16],
    wave_ram_index: u8,
    length_counter: LengthCounter,
    channel_clock: Clock,
    active: bool,
}

impl Snapshot for WaveChannel {
    type Snapshot = WaveChannelSnapshot;

    fn take_snapshot(&self) -> Self::Snapshot {
        WaveChannelSnapshot {
            nrx0: self.nrx0,
            nrx1: self.nrx1,
            nrx2: self.nrx2,
            nrx3: self.nrx3,
            nrx4: self.nrx4,
            wave_ram: self.wave_ram.ram,
            wave_ram_index: self.wave_ram.index,
            length_counter: self.length_counter.clone(),
            channel_clock: self.channel_clock.clone(),
            active: self.active,
        }
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        self.nrx0 = snapshot.nrx0;
        self.nrx1 = snapshot.nrx1;
        self.nrx2 = snapshot.nrx2;
        self.nrx3 = snapshot.nrx3;
        self.nrx4 = snapshot.nrx4;
        self.wave_ram.ram = snapshot.wave_ram;
        self.wave_ram.index = snapshot.wave_ram_index;
        self.length_counter = snapshot.length_counter;
        self.channel_clock = snapshot.channel_clock;
        self.active = snapshot.active;

        if let Some(blipbuf) = &mut self.blipbuf {
            blipbuf.clear();
        }
    }
}
