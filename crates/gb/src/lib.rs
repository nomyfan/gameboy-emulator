mod bus;
mod dma;
mod hram;
mod joypad;
mod serial;
mod timer;
mod wram;

use bus::{Bus, BusSnapshot};
use gb_apu::AudioOutHandle;
pub use gb_cartridge::Cartridge;
use gb_cpu_sm83::{Cpu, CpuSnapshot};
pub use gb_ppu::FrameOutHandle;
use gb_shared::{command::Command, Snapshot};

pub struct Manifest {
    pub cart: Cartridge,
    pub sample_rate: Option<u32>,
}

pub struct GameBoy {
    cpu: Cpu<Bus>,
    bus: Bus,
    clocks: u32,
    cart_checksum: u16,
}

impl GameBoy {
    pub fn new(manifest: Manifest) -> Self {
        let Manifest { cart, sample_rate } = manifest;

        let cart_header_checksum = cart.header.checksum;
        let cart_global_checksum = cart.header.global_checksum;
        let bus = Bus::new(cart, sample_rate);

        let mut cpu = Cpu::new(bus.clone());
        if cart_header_checksum == 0 {
            // https://gbdev.io/pandocs/Power_Up_Sequence.html#dmg_c
            // Unset H and C if the cartridge header checksum is 0.
            cpu.reg_f = 0x80;
        }

        Self { cpu, bus, clocks: 0, cart_checksum: cart_global_checksum }
    }

    pub fn set_handles(
        &mut self,
        frame_out_handle: Option<Box<FrameOutHandle>>,
        audio_out_handle: Option<Box<AudioOutHandle>>,
    ) {
        self.bus.ppu.set_frame_out_handle(frame_out_handle);
        self.bus.apu.set_audio_out_handle(audio_out_handle);
    }

    #[inline]
    pub fn cart_checksum(&self) -> u16 {
        self.cart_checksum
    }

    pub fn exec_command(&mut self, command: Command) {
        self.bus.handle_command(command);
    }

    pub fn continue_clocks(&mut self, clocks: u32) {
        loop {
            self.cpu.step();
            let finished_clocks = self.clocks + self.cpu.take_clocks() as u32;
            self.clocks = finished_clocks % clocks;

            if finished_clocks >= clocks {
                return;
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GameBoySnapshot {
    cart_checksum: u16,
    bus: BusSnapshot,
    cpu: CpuSnapshot,
}

impl GameBoySnapshot {
    #[inline]
    pub fn cart_checksum(&self) -> u16 {
        self.cart_checksum
    }
}

impl Snapshot for GameBoy {
    type Snapshot = GameBoySnapshot;

    fn take_snapshot(&self) -> Self::Snapshot {
        Self::Snapshot {
            bus: self.bus.take_snapshot(),
            cpu: self.cpu.take_snapshot(),
            cart_checksum: self.cart_checksum,
        }
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        self.bus.restore_snapshot(snapshot.bus);
        self.cpu.restore_snapshot(snapshot.cpu);
    }
}

impl TryFrom<&[u8]> for GameBoySnapshot {
    type Error = bincode::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        bincode::deserialize(value)
    }
}

impl TryFrom<&GameBoySnapshot> for Vec<u8> {
    type Error = bincode::Error;

    fn try_from(value: &GameBoySnapshot) -> Result<Self, Self::Error> {
        bincode::serialize(value)
    }
}

impl GameBoy {
    pub fn suspend_cartridge(&self) -> Option<Vec<u8>> {
        self.bus.cart.suspend()
    }

    pub fn resume_cartridge(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.bus.cart.resume(data)
    }
}
