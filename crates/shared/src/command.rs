use std::sync::mpsc::Receiver;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Joypad(JoypadCommand),
}

#[derive(Debug, Clone, Copy)]
pub enum JoypadCommand {
    PressKey(JoypadKey),
    ReleaseKey(JoypadKey),
}

#[derive(Debug, Clone, Copy)]
pub enum JoypadKey {
    A,
    B,
    Start,
    Select,
    Up,
    Down,
    Left,
    Right,
}

pub type CommandReceiver = Receiver<Command>;
