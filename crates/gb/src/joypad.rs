use gb_shared::{
    builder::ImBuilder,
    command::{JoypadCommand, JoypadKey},
    set_bits, unset_bits, Memory,
};

/// The state is true when the value is zero.
#[derive(Debug, Default)]
pub(crate) struct Joypad {
    // Buttons
    start: bool,
    select: bool,
    b: bool,
    a: bool,
    down: bool,
    up: bool,
    left: bool,
    right: bool,
    // Only one of them can be selected one time.
    select_buttons: bool,
    select_d_pad: bool,
}

impl Memory for Joypad {
    fn write(&mut self, _0xff00: u16, value: u8) {
        self.select_buttons = (value & set_bits!(0, 5)) == 0;
        self.select_d_pad = (value & set_bits!(1, 4)) == 0;
    }

    fn read(&self, _0xff00: u16) -> u8 {
        let (b3, b2, b1, b0) = if self.select_buttons {
            (self.start, self.select, self.b, self.a)
        } else if self.select_d_pad {
            (self.down, self.up, self.left, self.right)
        } else {
            // If neither buttons nor d-pad is selected,
            // then the low nibble is considered as 0xF.
            (false, false, false, false)
        };

        0x3F.if_then(self.select_buttons, |v| unset_bits!(v, 5))
            .if_then(self.select_d_pad, |v| unset_bits!(v, 4))
            .if_then(b3, |v| unset_bits!(v, 3))
            .if_then(b2, |v| unset_bits!(v, 2))
            .if_then(b1, |v| unset_bits!(v, 1))
            .if_then(b0, |v| unset_bits!(v, 0))
    }
}

impl Joypad {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn handle_command(&mut self, command: JoypadCommand) {
        let mut mutate_key_state = |key: JoypadKey, pressed: bool| match key {
            JoypadKey::A => self.a = pressed,
            JoypadKey::B => self.b = pressed,
            JoypadKey::Start => self.start = pressed,
            JoypadKey::Select => self.select = pressed,
            JoypadKey::Up => self.up = pressed,
            JoypadKey::Down => self.down = pressed,
            JoypadKey::Left => self.left = pressed,
            JoypadKey::Right => self.right = pressed,
        };

        match command {
            JoypadCommand::PressKey(key) => {
                mutate_key_state(key, true);
            }
            JoypadCommand::ReleaseKey(key) => {
                mutate_key_state(key, false);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_select_buttons() {
        let mut joypad = Joypad::new();
        joypad.write(0xFF00, set_bits!(0, 4));

        joypad.start = true;
        joypad.b = true;

        let value = joypad.read(0xFF00);
        assert_eq!(0b0001_0101, value);
    }

    #[test]
    fn read_select_d_pad() {
        let mut joypad = Joypad::new();
        joypad.write(0xFF00, set_bits!(0, 5));

        joypad.up = true;
        joypad.right = true;

        let value = joypad.read(0xFF00);
        assert_eq!(0b0010_1010, value);
    }

    #[test]
    fn neither_of_them_enabled() {
        let mut joypad = Joypad::new();
        joypad.write(0xFF00, 0b11_0000);

        joypad.a = true;
        joypad.up = true;

        let value = joypad.read(0xFF00);
        assert_eq!(0x3F, value);
    }
}
