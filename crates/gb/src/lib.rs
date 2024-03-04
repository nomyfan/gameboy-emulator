mod bus;
mod dma;
mod hram;
mod joypad;
mod serial;
mod timer;
mod wram;

use gb_cartridge::Cartridge;
use gb_cpu_sm83::Cpu;
use gb_cpu_sm83::CPU_PERIOD_NANOS;
use gb_shared::{
    command::{Command, CommandReceiver},
    FrameOutHandle,
};
use std::path::Path;

use crate::bus::Bus;

pub struct GameBoy {
    cpu: Cpu<Bus>,
    bus: Bus,
    command_receiver: Option<CommandReceiver>,
}

impl GameBoy {
    pub fn from_cartridge(cart: Cartridge) -> Self {
        let cart_header_checksum = cart.header.checksum;
        let bus = Bus::new(cart);

        let mut cpu = Cpu::new(bus.clone());
        if cart_header_checksum == 0 {
            // https://gbdev.io/pandocs/Power_Up_Sequence.html#dmg_c
            // Unset H and C if the cartridge header checksum is 0.
            cpu.reg_f = 0x80;
        }

        Self { cpu, bus, command_receiver: None }
    }

    pub fn try_from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let cart = Cartridge::load(path)?;
        Ok(Self::from_cartridge(cart))
    }

    pub fn play(
        mut self,
        frame_out_handle: Box<FrameOutHandle>,
        command_receiver: CommandReceiver,
    ) -> anyhow::Result<()> {
        self.bus.set_frame_out_handle(Some(frame_out_handle));
        self.command_receiver = Some(command_receiver);

        loop {
            let now = std::time::Instant::now();
            self.cpu.step();

            let spin_period = (CPU_PERIOD_NANOS * self.cpu.finish_cycles() as f64).round() as u128;
            while now.elapsed().as_nanos() < spin_period {
                std::hint::spin_loop();
            }

            // Safety: we set the command_receiver at the start of `play` function.
            if let Ok(command) = self.command_receiver.as_ref().unwrap().try_recv() {
                if let Command::Exit = command {
                    return Ok(());
                }
                self.bus.handle_command(command);
            }
        }
    }
}
