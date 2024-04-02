use gb_shared::{
    command::{JoypadCommand, JoypadKey},
    is_bit_set, set_bits, unset_bits, Interrupt, InterruptRequest, Memory,
};

/// The state is true when the value is zero.
#[derive(Debug)]
pub(crate) struct Joypad {
    /// - Bit 7: Start
    /// - Bit 6: Select
    /// - Bit 5: B
    /// - Bit 4: A
    /// - Bit 3: Down
    /// - Bit 2: Up
    /// - Bit 1: Left
    /// - Bit 0: Right
    buttons: u8,
    select_action: bool,
    select_direction: bool,
    irq: Interrupt,
}

impl Memory for Joypad {
    fn write(&mut self, _0xff00: u16, value: u8) {
        self.select_action = !is_bit_set!(value, 5);
        self.select_direction = !is_bit_set!(value, 4);
    }

    fn read(&self, _0xff00: u16) -> u8 {
        // https://gbdev.io/pandocs/Interrupt_Sources.html#int-60--joypad-interrupt:~:text=if%20both%20are%20selected%20and%2C%20for%20example%2C%20a%20bit%20is%20already%20held%20low%20by%20an%20action%20button%2C%20pressing%20the%20corresponding%20direction%20button%20would%20make%20no%20difference.

        let b3210 = (self.buttons & if self.select_direction { 0x0F } else { 0 })
            | ((self.buttons & if self.select_action { 0xF0 } else { 0 }) >> 4);

        // We use 1 to represent pressed while GameBoy use 0.
        !(((self.select_action as u8) << 5) | ((self.select_direction as u8) << 4) | b3210)
    }
}

impl Joypad {
    pub(crate) fn new() -> Self {
        Self {
            select_action: true,
            select_direction: true,
            irq: Interrupt::default(),
            buttons: 0x00,
        }
    }

    pub(crate) fn handle_command(&mut self, command: JoypadCommand) {
        let mut mutate_key_state = |key: JoypadKey, pressed: bool| {
            let bit = key as u8;
            let old_value = is_bit_set!(self.buttons, bit);

            self.buttons =
                if pressed { set_bits!(self.buttons, bit) } else { unset_bits!(self.buttons, bit) };

            old_value
        };

        match command {
            JoypadCommand::PressKey(key) => {
                if !mutate_key_state(key, true) {
                    // https://gbdev.io/pandocs/Interrupt_Sources.html#int-60--joypad-interrupt
                    self.irq.request_joypad();
                }
            }
            JoypadCommand::ReleaseKey(key) => {
                mutate_key_state(key, false);
            }
            JoypadCommand::State(state) => {
                let pressed_buttons = (state ^ self.buttons) & state;
                self.buttons = state;
                if pressed_buttons != 0 {
                    self.irq.request_joypad();
                }
            }
        }
    }

    pub fn take_irq(&mut self) -> u8 {
        self.irq.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gb_shared::set_bits;
    use gb_shared::InterruptType;

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

        joypad.buttons = 0b1010_0000;

        let value = joypad.read(0xFF00);
        assert_eq!(0b1101_0101, value);
    }

    #[test]
    fn read_select_d_pad() {
        let mut joypad = Joypad::new();
        joypad.write(0xFF00, set_bits!(0, 5));

        joypad.buttons = 0b0000_0101;

        let value = joypad.read(0xFF00);
        assert_eq!(0b1110_1010, value);
    }

    #[test]
    fn neither_of_them_enabled() {
        let mut joypad = Joypad::new();
        joypad.write(0xFF00, 0b11_0000);

        joypad.buttons = 0b0001_0100;

        let value = joypad.read(0xFF00);
        assert_eq!(0xFF, value);
    }

    #[test]
    fn req_interrupt_if_bit3210_change_from_high_to_low() {
        let mut joypad = Joypad::new();
        joypad.handle_command(JoypadCommand::PressKey(JoypadKey::B));
        assert_eq!(joypad.take_irq(), InterruptType::Joypad as u8);
    }
}
