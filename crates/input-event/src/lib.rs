use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

pub mod mapper;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum InputEvent {
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum KeyboardEvent {
    KeyPressed(Key),
    KeyReleased(Key),
    KeyHeld(Key),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum MouseEvent {
    Motion {
        axis: PointerAxis,
        diff: i32,
    },
    Button {
        event_type: KeyboardEventType,
        button: Key,
    },
    Scroll {
        axis: PointerAxis,
        diff: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum PointerAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum KeyboardEventType {
    KeyPressed,
    KeyReleased,
    KeyHeld,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum DeviceType {
    Keyboard,
    Mouse,
}

#[allow(non_camel_case_types)]
#[derive(Debug, EnumIter, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum Key {
    KEY_ESC,
    KEY_1,
    KEY_2,
    KEY_3,
    KEY_4,
    KEY_5,
    KEY_6,
    KEY_7,
    KEY_8,
    KEY_9,
    KEY_0,
    KEY_MINUS,
    KEY_EQUAL,
    KEY_BACKSPACE,
    KEY_TAB,
    KEY_Q,
    KEY_W,
    KEY_E,
    KEY_R,
    KEY_T,
    KEY_Y,
    KEY_U,
    KEY_I,
    KEY_O,
    KEY_P,
    KEY_LEFTBRACE,
    KEY_RIGHTBRACE,
    KEY_ENTER,
    KEY_LEFTCTRL,
    KEY_A,
    KEY_S,
    KEY_D,
    KEY_F,
    KEY_G,
    KEY_H,
    KEY_J,
    KEY_K,
    KEY_L,
    KEY_SEMICOLON,
    KEY_APOSTROPHE,
    KEY_GRAVE,
    KEY_LEFTSHIFT,
    KEY_BACKSLASH,
    KEY_Z,
    KEY_X,
    KEY_C,
    KEY_V,
    KEY_B,
    KEY_N,
    KEY_M,
    KEY_COMMA,
    KEY_DOT,
    KEY_SLASH,
    KEY_RIGHTSHIFT,
    KEY_KPASTERISK,
    KEY_LEFTALT,
    KEY_SPACE,
    KEY_CAPSLOCK,
    KEY_F1,
    KEY_F2,
    KEY_F3,
    KEY_F4,
    KEY_F5,
    KEY_F6,
    KEY_F7,
    KEY_F8,
    KEY_F9,
    KEY_F10,
    KEY_NUMLOCK,
    KEY_SCROLLLOCK,
    KEY_KP7,
    KEY_KP8,
    KEY_KP9,
    KEY_KPMINUS,
    KEY_KP4,
    KEY_KP5,
    KEY_KP6,
    KEY_KPPLUS,
    KEY_KP1,
    KEY_KP2,
    KEY_KP3,
    KEY_KP0,
    KEY_KPDOT,
    KEY_F11,
    KEY_F12,
    KEY_HOME,
    KEY_UP,
    KEY_PAGEUP,
    KEY_LEFT,
    KEY_RIGHT,
    KEY_END,
    KEY_DOWN,
    KEY_PAGEDOWN,
    KEY_INSERT,
    KEY_DELETE,
    KEY_LEFTMETA,
    KEY_RIGHTMETA,
    KEY_MENU,
    BTN_LEFT,
    BTN_RIGHT,
    BTN_MIDDLE,
    BTN_SIDE,
    BTN_EXTRA,
    BTN_FORWARD,
    BTN_BACK,
}
