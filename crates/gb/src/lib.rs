mod bus;
mod dma;
mod hram;
mod joypad;
#[cfg(not(target_family = "wasm"))]
pub mod native;
mod serial;
mod timer;
#[cfg(target_family = "wasm")]
pub mod wasm;
mod wram;

use web_time::{Duration, Instant};

use bus::Bus;
use gb_apu::AudioOutHandle;
use gb_cartridge::Cartridge;
use gb_cpu_sm83::Cpu;
pub use gb_ppu::FrameOutHandle;
use gb_shared::{command::Command, CPU_FREQ};

pub struct Manifest {
    pub cart: Cartridge,
    pub sample_rate: Option<u32>,
}

pub struct GameBoy {
    cpu: Cpu<Bus>,
    bus: Bus,
    clocks: u32,
    ts: Instant,
}

impl GameBoy {
    const EXEC_DURATION: Duration = Duration::from_millis(1000 / 4);
    const EXEC_CYCLES: u32 = (CPU_FREQ / 4);

    pub fn new(manifest: Manifest) -> Self {
        let Manifest { cart, sample_rate } = manifest;

        let cart_header_checksum = cart.header.checksum;
        let bus = Bus::new(cart, sample_rate);

        let mut cpu = Cpu::new(bus.clone());
        if cart_header_checksum == 0 {
            // https://gbdev.io/pandocs/Power_Up_Sequence.html#dmg_c
            // Unset H and C if the cartridge header checksum is 0.
            cpu.reg_f = 0x80;
        }

        Self { cpu, bus, clocks: 0, ts: Instant::now() }
    }

    pub fn set_handles(
        &mut self,
        frame_out_handle: Option<Box<FrameOutHandle>>,
        audio_out_handle: Option<Box<AudioOutHandle>>,
    ) {
        self.bus.ppu.set_frame_out_handle(frame_out_handle);
        if let Some(apu) = self.bus.apu.as_mut() {
            apu.set_audio_out_handle(audio_out_handle);
        }
    }

    pub fn play(
        mut self,
        frame_out_handle: Box<FrameOutHandle>,
        audio_out_handle: Option<Box<AudioOutHandle>>,
        pull_command: Box<dyn Fn() -> anyhow::Result<Option<Command>>>,
    ) -> anyhow::Result<()> {
        self.set_handles(Some(frame_out_handle), audio_out_handle);

        self.ts = Instant::now();
        loop {
            self.cpu.step();

            let cycles = self.clocks + self.cpu.take_clocks() as u32;
            self.clocks = cycles % Self::EXEC_CYCLES;

            match pull_command()? {
                Some(Command::Exit) => return Ok(()),
                Some(command) => self.bus.handle_command(command),
                None => {}
            }

            if cycles >= Self::EXEC_CYCLES {
                let duration = self.ts.elapsed();
                if duration < Self::EXEC_DURATION {
                    std::thread::sleep(Self::EXEC_DURATION - duration);
                }
                self.ts = Instant::now();
            }
        }
    }
}
