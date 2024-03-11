mod blipbuf;
mod channel;
mod clock;
mod length_counter;
mod utils;

use channel::{NoiseChannel, PulseChannel, WaveChannel};
use clock::Clock;
use gb_shared::{is_bit_set, AudioOutHandle, Memory, CPU_FREQ};

pub struct Apu {
    ch1: PulseChannel,
    ch2: PulseChannel,
    ch3: WaveChannel,
    ch4: NoiseChannel,
    mixer_clock: Clock,
    /// Master volumn & VIN panning.
    /// Bit 7: VIN left.
    /// Bit 6..=4: Volume left.
    /// Bit 3: VIN right.
    /// Bit 2..=0: Volume right.
    /// Value zero for volume is very quiet instead of silent.
    nr50: u8,
    /// Sound spanning.
    /// Bit 7: CH4 left.
    /// Bit 6: CH3 left.
    /// Bit 5: CH2 left.
    /// Bit 4: CH1 left.
    /// Bit 3: CH4 right.
    /// Bit 2: CH3 right.
    /// Bit 1: CH2 right.
    /// Bit 0: CH1 right.
    nr51: u8,
    /// Audio master control.
    /// Bit 7 - All sound on/off (1:on, 0:off)
    /// Bit 3 - CH4 ON flag (Read Only)
    /// Bit 2 - CH3 ON flag (Read Only)
    /// Bit 1 - CH2 ON flag (Read Only)
    /// Bit 0 - CH1 ON flag (Read Only)
    nr52: u8,
    audio_out_handle: Option<Box<AudioOutHandle>>,
    samples_buffer: Vec<i16>,
    mixed_samples_buffer: Vec<(f32, f32)>,
}

impl Apu {
    const MIXER_FREQ: u32 = 64;

    fn new_mixer_clock() -> Clock {
        Clock::new(gb_shared::CPU_FREQ / Self::MIXER_FREQ)
    }

    pub fn new(sample_rate: u32) -> Self {
        let frequency = CPU_FREQ;
        let buffer_size = sample_rate.div_ceil(Self::MIXER_FREQ) as usize;
        Self {
            ch1: PulseChannel::new(frequency, sample_rate, true),
            ch2: PulseChannel::new(frequency, sample_rate, false),
            ch3: WaveChannel::new(frequency, sample_rate),
            ch4: NoiseChannel::new(frequency, sample_rate),
            mixer_clock: Self::new_mixer_clock(),
            nr50: 0x00,
            nr51: 0x00,
            nr52: 0x00,
            audio_out_handle: None,
            samples_buffer: vec![0; buffer_size],
            mixed_samples_buffer: vec![(0.0, 0.0); buffer_size],
        }
    }

    pub fn set_audio_out_handle(&mut self, audio_out_handle: Option<Box<AudioOutHandle>>) {
        self.audio_out_handle = audio_out_handle;
    }

    fn turn_off(&mut self) {
        self.ch1.turn_off();
        self.ch2.turn_off();
        self.ch3.turn_off();
        self.ch4.turn_off();
        self.mixer_clock = Self::new_mixer_clock();
        self.nr50 = 0;
        self.nr51 = 0;
        self.nr52 = 0;
    }

    #[inline]
    fn audio_on(&self) -> bool {
        self.nr52 & 0x80 != 0
    }

    #[inline]
    fn master_left_volume(&self) -> u8 {
        (self.nr50 >> 4) & 0b111
    }

    #[inline]
    fn master_right_volume(&self) -> u8 {
        self.nr50 & 0b111
    }

    pub fn step(&mut self) {
        if !self.audio_on() {
            return;
        }

        self.ch1.step();
        self.ch2.step();
        self.ch3.step();
        self.ch4.step();

        if self.mixer_clock.step() {
            let left_volume_coefficient =
                ((self.master_left_volume() + 1) as f32 / 8.0) * (1.0 / 15.0) * 0.25;
            let right_volume_coefficient =
                ((self.master_right_volume() + 1) as f32 / 8.0) * (1.0 / 15.0) * 0.25;

            self.mixed_samples_buffer.iter_mut().for_each(|(l, r)| {
                *l = 0.0;
                *r = 0.0;
            });
            self.samples_buffer.fill(0);

            let mut mix = |left: bool, right: bool, samples: &[i16]| {
                for (v, mixed) in samples.iter().zip(&mut self.mixed_samples_buffer) {
                    if left {
                        mixed.0 += f32::from(*v) * left_volume_coefficient;
                    }
                    if right {
                        mixed.1 += f32::from(*v) * right_volume_coefficient;
                    }
                }
            };

            self.ch1.read_samples(&mut self.samples_buffer, self.mixer_clock.div());
            mix(is_bit_set!(self.nr51, 4), is_bit_set!(self.nr51, 0), &mut self.samples_buffer);

            self.ch2.read_samples(&mut self.samples_buffer, self.mixer_clock.div());
            mix(is_bit_set!(self.nr51, 5), is_bit_set!(self.nr51, 1), &mut self.samples_buffer);

            self.ch3.read_samples(&mut self.samples_buffer, self.mixer_clock.div());
            mix(is_bit_set!(self.nr51, 6), is_bit_set!(self.nr51, 2), &mut self.samples_buffer);

            self.ch4.read_samples(&mut self.samples_buffer, self.mixer_clock.div());
            mix(is_bit_set!(self.nr51, 7), is_bit_set!(self.nr51, 3), &mut self.samples_buffer);

            if let Some(handle) = self.audio_out_handle.as_mut() {
                handle(&self.mixed_samples_buffer);
            }
        }
    }
}

impl Memory for Apu {
    fn write(&mut self, addr: u16, value: u8) {
        // All registers except NR52 are read-only when APU is disabled.
        // @see https://gbdev.io/pandocs/Audio_Registers.html#sound-channel-4--noise:~:text=makes%20them%20read-only%20until%20turned%20back%20on
        if !self.audio_on() && addr != 0xFF26 && !(0xFF30..=0xFF3F).contains(&addr) {
            return;
        }

        match addr {
            0xFF10..=0xFF14 => self.ch1.write(addr - 0xFF10, value),
            0xFF15 => {}
            0xFF16..=0xFF19 => self.ch2.write(addr - 0xFF15, value),

            0xFF1A..=0xFF1E => self.ch3.write(addr - 0xFF1A, value),
            0xFF1F => {}
            0xFF20..=0xFF23 => self.ch4.write(addr - 0xFF1F, value),

            0xFF24 => self.nr50 = value,
            0xFF25 => self.nr51 = value,
            0xFF26 => {
                self.nr52 = value;
                if !self.audio_on() {
                    self.turn_off();
                }
            }
            0xFF27..=0xFF2F => {}
            0xFF30..=0xFF3F => {
                self.ch3.wave_ram.write(addr, value);
            }
            _ => unreachable!(
                "Invalid APU register write at address: {:#X} with value: {:#X}",
                addr, value
            ),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        const MASKS: [u8; 0x30] = [
            0x80, 0x3F, 0x00, 0xFF, 0xBF, // NR1y
            0xFF, 0x3F, 0x00, 0xFF, 0xBF, // NR2y
            0x7F, 0xFF, 0x9F, 0xFF, 0xBF, // NR3y
            0xFF, 0xFF, 0x00, 0x00, 0xBF, // NR4y
            0x00, 0x00, 0x70, // NR5y
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // Unused memory
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Wave RAM
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Wave RAM
        ];

        let value = match addr {
            0xFF10..=0xFF14 => self.ch1.read(addr - 0xFF10),
            0xFF15 => 0,
            0xFF16..=0xFF19 => self.ch2.read(addr - 0xFF15),
            0xFF1A..=0xFF1E => self.ch3.read(addr - 0xFF1A),
            0xFF1F => 0,
            0xFF20..=0xFF23 => self.ch4.read(addr - 0xFF1F),

            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            0xFF26 => {
                let ch1_on = self.ch1.on() as u8;
                let ch2_on = (self.ch2.on() as u8) << 1;
                let ch3_on = (self.ch3.on() as u8) << 2;
                let ch4_on = (self.ch4.on() as u8) << 3;

                log::debug!("CH1: {}, CH2: {}, CH3: {}, CH4: {}", ch1_on, ch2_on, ch3_on, ch4_on);

                (self.nr52 & 0x80) | ch1_on | ch2_on | ch3_on | ch4_on
            }
            0xFF27..=0xFF2F => 0,
            0xFF30..=0xFF3F => self.ch3.wave_ram.read(addr),
            _ => unreachable!("Invalid APU register read at address: {:#X}", addr),
        };

        value | MASKS[addr as usize - 0xFF10]
    }
}
