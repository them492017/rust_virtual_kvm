use std::ffi::c_uchar;

use x11::{
    keysym,
    xlib::{Display, XKeysymToKeycode},
};

use crate::{mapper::error::EventMappingError, Key};

impl Key {
    pub fn to_x11_keycode(self, display: *mut Display) -> Result<c_uchar, EventMappingError> {
        let keysym = match self {
            Key::KEY_ESC => keysym::XK_Escape,
            Key::KEY_1 => keysym::XK_1,
            Key::KEY_2 => keysym::XK_2,
            Key::KEY_3 => keysym::XK_3,
            Key::KEY_4 => keysym::XK_4,
            Key::KEY_5 => keysym::XK_5,
            Key::KEY_6 => keysym::XK_6,
            Key::KEY_7 => keysym::XK_7,
            Key::KEY_8 => keysym::XK_8,
            Key::KEY_9 => keysym::XK_9,
            Key::KEY_0 => keysym::XK_0,
            Key::KEY_MINUS => keysym::XK_minus,
            Key::KEY_EQUAL => keysym::XK_equal,
            Key::KEY_BACKSPACE => keysym::XK_BackSpace,
            Key::KEY_TAB => keysym::XK_Tab,
            Key::KEY_Q => keysym::XK_Q,
            Key::KEY_W => keysym::XK_W,
            Key::KEY_E => keysym::XK_E,
            Key::KEY_R => keysym::XK_R,
            Key::KEY_T => keysym::XK_T,
            Key::KEY_Y => keysym::XK_Y,
            Key::KEY_U => keysym::XK_U,
            Key::KEY_I => keysym::XK_I,
            Key::KEY_O => keysym::XK_O,
            Key::KEY_P => keysym::XK_P,
            Key::KEY_LEFTBRACE => keysym::XK_bracketleft,
            Key::KEY_RIGHTBRACE => keysym::XK_bracketright,
            Key::KEY_ENTER => keysym::XK_Return,
            Key::KEY_LEFTCTRL => keysym::XK_Control_L,
            Key::KEY_A => keysym::XK_A,
            Key::KEY_S => keysym::XK_S,
            Key::KEY_D => keysym::XK_D,
            Key::KEY_F => keysym::XK_F,
            Key::KEY_G => keysym::XK_G,
            Key::KEY_H => keysym::XK_H,
            Key::KEY_J => keysym::XK_J,
            Key::KEY_K => keysym::XK_K,
            Key::KEY_L => keysym::XK_L,
            Key::KEY_SEMICOLON => keysym::XK_semicolon,
            Key::KEY_APOSTROPHE => keysym::XK_apostrophe,
            Key::KEY_GRAVE => keysym::XK_grave,
            Key::KEY_LEFTSHIFT => keysym::XK_Shift_L,
            Key::KEY_BACKSLASH => keysym::XK_backslash,
            Key::KEY_Z => keysym::XK_Z,
            Key::KEY_X => keysym::XK_X,
            Key::KEY_C => keysym::XK_C,
            Key::KEY_V => keysym::XK_V,
            Key::KEY_B => keysym::XK_B,
            Key::KEY_N => keysym::XK_N,
            Key::KEY_M => keysym::XK_M,
            Key::KEY_COMMA => keysym::XK_comma,
            Key::KEY_DOT => keysym::XK_period,
            Key::KEY_SLASH => keysym::XK_slash,
            Key::KEY_RIGHTSHIFT => keysym::XK_Shift_R,
            Key::KEY_KPASTERISK => keysym::XK_KP_Multiply,
            Key::KEY_LEFTALT => keysym::XK_Alt_L,
            Key::KEY_SPACE => keysym::XK_space,
            Key::KEY_CAPSLOCK => keysym::XK_Caps_Lock,
            Key::KEY_F1 => keysym::XK_F1,
            Key::KEY_F2 => keysym::XK_F2,
            Key::KEY_F3 => keysym::XK_F3,
            Key::KEY_F4 => keysym::XK_F4,
            Key::KEY_F5 => keysym::XK_F5,
            Key::KEY_F6 => keysym::XK_F6,
            Key::KEY_F7 => keysym::XK_F7,
            Key::KEY_F8 => keysym::XK_F8,
            Key::KEY_F9 => keysym::XK_F9,
            Key::KEY_F10 => keysym::XK_F10,
            Key::KEY_NUMLOCK => keysym::XK_Num_Lock,
            Key::KEY_SCROLLLOCK => keysym::XK_Scroll_Lock,
            Key::KEY_KP7 => keysym::XK_KP_7,
            Key::KEY_KP8 => keysym::XK_KP_8,
            Key::KEY_KP9 => keysym::XK_KP_9,
            Key::KEY_KPMINUS => keysym::XK_KP_Subtract,
            Key::KEY_KP4 => keysym::XK_KP_4,
            Key::KEY_KP5 => keysym::XK_KP_5,
            Key::KEY_KP6 => keysym::XK_KP_6,
            Key::KEY_KPPLUS => keysym::XK_KP_Add,
            Key::KEY_KP1 => keysym::XK_KP_1,
            Key::KEY_KP2 => keysym::XK_KP_2,
            Key::KEY_KP3 => keysym::XK_KP_3,
            Key::KEY_KP0 => keysym::XK_KP_0,
            Key::KEY_KPDOT => keysym::XK_KP_Decimal,
            Key::KEY_F11 => keysym::XK_F11,
            Key::KEY_F12 => keysym::XK_F12,
            Key::KEY_HOME => keysym::XK_Home,
            Key::KEY_UP => keysym::XK_Up,
            Key::KEY_PAGEUP => keysym::XK_Prior,
            Key::KEY_LEFT => keysym::XK_Left,
            Key::KEY_RIGHT => keysym::XK_Right,
            Key::KEY_END => keysym::XK_End,
            Key::KEY_DOWN => keysym::XK_Down,
            Key::KEY_PAGEDOWN => keysym::XK_Next,
            Key::KEY_INSERT => keysym::XK_Insert,
            Key::KEY_DELETE => keysym::XK_Delete,
            Key::KEY_LEFTMETA => keysym::XK_Super_L,
            Key::KEY_RIGHTMETA => keysym::XK_Super_R,
            Key::KEY_MENU => keysym::XK_Menu,
            _ => return Err(EventMappingError::UnsupportedKeyError), // TODO: maybe change
        };

        unsafe { Ok(XKeysymToKeycode(display, keysym.into())) }
    }
}
