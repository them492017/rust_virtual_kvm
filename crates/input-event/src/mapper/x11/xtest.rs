use std::ffi::{c_uchar, c_uint};

use x11::{
    keysym,
    xlib::{Display, XKeysymToKeycode},
};

use crate::{mapper::error::EventMappingError, Button, Key, PointerAxis};

impl Key {
    /// Converts a `Key` to an X11 keycode using the provided display.
    ///
    /// # Parameters
    /// - `display`: A pointer to an X11 `Display`
    ///
    /// # Returns
    /// - `Ok(c_uchar)`: The corresponding X11 keycode.
    /// - `Err(EventMappingError)`: If the mapping fails.
    ///
    /// # Safety
    /// This function assumes the provided `display` pointer is valid and properly initialized.
    pub unsafe fn to_x11_keycode(
        self,
        display: *mut Display,
    ) -> Result<c_uchar, EventMappingError> {
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
        };

        unsafe { Ok(XKeysymToKeycode(display, keysym.into())) }
    }
}

impl Button {
    /// Converts a `Button` to its corresponding X11 button number.
    ///
    /// # Returns
    /// - X11 button number for the given button:
    ///   - **Left Button:** `1`
    ///   - **Right Button:** `3`
    ///   - **Middle Button:** `2`
    ///   - **Forward Button:** `9`
    ///   - **Back Button:** `8`
    pub fn to_x11_button_num(self) -> c_uint {
        match self {
            Button::BTN_LEFT => 1,
            Button::BTN_RIGHT => 3,
            Button::BTN_MIDDLE => 2,
            Button::BTN_FORWARD => 9,
            Button::BTN_BACK => 8,
        }
    }
}

impl PointerAxis {
    /// Converts a scroll direction into the corresponding X11 button number.
    ///
    /// # Parameters
    /// - `direction`: `true` for positive direction (up or right), `false` for negative direction (down or left).
    ///
    /// # Returns
    /// - The X11 button number corresponding to the given axis and direction:
    ///   - **Vertical:** Up (`4`), Down (`5`)
    ///   - **Horizontal:** Right (`7`), Left (`6`)
    pub fn to_x11_button_num(self, direction: bool) -> c_uint {
        match self {
            PointerAxis::Vertical if direction => 4,
            PointerAxis::Vertical => 5,
            PointerAxis::Horizontal if direction => 7,
            PointerAxis::Horizontal => 6,
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn given_some_key_should_map_to_corresponding_x11_keycode() {
    }
}
