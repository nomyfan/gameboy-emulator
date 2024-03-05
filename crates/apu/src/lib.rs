mod blipbuf;
mod channel;
mod clock;
mod length_timer;
mod utils;

use channel::{NoiseChannel, PulseChannel, WaveChannel};
use clock::Clock;
use gb_shared::{unset_bits, Memory, CPU_FREQ};

type SoundPanning = (bool, bool);

enum Channel {
    CH1,
    CH2,
    CH3,
    CH4,
}

pub struct Apu {
    ch1: PulseChannel,
    ch2: PulseChannel,
    ch3: WaveChannel,
    ch4: NoiseChannel,
    mixer_clock: Clock,
    // TODO: measure consume speed vs. produce speed
    output_buffer: Vec<(f32, f32)>,
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
}

impl Apu {
    fn new_mixer_clock() -> Clock {
        // TODO: adjust mixer frequency, 4194304 will cause samples to be too large
        // Clock::new(4_194_304)
        Clock::new(8192)
    }

    pub fn new(sample_rate: u32) -> Self {
        let frequency = CPU_FREQ;
        Self {
            ch1: PulseChannel::from_nrxs(
                (0x80, 0xBF, 0xF3, 0xFF, 0xBF),
                frequency,
                sample_rate,
                true,
            ),
            ch2: PulseChannel::from_nrxs(
                (0x00, 0x3F, 0x00, 0xFF, 0xBF),
                frequency,
                sample_rate,
                false,
            ),
            ch3: WaveChannel::from_nrxs((0x7F, 0xFF, 0x9F, 0xFF, 0xBF), frequency, sample_rate),
            ch4: NoiseChannel::from_nrxs((0xFF, 0x00, 0x00, 0xBF), frequency, sample_rate),
            mixer_clock: Self::new_mixer_clock(),
            output_buffer: vec![],
            nr50: 0x77,
            nr51: 0xF3,
            nr52: 0xF1,
            frequency,
            sample_rate,
        }
    }

    fn turn_off(&mut self) {
        self.ch1 = PulseChannel::new(self.frequency, self.sample_rate, true);
        self.ch2 = PulseChannel::new(self.frequency, self.sample_rate, false);
        self.ch3 = WaveChannel::new(self.frequency, self.sample_rate);
        self.ch4 = NoiseChannel::new(self.frequency, self.sample_rate);
        self.mixer_clock = Self::new_mixer_clock();
        self.output_buffer.clear();
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

    fn sound_panning(&self, channel: &Channel) -> SoundPanning {
        let (left, right) = match channel {
            Channel::CH1 => (self.nr51 & 0x10 != 0, self.nr51 & 0x01 != 0),
            Channel::CH2 => (self.nr51 & 0x20 != 0, self.nr51 & 0x02 != 0),
            Channel::CH3 => (self.nr51 & 0x40 != 0, self.nr51 & 0x04 != 0),
            Channel::CH4 => (self.nr51 & 0x80 != 0, self.nr51 & 0x08 != 0),
        };

        (left, right)
    }

    pub fn step(&mut self) {
        if !self.audio_on() {
            return;
        }

        self.ch1.next();
        self.ch2.next();
        self.ch3.next();
        self.ch4.next();

        if self.mixer_clock.next() {
            let ch1_samples = self.ch1.read_samples(self.mixer_clock.div());
            let ch2_samples = self.ch2.read_samples(self.mixer_clock.div());
            let ch3_samples = self.ch3.read_samples(self.mixer_clock.div());
            let ch4_samples = self.ch4.read_samples(self.mixer_clock.div());

            debug_assert_eq!(ch1_samples.len(), ch2_samples.len());
            debug_assert_eq!(ch2_samples.len(), ch3_samples.len());
            debug_assert_eq!(ch3_samples.len(), ch4_samples.len());

            let sample_count = ch1_samples.len();

            let left_volume_coefficient =
                ((self.master_left_volume() + 1) as f32 / 8.0) * (1.0 / 16.0) * 0.25;
            let right_volume_coefficient =
                ((self.master_right_volume() + 1) as f32 / 8.0) * (1.0 / 16.0) * 0.25;

            let mut mixed_samples = vec![(0.0, 0.0); sample_count];

            let mut mix = |(left, right): (bool, bool), samples: Vec<i16>| {
                for (v, mixed) in samples.iter().zip(&mut mixed_samples) {
                    if left {
                        mixed.0 += f32::from(*v) * left_volume_coefficient;
                    }
                    if right {
                        mixed.1 += f32::from(*v) * right_volume_coefficient;
                    }
                }
            };

            mix(self.sound_panning(&Channel::CH1), ch1_samples);
            mix(self.sound_panning(&Channel::CH2), ch2_samples);
            mix(self.sound_panning(&Channel::CH3), ch3_samples);
            mix(self.sound_panning(&Channel::CH4), ch4_samples);

            log::debug!("sample count {}", sample_count);
            for (l, r) in mixed_samples.iter() {
                if l > &1.0 || r > &1.0 || l < &-1.0 || r < &-1.0 {
                    panic!("(l,r) = ({},{})", l, r);
                }
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
