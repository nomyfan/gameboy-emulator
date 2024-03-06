mod blipbuf;
mod channel;
mod clock;
mod length_timer;
mod utils;

use channel::{NoiseChannel, PulseChannel, WaveChannel};
use clock::Clock;
use gb_shared::{is_bit_set, unset_bits, AudioOutHandle, Memory, CPU_FREQ};

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
    frequency: u32,
    sample_rate: u32,
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
            nr50: 0x77,
            nr51: 0xF3,
            nr52: 0xF1,
            frequency,
            sample_rate,
            audio_out_handle: None,
            samples_buffer: vec![0; buffer_size],
            mixed_samples_buffer: vec![(0.0, 0.0); buffer_size],
        }
    }

    pub fn set_audio_out_handle(&mut self, audio_out_handle: Option<Box<AudioOutHandle>>) {
        self.audio_out_handle = audio_out_handle;
    }

    fn turn_off(&mut self) {
        self.ch1 = PulseChannel::new(self.frequency, self.sample_rate, true);
        self.ch2 = PulseChannel::new(self.frequency, self.sample_rate, false);
        self.ch3 = WaveChannel::new(self.frequency, self.sample_rate);
        self.ch4 = NoiseChannel::new(self.frequency, self.sample_rate);
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
        if !self.audio_on() && addr != 0xFF26 {
            return;
        }

        match addr {
            0xFF10 => self.ch1.set_nrx0(value),
            0xFF11 => self.ch1.set_nrx1(value),
            0xFF12 => self.ch1.set_nrx2(value),
            0xFF13 => self.ch1.set_nrx3(value),
            0xFF14 => self.ch1.set_nrx4(value),

            0xFF15 => {}
            0xFF16 => self.ch2.set_nrx1(value),
            0xFF17 => self.ch2.set_nrx2(value),
            0xFF18 => self.ch2.set_nrx3(value),
            0xFF19 => self.ch2.set_nrx4(value),

            0xFF1A => self.ch3.set_nrx0(value),
            0xFF1B => self.ch3.set_nrx1(value),
            0xFF1C => self.ch3.set_nrx2(value),
            0xFF1D => self.ch3.set_nrx3(value),
            0xFF1E => self.ch3.set_nrx4(value),

            0xFF1F => {}
            0xFF20 => self.ch4.set_nrx1(value),
            0xFF21 => self.ch4.set_nrx2(value),
            0xFF22 => self.ch4.set_nrx3(value),
            0xFF23 => self.ch4.set_nrx4(value),

            0xFF24 => self.nr50 = value,
            0xFF25 => self.nr51 = value,
            0xFF26 => {
                self.nr52 = unset_bits!(self.nr52, 7) | (value & 0x80);
                if !self.audio_on() {
                    self.turn_off();
                }
            }
            0xFF27..=0xFF2F => {}
            0xFF30..=0xFF3F => self.ch3.wave_ram.write(addr, value),
            _ => unreachable!(
                "Invalid APU register write at address: {:#X} with value: {:#X}",
                addr, value
            ),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF10 => self.ch1.nrx0(),
            0xFF11 => self.ch1.nrx1(),
            0xFF12 => self.ch1.nrx2(),
            0xFF13 => self.ch1.nrx3(),
            0xFF14 => self.ch1.nrx4(),

            0xFF15 => 0,
            0xFF16 => self.ch2.nrx1(),
            0xFF17 => self.ch2.nrx2(),
            0xFF18 => self.ch2.nrx3(),
            0xFF19 => self.ch2.nrx4(),

            0xFF1A => self.ch3.nrx0(),
            0xFF1B => self.ch3.nrx1(),
            0xFF1C => self.ch3.nrx2(),
            0xFF1D => self.ch3.nrx3(),
            0xFF1E => self.ch3.nrx4(),

            0xFF1F => 0,
            0xFF20 => self.ch4.nrx1(),
            0xFF21 => self.ch4.nrx2(),
            0xFF22 => self.ch4.nrx3(),
            0xFF23 => self.ch4.nrx4(),

            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            0xFF26 => {
                let ch1_active = self.ch1.active() as u8;
                let ch2_active = (self.ch2.active() as u8) << 1;
                let ch3_active = (self.ch3.active() as u8) << 2;
                let ch4_active = (self.ch4.active() as u8) << 3;

                (self.nr52 & 0x80) | ch1_active | ch2_active | ch3_active | ch4_active
            }
            0xFF27..=0xFF2F => 0,
            0xFF30..=0xFF3F => self.ch3.wave_ram.read(addr),
            _ => unreachable!("Invalid APU register read at address: {:#X}", addr),
        }
    }
}
