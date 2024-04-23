use std::sync::mpsc::Receiver;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Joypad(JoypadCommand),
}

#[derive(Debug, Clone, Copy)]
pub enum JoypadCommand {
    PressKey(JoypadKey),
    ReleaseKey(JoypadKey),
    State(u8),
}

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy)]
pub enum JoypadKey {
    Start = 0x80,
    Select = 0x40,
    B = 0x20,
    A = 0x10,
    Down = 0x08,
    Up = 0x04,
    Left = 0x02,
    Right = 0x01,
}

pub type CommandReceiver = Receiver<Command>;
