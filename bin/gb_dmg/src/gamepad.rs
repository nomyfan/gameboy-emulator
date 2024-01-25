use gb_shared::command::JoypadKey;
use gilrs::ev::filter::{Filter, Repeat};
use gilrs::{Button, EventType, GilrsBuilder};

const GAMEPAD_KEY_JOYPAD_KEY_PAIRS: [(Button, JoypadKey); 8] = [
    (Button::DPadUp, JoypadKey::Up),
    (Button::DPadDown, JoypadKey::Down),
    (Button::DPadLeft, JoypadKey::Left),
    (Button::DPadRight, JoypadKey::Right),
    (Button::South, JoypadKey::A),
    (Button::East, JoypadKey::B),
    (Button::Start, JoypadKey::Start),
    (Button::Select, JoypadKey::Select),
];

pub(crate) fn run_event_loop(on_key_change: impl Fn(JoypadKey, bool)) {
    let mut gilrs = GilrsBuilder::new().set_update_state(false).build().unwrap();

    let repeat_filter = Repeat::new();
    loop {
        while let Some(ev) = gilrs.next_event_blocking(None).filter_ev(&repeat_filter, &mut gilrs) {
            if let EventType::ButtonChanged(button, value, _) = ev.event {
                if let Some(pair) =
                    GAMEPAD_KEY_JOYPAD_KEY_PAIRS.iter().find(|pair| pair.0 == button)
                {
                    on_key_change(pair.1, value != 0.0);
                }
            }

            gilrs.update(&ev);
        }

        gilrs.inc();
    }
}
