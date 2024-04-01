#![cfg(target_family = "wasm")]

pub use super::{GameBoy, Manifest};
pub use gb_cartridge::Cartridge;
use gb_ppu::VideoFrame;
use gb_shared::command::Command;

impl GameBoy {
    pub fn pull_frame(&self) -> (&VideoFrame, u32) {
        self.bus.ppu.pull_frame()
    }

    pub fn exec_command(&mut self, command: Command) {
        match command {
            Command::Exit => {
                // Exit command has no effects in this pattern
                // TODO: maybe we should delete exit command and let user manually control only
            }
            _ => self.bus.handle_command(command),
        }
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
