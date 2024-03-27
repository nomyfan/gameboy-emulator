use std::sync::mpsc::Receiver;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Exit,
    Joypad(JoypadCommand),
}

#[derive(Debug, Clone, Copy)]
pub enum JoypadCommand {
    PressKey(JoypadKey),
    ReleaseKey(JoypadKey),
}

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy)]
pub enum JoypadKey {
    Start = 7,
    Select = 6,
    B = 5,
    A = 4,
    Down = 3,
    Up = 2,
    Left = 1,
    Right = 0,
}

pub type CommandReceiver = Receiver<Command>;
