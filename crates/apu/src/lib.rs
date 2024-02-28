mod blipbuf;
mod channel;
mod clock;
mod utils;

use channel::{NoiseChannel, PulseChannel, WaveChannel};
use gb_shared::Memory;

fn mix(buffer: &mut Vec<(f32, f32)>, max: usize, left_output: &[f32], right_output: &[f32]) {
    for (l, r) in left_output.iter().zip(right_output) {
        if buffer.len() >= max {
            break;
        }

        buffer.push((*l, *r));
    }
}

pub struct Apu {
    ch1: PulseChannel,
    ch2: PulseChannel,
    ch3: WaveChannel,
    ch4: NoiseChannel,
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
}

impl Apu {
    pub fn new(frequency: u32, sample_rate: u32) -> Self {
        let (ch1, ch2) = PulseChannel::new_chs(frequency, sample_rate);
        Self {
            ch1,
            ch2,
            ch3: WaveChannel::new(frequency, sample_rate),
            ch4: NoiseChannel::new(frequency, sample_rate),
            output_buffer: vec![],
            nr50: 0x77,
            nr51: 0xF3,
            nr52: 0xF1,
        }
    }

    fn audio_on(&self) -> bool {
        self.nr52 & 0x80 != 0
    }

    pub fn next(&mut self) {
        if !self.audio_on() {
            return;
        }

        self.ch1.next();
        self.ch2.next();
        todo!()
    }
}

impl Memory for Apu {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF10 => {
                self.ch1.nrx0 = value;
                self.ch2.nrx0 = value;
            }
            0xFF11 => self.ch1.nrx1 = value,
            0xFF12 => self.ch1.nrx2 = value,
            0xFF13 => self.ch1.nrx3 = value,
            0xFF14 => self.ch1.nrx4 = value,

            0xFF16 => self.ch2.nrx1 = value,
            0xFF17 => self.ch2.nrx2 = value,
            0xFF18 => self.ch2.nrx3 = value,
            0xFF19 => self.ch2.nrx4 = value,

            0xFF1A => self.ch3.nrx0 = value,
            0xFF1B => self.ch3.nrx1 = value,
            0xFF1C => self.ch3.nrx2 = value,
            0xFF1D => self.ch3.nrx3 = value,
            0xFF1E => self.ch3.nrx4 = value,

            0xFF20 => self.ch4.nrx1 = value,
            0xFF21 => self.ch4.nrx2 = value,
            0xFF22 => self.ch4.nrx3 = value,
            0xFF23 => self.ch4.nrx4 = value,

            0xFF24 => self.nr50 = value,
            0xFF25 => self.nr51 = value,
            0xFF26 => self.nr52 = value,
            0xFF30..=0xFF3F => self.ch3.wave_ram[(addr - 0xFF30) as usize] = value,
            _ => unreachable!(
                "Invalid APU register write at address: {:#X} with value: {:#X}",
                addr, value
            ),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF10 => self.ch1.nrx0,
            0xFF11 => self.ch1.nrx1,
            0xFF12 => self.ch1.nrx2,
            0xFF13 => self.ch1.nrx3,
            0xFF14 => self.ch1.nrx4,

            0xFF16 => self.ch2.nrx1,
            0xFF17 => self.ch2.nrx2,
            0xFF18 => self.ch2.nrx3,
            0xFF19 => self.ch2.nrx4,

            0xFF1A => self.ch3.nrx0,
            0xFF1B => self.ch3.nrx1,
            0xFF1C => self.ch3.nrx2,
            0xFF1D => self.ch3.nrx3,
            0xFF1E => self.ch3.nrx4,

            0xFF20 => self.ch4.nrx1,
            0xFF21 => self.ch4.nrx2,
            0xFF22 => self.ch4.nrx3,
            0xFF23 => self.ch4.nrx4,

            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            0xFF26 => self.nr52,
            0xFF30..=0xFF3F => self.ch3.wave_ram[(addr - 0xFF30) as usize],
            _ => unreachable!("Invalid APU register read at address: {:#X}", addr),
        }
    }
}
