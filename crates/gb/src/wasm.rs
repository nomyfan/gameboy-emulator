#![cfg(feature = "wasm")]

pub use super::{GameBoy, Manifest};
use gb_apu::AudioOutHandle;
pub use gb_cartridge::Cartridge;
use gb_ppu::{FrameOutHandle, VideoFrame};
use gb_shared::command::Command;

// TODO: bindgen command

impl GameBoy {
    pub fn set_handles(
        &mut self,
        frame_out_handle: Box<FrameOutHandle>,
        audio_out_handle: Option<Box<AudioOutHandle>>,
    ) {
        self.bus.ppu.set_frame_out_handle(Some(frame_out_handle));
        if let Some(apu) = self.bus.apu.as_mut() {
            apu.set_audio_out_handle(audio_out_handle);
        }
    }

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

    pub fn play_with_clocks(&mut self) {
        loop {
            self.cpu.step();
            let cycles = self.cycles + self.cpu.finish_cycles() as u32;
            // TODO: make EXEC_CYCLES as parameter
            self.cycles = cycles % 70224;

            if cycles >= 70224 {
                return;
            }
        }
    }
}
