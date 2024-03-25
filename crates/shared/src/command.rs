use std::sync::mpsc::Receiver;

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
