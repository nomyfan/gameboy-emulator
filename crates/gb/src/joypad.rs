use gb_shared::{
    command::{JoypadCommand, JoypadKey},
    is_bit_set, Memory,
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
        self.select_buttons = !is_bit_set!(value, 5);
        self.select_d_pad = !is_bit_set!(value, 4);
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

        let b3210 = ((b3 as u8) << 3) | ((b2 as u8) << 2) | ((b1 as u8) << 1) | (b0 as u8);

        !(((self.select_buttons as u8) << 5) | ((self.select_d_pad as u8) << 4) | b3210)
    }
}

impl Joypad {
    pub(crate) fn new() -> Self {
        let mut v = Self::default();
        v.select_buttons = true;
        v.select_d_pad = true;

        v
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
    use gb_shared::set_bits;

    #[test]
    fn initial_value() {
        let joypad = Joypad::new();

        assert_eq!(joypad.read(0xFF00), 0xCF);
    }

    #[test]
    fn always_1_on_unused_bits() {
        let mut joypad = Joypad::new();
        joypad.write(0xFF00, 0x00);

        assert_eq!(joypad.read(0xFF00), 0xCF);
    }

    #[test]
    fn read_select_buttons() {
        let mut joypad = Joypad::new();
        joypad.write(0xFF00, set_bits!(0, 4));

        joypad.start = true;
        joypad.b = true;

        let value = joypad.read(0xFF00);
        assert_eq!(0b1101_0101, value);
    }

    #[test]
    fn read_select_d_pad() {
        let mut joypad = Joypad::new();
        joypad.write(0xFF00, set_bits!(0, 5));

        joypad.up = true;
        joypad.right = true;

        let value = joypad.read(0xFF00);
        assert_eq!(0b1110_1010, value);
    }

    #[test]
    fn neither_of_them_enabled() {
        let mut joypad = Joypad::new();
        joypad.write(0xFF00, 0b11_0000);

        joypad.a = true;
        joypad.up = true;

        let value = joypad.read(0xFF00);
        assert_eq!(0xFF, value);
    }
}
