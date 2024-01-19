use gb_shared::{
    command::{JoypadCommand, JoypadKey},
    is_bit_set, InterruptRequest, Memory,
};

/// The state is true when the value is zero.
#[derive(Debug)]
pub(crate) struct Joypad<IRQ: InterruptRequest> {
    start: bool,
    select: bool,
    b: bool,
    a: bool,
    down: bool,
    up: bool,
    left: bool,
    right: bool,
    select_buttons: bool,
    select_d_pad: bool,
    irq: IRQ,
}

impl<IRQ: InterruptRequest> Memory for Joypad<IRQ> {
    fn write(&mut self, _0xff00: u16, value: u8) {
        self.select_buttons = !is_bit_set!(value, 5);
        self.select_d_pad = !is_bit_set!(value, 4);
    }

    fn read(&self, _0xff00: u16) -> u8 {
        let (mut b3, mut b2, mut b1, mut b0) = (false, false, false, false);

        // https://gbdev.io/pandocs/Interrupt_Sources.html#int-60--joypad-interrupt:~:text=if%20both%20are%20selected%20and%2C%20for%20example%2C%20a%20bit%20is%20already%20held%20low%20by%20an%20action%20button%2C%20pressing%20the%20corresponding%20direction%20button%20would%20make%20no%20difference.
        if self.select_buttons {
            b3 |= self.start;
            b2 |= self.select;
            b1 |= self.b;
            b0 |= self.a;
        }
        if self.select_d_pad {
            b3 |= self.down;
            b2 |= self.up;
            b1 |= self.left;
            b0 |= self.right;
        }

        let b3210 = ((b3 as u8) << 3) | ((b2 as u8) << 2) | ((b1 as u8) << 1) | (b0 as u8);

        !(((self.select_buttons as u8) << 5) | ((self.select_d_pad as u8) << 4) | b3210)
    }
}

impl<IRQ: InterruptRequest> Joypad<IRQ> {
    pub(crate) fn new(irq: IRQ) -> Self {
        Self {
            select_buttons: true,
            select_d_pad: true,
            irq,
            start: false,
            select: false,
            b: false,
            a: false,
            down: false,
            up: false,
            left: false,
            right: false,
        }
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
                // https://gbdev.io/pandocs/Interrupt_Sources.html#int-60--joypad-interrupt
                self.irq.request_joypad();
            }
            JoypadCommand::ReleaseKey(key) => {
                mutate_key_state(key, false);
            }
        }
    }
}

#[cfg(test)]
mockall::mock! {
    pub Irq {}

    impl InterruptRequest for Irq {
        fn request(&mut self, interrupt_type: gb_shared::InterruptType);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gb_shared::set_bits;
    use gb_shared::InterruptType;
    use mockall::predicate::*;

    fn prepare_irq() -> MockIrq {
        MockIrq::default()
    }

    #[test]
    fn initial_value() {
        let joypad = Joypad::new(prepare_irq());

        assert_eq!(joypad.read(0xFF00), 0xCF);
    }

    #[test]
    fn always_1_on_unused_bits() {
        let mut joypad = Joypad::new(prepare_irq());
        joypad.write(0xFF00, 0x00);

        assert_eq!(joypad.read(0xFF00), 0xCF);
    }

    #[test]
    fn read_select_buttons() {
        let mut joypad = Joypad::new(prepare_irq());
        joypad.write(0xFF00, set_bits!(0, 4));

        joypad.start = true;
        joypad.b = true;

        let value = joypad.read(0xFF00);
        assert_eq!(0b1101_0101, value);
    }

    #[test]
    fn read_select_d_pad() {
        let mut joypad = Joypad::new(prepare_irq());
        joypad.write(0xFF00, set_bits!(0, 5));

        joypad.up = true;
        joypad.right = true;

        let value = joypad.read(0xFF00);
        assert_eq!(0b1110_1010, value);
    }

    #[test]
    fn neither_of_them_enabled() {
        let mut joypad = Joypad::new(prepare_irq());
        joypad.write(0xFF00, 0b11_0000);

        joypad.a = true;
        joypad.up = true;

        let value = joypad.read(0xFF00);
        assert_eq!(0xFF, value);
    }

    #[test]
    fn req_interrupt_if_bit3210_change_from_high_to_low() {
        let mut irq = prepare_irq();
        irq.expect_request().with(eq(InterruptType::Joypad)).once().return_const(());

        let mut joypad = Joypad::new(irq);
        joypad.handle_command(JoypadCommand::PressKey(JoypadKey::B));
    }
}
