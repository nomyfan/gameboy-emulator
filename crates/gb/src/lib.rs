mod bus;
mod dma;
mod hram;
mod joypad;
#[cfg(feature = "native")]
pub mod native;
mod serial;
mod timer;
#[cfg(feature = "wasm")]
pub mod wasm;
mod wram;
