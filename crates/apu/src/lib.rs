mod blipbuf;
mod channel;
mod clock;
mod utils;

use channel::{
    Channel1, Channel1Snapshot, Channel2, Channel2Snapshot, Channel3, Channel3Snapshot, Channel4,
    Channel4Snapshot, FrameSequencer,
};
use clock::Clock;
use gb_shared::{is_bit_set, Memory, Snapshot, CPU_FREQ};

pub type AudioOutHandle = dyn FnMut(&[(f32, f32)]);

pub struct Apu {
    ch1: Channel1,
    ch2: Channel2,
    ch3: Channel3,
    ch4: Channel4,
    mixer_clock: Clock,
    /// Master volume & VIN panning.
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
    fs: FrameSequencer,
}

impl Apu {
    const MIXER_FREQ: u32 = 64;

    fn new_mixer_clock() -> Clock {
        Clock::new(gb_shared::CPU_FREQ / Self::MIXER_FREQ)
    }

    pub fn new(sample_rate: Option<u32>) -> Self {
        let frequency = CPU_FREQ;
        let buffer_size =
            sample_rate.map_or(0, |sample_rate| sample_rate.div_ceil(Self::MIXER_FREQ) as usize);
        let fs = FrameSequencer::new();
        let instance = Self {
            ch1: Channel1::new(frequency, sample_rate),
            ch2: Channel2::new(frequency, sample_rate),
            ch3: Channel3::new(frequency, sample_rate),
            ch4: Channel4::new(frequency, sample_rate),
            mixer_clock: Self::new_mixer_clock(),
            nr50: 0x00,
            nr51: 0x00,
            nr52: 0x00,
            audio_out_handle: None,
            samples_buffer: vec![0; buffer_size],
            mixed_samples_buffer: vec![(0.0, 0.0); buffer_size],
            fs,
        };

        log::debug!("APU is created: {:?}", instance);

        instance
    }

    pub fn set_audio_out_handle(&mut self, audio_out_handle: Option<Box<AudioOutHandle>>) {
        self.audio_out_handle = audio_out_handle;
    }

    fn power_off(&mut self) {
        self.ch1.power_off();
        self.ch2.power_off();
        self.ch3.power_off();
        self.ch4.power_off();
        self.fs.power_off();
        self.mixer_clock = Self::new_mixer_clock();
        self.nr50 = 0;
        self.nr51 = 0;
        self.nr52 = 0;
        log::debug!("APU is turned off {:?}", self);
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

        let frame = self.fs.step();
        self.ch1.step(frame);
        self.ch2.step(frame);
        self.ch3.step(frame);
        self.ch4.step(frame);

        if self.mixer_clock.step() && !self.samples_buffer.is_empty() {
            let left_volume_coefficient =
                ((self.master_left_volume() + 1) as f32 / 8.0) * (1.0 / 15.0) * 0.25;
            let right_volume_coefficient =
                ((self.master_right_volume() + 1) as f32 / 8.0) * (1.0 / 15.0) * 0.25;

            self.mixed_samples_buffer.fill_with(Default::default);
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

            let ch1_samples =
                self.ch1.read_samples(&mut self.samples_buffer, self.mixer_clock.div());

            mix(is_bit_set!(self.nr51, 4), is_bit_set!(self.nr51, 0), &mut self.samples_buffer);

            let ch2_samples =
                self.ch2.read_samples(&mut self.samples_buffer, self.mixer_clock.div());
            debug_assert_eq!(ch1_samples, ch2_samples);

            mix(is_bit_set!(self.nr51, 5), is_bit_set!(self.nr51, 1), &mut self.samples_buffer);

            let ch3_samples =
                self.ch3.read_samples(&mut self.samples_buffer, self.mixer_clock.div());
            debug_assert_eq!(ch2_samples, ch3_samples);

            mix(is_bit_set!(self.nr51, 6), is_bit_set!(self.nr51, 2), &mut self.samples_buffer);

            let ch4_samples =
                self.ch4.read_samples(&mut self.samples_buffer, self.mixer_clock.div());
            debug_assert_eq!(ch3_samples, ch4_samples);

            mix(is_bit_set!(self.nr51, 7), is_bit_set!(self.nr51, 3), &mut self.samples_buffer);

            if let Some(handle) = self.audio_out_handle.as_mut() {
                handle(&self.mixed_samples_buffer[..ch1_samples]);
            }
        }
    }
}

impl Memory for Apu {
    fn write(&mut self, addr: u16, value: u8) {
        // All registers except NR52 are read-only when APU is disabled.
        // @see https://gbdev.io/pandocs/Audio_Registers.html#sound-channel-4--noise:~:text=makes%20them%20read-only%20until%20turned%20back%20on
        if !self.audio_on() {
            if addr == 0xFF11 {
                // On DMG, length counter are unaffected by power and can still be written while off.
                self.ch1.set_length_counter(value & 0x3F);
                return;
            }

            if addr == 0xFF16 {
                // On DMG, length counter are unaffected by power and can still be written while off.
                self.ch2.set_length_counter(value & 0x3F);
                return;
            }

            if addr == 0xFF1B {
                self.ch3.set_length_counter(value);
                // On DMG, length counter are unaffected by power and can still be written while off.
                return;
            }

            if addr == 0xFF20 {
                // On DMG, length counter are unaffected by power and can still be written while off.
                self.ch4.set_length_counter(value & 0x3F);
                return;
            }

            if addr != 0xFF26 // NR52
            && !(0xFF30..=0xFF3F).contains(&addr)
            // Wave RAM
            {
                return;
            }
        }

        match addr {
            0xFF10..=0xFF14 => {
                log::debug!("Write(B) NR1{} value: {:#X}, {:?}", addr - 0xFF10, value, self);
                self.ch1.write(addr - 0xFF10, value);
                log::debug!("Write(A) NR1{} value: {:#X}, {:?}", addr - 0xFF10, value, self);
            }
            0xFF15 => {}
            0xFF16..=0xFF19 => {
                log::debug!("Write(B) NR2{} value: {:#X}, {:?}", addr - 0xFF15, value, self);
                self.ch2.write(addr - 0xFF15, value);
                log::debug!("Write(A) NR2{} value: {:#X}, {:?}", addr - 0xFF15, value, self);
            }

            0xFF1A..=0xFF1E => {
                log::debug!("Write(B) NR3{} value: {:#X}, {:?}", addr - 0xFF1A, value, self);
                self.ch3.write(addr - 0xFF1A, value);
                log::debug!("Write(A) NR3{} value: {:#X}, {:?}", addr - 0xFF1A, value, self);
            }
            0xFF1F => {}
            0xFF20..=0xFF23 => {
                log::debug!("Write(B) NR4{} value: {:#X}, {:?}", addr - 0xFF1F, value, self);
                self.ch4.write(addr - 0xFF1F, value);
                log::debug!("Write(A) NR4{} value: {:#X}, {:?}", addr - 0xFF1F, value, self);
            }

            0xFF24 => self.nr50 = value,
            0xFF25 => self.nr51 = value,
            0xFF26 => {
                log::debug!("Write(B) NR52 value: {:#X}, {:?}", value, self);
                let was_power_on = self.audio_on();
                self.nr52 = value & 0x80;
                if !self.audio_on() {
                    self.power_off();
                } else if !was_power_on {
                    log::debug!("APU is turned on, {:?}", self);
                }
                log::debug!("Write(A) NR52 value: {:#X}, {:?}", value, self);
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
                let ch1_active = self.ch1.active() as u8;
                let ch2_active = (self.ch2.active() as u8) << 1;
                let ch3_active = (self.ch3.active() as u8) << 2;
                let ch4_active = (self.ch4.active() as u8) << 3;

                self.nr52 | ch1_active | ch2_active | ch3_active | ch4_active
            }
            0xFF27..=0xFF2F => 0,
            0xFF30..=0xFF3F => self.ch3.wave_ram.read(addr),
            _ => unreachable!("Invalid APU register read at address: {:#X}", addr),
        };

        value | MASKS[addr as usize - 0xFF10]
    }
}

impl std::fmt::Debug for Apu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ch1_active = self.ch1.active() as u8;
        let ch2_active = (self.ch2.active() as u8) << 1;
        let ch3_active = (self.ch3.active() as u8) << 2;
        let ch4_active = (self.ch4.active() as u8) << 3;
        let nrx52 = self.nr52 | ch1_active | ch2_active | ch3_active | ch4_active;
        f.debug_struct("Apu")
            .field("CH1", &self.ch1)
            .field("CH2", &self.ch2)
            .field("CH3", &self.ch3)
            .field("CH4", &self.ch4)
            .field("NR52", &format_args!("{:#X}", nrx52))
            .finish()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ApuSnapshot {
    ch1: Channel1Snapshot,
    ch2: Channel2Snapshot,
    ch3: Channel3Snapshot,
    ch4: Channel4Snapshot,
    mixer_clock: Clock,
    nr50: u8,
    nr51: u8,
    nr52: u8,
    fs: FrameSequencer,
}

impl Snapshot for Apu {
    type Snapshot = ApuSnapshot;

    fn snapshot(&self) -> Self::Snapshot {
        ApuSnapshot {
            ch1: self.ch1.snapshot(),
            ch2: self.ch2.snapshot(),
            ch3: self.ch3.snapshot(),
            ch4: self.ch4.snapshot(),
            mixer_clock: self.mixer_clock,
            nr50: self.nr50,
            nr51: self.nr51,
            nr52: self.nr52,
            fs: self.fs.clone(),
        }
    }

    fn restore(&mut self, snapshot: Self::Snapshot) {
        self.ch1.restore(snapshot.ch1);
        self.ch2.restore(snapshot.ch2);
        self.ch3.restore(snapshot.ch3);
        self.ch4.restore(snapshot.ch4);
        self.mixer_clock = snapshot.mixer_clock;
        self.nr50 = snapshot.nr50;
        self.nr51 = snapshot.nr51;
        self.nr52 = snapshot.nr52;
        self.fs = snapshot.fs;
    }
}
