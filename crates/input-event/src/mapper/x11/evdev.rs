use crate::mapper::error::EventMappingError;

impl From<crate::InputEvent> for evdev::InputEvent {
    fn from(value: crate::InputEvent) -> Self {
        match value {
            crate::InputEvent::Keyboard(event) => match event {
                crate::KeyboardEvent::KeyPressed(key) => evdev::InputEvent::new(
                    evdev::EventType::KEY,
                    evdev::Key::from(key).code(),
                    to_evdev_value(event),
                ),
                crate::KeyboardEvent::KeyReleased(key) => evdev::InputEvent::new(
                    evdev::EventType::KEY,
                    evdev::Key::from(key).code(),
                    to_evdev_value(event),
                ),
                crate::KeyboardEvent::KeyHeld(key) => evdev::InputEvent::new(
                    evdev::EventType::KEY,
                    evdev::Key::from(key).code(),
                    to_evdev_value(event),
                ),
            },
            crate::InputEvent::Mouse(event) => match event {
                crate::MouseEvent::Motion { axis, diff } => match axis {
                    crate::PointerAxis::Horizontal => evdev::InputEvent::new(
                        evdev::EventType::RELATIVE,
                        evdev::RelativeAxisType::REL_X.0,
                        diff,
                    ),
                    crate::PointerAxis::Vertical => evdev::InputEvent::new(
                        evdev::EventType::RELATIVE,
                        evdev::RelativeAxisType::REL_Y.0,
                        diff,
                    ),
                },
                crate::MouseEvent::Button { event_type, button } => match event_type {
                    crate::KeyboardEventType::KeyPressed => evdev::InputEvent::new(
                        evdev::EventType::KEY,
                        evdev::Key::from(button).code(),
                        to_evdev_value1(event_type),
                    ),
                    crate::KeyboardEventType::KeyReleased => evdev::InputEvent::new(
                        evdev::EventType::KEY,
                        evdev::Key::from(button).code(),
                        to_evdev_value1(event_type),
                    ),
                    crate::KeyboardEventType::KeyHeld => evdev::InputEvent::new(
                        evdev::EventType::KEY,
                        evdev::Key::from(button).code(),
                        to_evdev_value1(event_type),
                    ),
                },
                crate::MouseEvent::Scroll { axis, diff } => {
                    unimplemented!("Scroll events are not supported")
                }
            },
        }
    }
}

impl TryFrom<evdev::InputEvent> for crate::InputEvent {
    type Error = EventMappingError;

    fn try_from(value: evdev::InputEvent) -> Result<Self, Self::Error> {
        match value.event_type() {
            evdev::EventType::KEY => {
                let key = evdev::Key::new(value.code()).into();
                // TODO: button events should be mapped to a MouseEvent
                Ok(crate::InputEvent::Keyboard(match value.value() {
                    0 => crate::KeyboardEvent::KeyReleased(key),
                    1 => crate::KeyboardEvent::KeyPressed(key),
                    2 => crate::KeyboardEvent::KeyHeld(key),
                    _ => {
                        eprintln!("Invalid key event type value: {value:?}");
                        return Err(EventMappingError::InvalidEventError);
                    }
                }))
            }
            evdev::EventType::RELATIVE => {
                let axis = if value.code() == evdev::RelativeAxisType::REL_X.0 {
                    crate::PointerAxis::Horizontal
                } else if value.code() == evdev::RelativeAxisType::REL_Y.0 {
                    crate::PointerAxis::Vertical
                } else {
                    eprintln!("Invalid relative event axis value: {value:?}");
                    return Err(EventMappingError::InvalidEventError);
                };
                Ok(crate::InputEvent::Mouse(crate::MouseEvent::Motion {
                    axis,
                    diff: value.value(),
                }))
            }
            _ => {
                eprintln!("Unsupported event type: {value:?}");
                Err(EventMappingError::UnsupportedEventError)
            }
        }
    }
}

// TODO: refactor this
fn to_evdev_value(event: crate::KeyboardEvent) -> i32 {
    match event {
        crate::KeyboardEvent::KeyPressed(_) => 1,
        crate::KeyboardEvent::KeyReleased(_) => 0,
        crate::KeyboardEvent::KeyHeld(_) => 2,
    }
}

fn to_evdev_value1(event: crate::KeyboardEventType) -> i32 {
    match event {
        crate::KeyboardEventType::KeyPressed => 1,
        crate::KeyboardEventType::KeyReleased => 0,
        crate::KeyboardEventType::KeyHeld => 2,
    }
}

impl From<crate::Key> for evdev::Key {
    fn from(value: crate::Key) -> Self {
        match value {
            crate::Key::KEY_ESC => evdev::Key::KEY_ESC,
            crate::Key::KEY_1 => evdev::Key::KEY_1,
            crate::Key::KEY_2 => evdev::Key::KEY_2,
            crate::Key::KEY_3 => evdev::Key::KEY_3,
            crate::Key::KEY_4 => evdev::Key::KEY_4,
            crate::Key::KEY_5 => evdev::Key::KEY_5,
            crate::Key::KEY_6 => evdev::Key::KEY_6,
            crate::Key::KEY_7 => evdev::Key::KEY_7,
            crate::Key::KEY_8 => evdev::Key::KEY_8,
            crate::Key::KEY_9 => evdev::Key::KEY_9,
            crate::Key::KEY_0 => evdev::Key::KEY_0,
            crate::Key::KEY_MINUS => evdev::Key::KEY_MINUS,
            crate::Key::KEY_EQUAL => evdev::Key::KEY_EQUAL,
            crate::Key::KEY_BACKSPACE => evdev::Key::KEY_BACKSPACE,
            crate::Key::KEY_TAB => evdev::Key::KEY_TAB,
            crate::Key::KEY_Q => evdev::Key::KEY_Q,
            crate::Key::KEY_W => evdev::Key::KEY_W,
            crate::Key::KEY_E => evdev::Key::KEY_E,
            crate::Key::KEY_R => evdev::Key::KEY_R,
            crate::Key::KEY_T => evdev::Key::KEY_T,
            crate::Key::KEY_Y => evdev::Key::KEY_Y,
            crate::Key::KEY_U => evdev::Key::KEY_U,
            crate::Key::KEY_I => evdev::Key::KEY_I,
            crate::Key::KEY_O => evdev::Key::KEY_O,
            crate::Key::KEY_P => evdev::Key::KEY_P,
            crate::Key::KEY_LEFTBRACE => evdev::Key::KEY_LEFTBRACE,
            crate::Key::KEY_RIGHTBRACE => evdev::Key::KEY_RIGHTBRACE,
            crate::Key::KEY_ENTER => evdev::Key::KEY_ENTER,
            crate::Key::KEY_LEFTCTRL => evdev::Key::KEY_LEFTCTRL,
            crate::Key::KEY_A => evdev::Key::KEY_A,
            crate::Key::KEY_S => evdev::Key::KEY_S,
            crate::Key::KEY_D => evdev::Key::KEY_D,
            crate::Key::KEY_F => evdev::Key::KEY_F,
            crate::Key::KEY_G => evdev::Key::KEY_G,
            crate::Key::KEY_H => evdev::Key::KEY_H,
            crate::Key::KEY_J => evdev::Key::KEY_J,
            crate::Key::KEY_K => evdev::Key::KEY_K,
            crate::Key::KEY_L => evdev::Key::KEY_L,
            crate::Key::KEY_SEMICOLON => evdev::Key::KEY_SEMICOLON,
            crate::Key::KEY_APOSTROPHE => evdev::Key::KEY_APOSTROPHE,
            crate::Key::KEY_GRAVE => evdev::Key::KEY_GRAVE,
            crate::Key::KEY_LEFTSHIFT => evdev::Key::KEY_LEFTSHIFT,
            crate::Key::KEY_BACKSLASH => evdev::Key::KEY_BACKSLASH,
            crate::Key::KEY_Z => evdev::Key::KEY_Z,
            crate::Key::KEY_X => evdev::Key::KEY_X,
            crate::Key::KEY_C => evdev::Key::KEY_C,
            crate::Key::KEY_V => evdev::Key::KEY_V,
            crate::Key::KEY_B => evdev::Key::KEY_B,
            crate::Key::KEY_N => evdev::Key::KEY_N,
            crate::Key::KEY_M => evdev::Key::KEY_M,
            crate::Key::KEY_COMMA => evdev::Key::KEY_COMMA,
            crate::Key::KEY_DOT => evdev::Key::KEY_DOT,
            crate::Key::KEY_SLASH => evdev::Key::KEY_SLASH,
            crate::Key::KEY_RIGHTSHIFT => evdev::Key::KEY_RIGHTSHIFT,
            crate::Key::KEY_KPASTERISK => evdev::Key::KEY_KPASTERISK,
            crate::Key::KEY_LEFTALT => evdev::Key::KEY_LEFTALT,
            crate::Key::KEY_SPACE => evdev::Key::KEY_SPACE,
            crate::Key::KEY_CAPSLOCK => evdev::Key::KEY_CAPSLOCK,
            crate::Key::KEY_F1 => evdev::Key::KEY_F1,
            crate::Key::KEY_F2 => evdev::Key::KEY_F2,
            crate::Key::KEY_F3 => evdev::Key::KEY_F3,
            crate::Key::KEY_F4 => evdev::Key::KEY_F4,
            crate::Key::KEY_F5 => evdev::Key::KEY_F5,
            crate::Key::KEY_F6 => evdev::Key::KEY_F6,
            crate::Key::KEY_F7 => evdev::Key::KEY_F7,
            crate::Key::KEY_F8 => evdev::Key::KEY_F8,
            crate::Key::KEY_F9 => evdev::Key::KEY_F9,
            crate::Key::KEY_F10 => evdev::Key::KEY_F10,
            crate::Key::KEY_NUMLOCK => evdev::Key::KEY_NUMLOCK,
            crate::Key::KEY_SCROLLLOCK => evdev::Key::KEY_SCROLLLOCK,
            crate::Key::KEY_KP7 => evdev::Key::KEY_KP7,
            crate::Key::KEY_KP8 => evdev::Key::KEY_KP8,
            crate::Key::KEY_KP9 => evdev::Key::KEY_KP9,
            crate::Key::KEY_KPMINUS => evdev::Key::KEY_KPMINUS,
            crate::Key::KEY_KP4 => evdev::Key::KEY_KP4,
            crate::Key::KEY_KP5 => evdev::Key::KEY_KP5,
            crate::Key::KEY_KP6 => evdev::Key::KEY_KP6,
            crate::Key::KEY_KPPLUS => evdev::Key::KEY_KPPLUS,
            crate::Key::KEY_KP1 => evdev::Key::KEY_KP1,
            crate::Key::KEY_KP2 => evdev::Key::KEY_KP2,
            crate::Key::KEY_KP3 => evdev::Key::KEY_KP3,
            crate::Key::KEY_KP0 => evdev::Key::KEY_KP0,
            crate::Key::KEY_KPDOT => evdev::Key::KEY_KPDOT,
            crate::Key::KEY_F11 => evdev::Key::KEY_F11,
            crate::Key::KEY_F12 => evdev::Key::KEY_F12,
            crate::Key::KEY_HOME => evdev::Key::KEY_HOME,
            crate::Key::KEY_UP => evdev::Key::KEY_UP,
            crate::Key::KEY_PAGEUP => evdev::Key::KEY_PAGEUP,
            crate::Key::KEY_LEFT => evdev::Key::KEY_LEFT,
            crate::Key::KEY_RIGHT => evdev::Key::KEY_RIGHT,
            crate::Key::KEY_END => evdev::Key::KEY_END,
            crate::Key::KEY_DOWN => evdev::Key::KEY_DOWN,
            crate::Key::KEY_PAGEDOWN => evdev::Key::KEY_PAGEDOWN,
            crate::Key::KEY_INSERT => evdev::Key::KEY_INSERT,
            crate::Key::KEY_DELETE => evdev::Key::KEY_DELETE,
            crate::Key::KEY_LEFTMETA => evdev::Key::KEY_LEFTMETA,
            crate::Key::KEY_RIGHTMETA => evdev::Key::KEY_RIGHTMETA,
            crate::Key::KEY_MENU => evdev::Key::KEY_MENU,
            crate::Key::BTN_LEFT => evdev::Key::BTN_LEFT,
            crate::Key::BTN_RIGHT => evdev::Key::BTN_RIGHT,
            crate::Key::BTN_MIDDLE => evdev::Key::BTN_MIDDLE,
            crate::Key::BTN_SIDE => evdev::Key::BTN_SIDE,
            crate::Key::BTN_EXTRA => evdev::Key::BTN_EXTRA,
            crate::Key::BTN_FORWARD => evdev::Key::BTN_FORWARD,
            crate::Key::BTN_BACK => evdev::Key::BTN_BACK,
        }
    }
}

impl From<evdev::Key> for crate::Key {
    fn from(val: evdev::Key) -> Self {
        match val {
            evdev::Key::KEY_ESC => crate::Key::KEY_ESC,
            evdev::Key::KEY_1 => crate::Key::KEY_1,
            evdev::Key::KEY_2 => crate::Key::KEY_2,
            evdev::Key::KEY_3 => crate::Key::KEY_3,
            evdev::Key::KEY_4 => crate::Key::KEY_4,
            evdev::Key::KEY_5 => crate::Key::KEY_5,
            evdev::Key::KEY_6 => crate::Key::KEY_6,
            evdev::Key::KEY_7 => crate::Key::KEY_7,
            evdev::Key::KEY_8 => crate::Key::KEY_8,
            evdev::Key::KEY_9 => crate::Key::KEY_9,
            evdev::Key::KEY_0 => crate::Key::KEY_0,
            evdev::Key::KEY_MINUS => crate::Key::KEY_MINUS,
            evdev::Key::KEY_EQUAL => crate::Key::KEY_EQUAL,
            evdev::Key::KEY_BACKSPACE => crate::Key::KEY_BACKSPACE,
            evdev::Key::KEY_TAB => crate::Key::KEY_TAB,
            evdev::Key::KEY_Q => crate::Key::KEY_Q,
            evdev::Key::KEY_W => crate::Key::KEY_W,
            evdev::Key::KEY_E => crate::Key::KEY_E,
            evdev::Key::KEY_R => crate::Key::KEY_R,
            evdev::Key::KEY_T => crate::Key::KEY_T,
            evdev::Key::KEY_Y => crate::Key::KEY_Y,
            evdev::Key::KEY_U => crate::Key::KEY_U,
            evdev::Key::KEY_I => crate::Key::KEY_I,
            evdev::Key::KEY_O => crate::Key::KEY_O,
            evdev::Key::KEY_P => crate::Key::KEY_P,
            evdev::Key::KEY_LEFTBRACE => crate::Key::KEY_LEFTBRACE,
            evdev::Key::KEY_RIGHTBRACE => crate::Key::KEY_RIGHTBRACE,
            evdev::Key::KEY_ENTER => crate::Key::KEY_ENTER,
            evdev::Key::KEY_LEFTCTRL => crate::Key::KEY_LEFTCTRL,
            evdev::Key::KEY_A => crate::Key::KEY_A,
            evdev::Key::KEY_S => crate::Key::KEY_S,
            evdev::Key::KEY_D => crate::Key::KEY_D,
            evdev::Key::KEY_F => crate::Key::KEY_F,
            evdev::Key::KEY_G => crate::Key::KEY_G,
            evdev::Key::KEY_H => crate::Key::KEY_H,
            evdev::Key::KEY_J => crate::Key::KEY_J,
            evdev::Key::KEY_K => crate::Key::KEY_K,
            evdev::Key::KEY_L => crate::Key::KEY_L,
            evdev::Key::KEY_SEMICOLON => crate::Key::KEY_SEMICOLON,
            evdev::Key::KEY_APOSTROPHE => crate::Key::KEY_APOSTROPHE,
            evdev::Key::KEY_GRAVE => crate::Key::KEY_GRAVE,
            evdev::Key::KEY_LEFTSHIFT => crate::Key::KEY_LEFTSHIFT,
            evdev::Key::KEY_BACKSLASH => crate::Key::KEY_BACKSLASH,
            evdev::Key::KEY_Z => crate::Key::KEY_Z,
            evdev::Key::KEY_X => crate::Key::KEY_X,
            evdev::Key::KEY_C => crate::Key::KEY_C,
            evdev::Key::KEY_V => crate::Key::KEY_V,
            evdev::Key::KEY_B => crate::Key::KEY_B,
            evdev::Key::KEY_N => crate::Key::KEY_N,
            evdev::Key::KEY_M => crate::Key::KEY_M,
            evdev::Key::KEY_COMMA => crate::Key::KEY_COMMA,
            evdev::Key::KEY_DOT => crate::Key::KEY_DOT,
            evdev::Key::KEY_SLASH => crate::Key::KEY_SLASH,
            evdev::Key::KEY_RIGHTSHIFT => crate::Key::KEY_RIGHTSHIFT,
            evdev::Key::KEY_KPASTERISK => crate::Key::KEY_KPASTERISK,
            evdev::Key::KEY_LEFTALT => crate::Key::KEY_LEFTALT,
            evdev::Key::KEY_SPACE => crate::Key::KEY_SPACE,
            evdev::Key::KEY_CAPSLOCK => crate::Key::KEY_CAPSLOCK,
            evdev::Key::KEY_F1 => crate::Key::KEY_F1,
            evdev::Key::KEY_F2 => crate::Key::KEY_F2,
            evdev::Key::KEY_F3 => crate::Key::KEY_F3,
            evdev::Key::KEY_F4 => crate::Key::KEY_F4,
            evdev::Key::KEY_F5 => crate::Key::KEY_F5,
            evdev::Key::KEY_F6 => crate::Key::KEY_F6,
            evdev::Key::KEY_F7 => crate::Key::KEY_F7,
            evdev::Key::KEY_F8 => crate::Key::KEY_F8,
            evdev::Key::KEY_F9 => crate::Key::KEY_F9,
            evdev::Key::KEY_F10 => crate::Key::KEY_F10,
            evdev::Key::KEY_NUMLOCK => crate::Key::KEY_NUMLOCK,
            evdev::Key::KEY_SCROLLLOCK => crate::Key::KEY_SCROLLLOCK,
            evdev::Key::KEY_KP7 => crate::Key::KEY_KP7,
            evdev::Key::KEY_KP8 => crate::Key::KEY_KP8,
            evdev::Key::KEY_KP9 => crate::Key::KEY_KP9,
            evdev::Key::KEY_KPMINUS => crate::Key::KEY_KPMINUS,
            evdev::Key::KEY_KP4 => crate::Key::KEY_KP4,
            evdev::Key::KEY_KP5 => crate::Key::KEY_KP5,
            evdev::Key::KEY_KP6 => crate::Key::KEY_KP6,
            evdev::Key::KEY_KPPLUS => crate::Key::KEY_KPPLUS,
            evdev::Key::KEY_KP1 => crate::Key::KEY_KP1,
            evdev::Key::KEY_KP2 => crate::Key::KEY_KP2,
            evdev::Key::KEY_KP3 => crate::Key::KEY_KP3,
            evdev::Key::KEY_KP0 => crate::Key::KEY_KP0,
            evdev::Key::KEY_KPDOT => crate::Key::KEY_KPDOT,
            evdev::Key::KEY_F11 => crate::Key::KEY_F11,
            evdev::Key::KEY_F12 => crate::Key::KEY_F12,
            evdev::Key::KEY_HOME => crate::Key::KEY_HOME,
            evdev::Key::KEY_UP => crate::Key::KEY_UP,
            evdev::Key::KEY_PAGEUP => crate::Key::KEY_PAGEUP,
            evdev::Key::KEY_LEFT => crate::Key::KEY_LEFT,
            evdev::Key::KEY_RIGHT => crate::Key::KEY_RIGHT,
            evdev::Key::KEY_END => crate::Key::KEY_END,
            evdev::Key::KEY_DOWN => crate::Key::KEY_DOWN,
            evdev::Key::KEY_PAGEDOWN => crate::Key::KEY_PAGEDOWN,
            evdev::Key::KEY_INSERT => crate::Key::KEY_INSERT,
            evdev::Key::KEY_DELETE => crate::Key::KEY_DELETE,
            evdev::Key::KEY_LEFTMETA => crate::Key::KEY_LEFTMETA,
            evdev::Key::KEY_RIGHTMETA => crate::Key::KEY_RIGHTMETA,
            evdev::Key::KEY_MENU => crate::Key::KEY_MENU,
            evdev::Key::BTN_LEFT => crate::Key::BTN_LEFT,
            evdev::Key::BTN_RIGHT => crate::Key::BTN_RIGHT,
            evdev::Key::BTN_MIDDLE => crate::Key::BTN_MIDDLE,
            evdev::Key::BTN_SIDE => crate::Key::BTN_SIDE,
            evdev::Key::BTN_EXTRA => crate::Key::BTN_EXTRA,
            evdev::Key::BTN_FORWARD => crate::Key::BTN_FORWARD,
            evdev::Key::BTN_BACK => crate::Key::BTN_BACK,
            _ => unimplemented!("Unsupported key {val:?}"),
        }
    }
}
