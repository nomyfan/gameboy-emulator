mod bus;
mod dma;
mod hram;
mod joypad;
mod misc_ram;
mod serial;
mod timer;
mod vdma;
mod wram;

use bus::{Bus, BusSnapshot};
pub use gb_apu::buffer_size_from_sample_rate;
pub use gb_apu::AudioHandle;
pub use gb_cartridge::Cartridge;
use gb_cpu_sm83::{Cpu, CpuSnapshot};
pub use gb_ppu::FrameHandle;
use gb_shared::{command::Command, MachineModel, Snapshot};

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
        let machine_model = cart.machine_model();
        let bus = Bus::new(cart, sample_rate);

        let cpu = match machine_model {
            MachineModel::DMG => Cpu::new_dmg(bus.clone(), cart_header_checksum),
            MachineModel::CGB => Cpu::new_cgb(bus.clone()),
        };

        Self { cpu, bus, clocks: 0, cart_checksum: cart_global_checksum }
    }

    pub fn replace_frame_handle(
        &mut self,
        handle: Option<Box<FrameHandle>>,
    ) -> Option<Box<FrameHandle>> {
        let prev = self.bus.ppu.frame_handle.take();
        self.bus.ppu.frame_handle = handle;
        prev
    }

    pub fn replace_audio_handle(
        &mut self,
        handle: Option<Box<AudioHandle>>,
    ) -> Option<Box<AudioHandle>> {
        let prev = self.bus.apu.audio_handle.take();
        self.bus.apu.audio_handle = handle;
        prev
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
