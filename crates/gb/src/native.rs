#![cfg(feature = "native")]

pub use gb_apu::AudioOutHandle;
pub use gb_cartridge::Cartridge;
pub use gb_ppu::{FrameOutHandle, VideoFrame};

pub use super::{GameBoy, Manifest};

impl GameBoy {
    pub fn try_from_path<P: AsRef<std::path::Path>>(
        path: P,
        sample_rate: Option<u32>,
    ) -> anyhow::Result<Self> {
        let cart = Cartridge::try_from_path(path)?;
        Ok(Self::new(Manifest { cart, sample_rate }))
    }
}
