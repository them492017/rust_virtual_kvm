use std::{io, thread, time::Duration};

use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, Device, EventType, InputEvent, Key, RelativeAxisType,
};

use super::error::DynError;

pub fn pick_device(name: &str) -> evdev::Device {
    use std::io::prelude::*;

    let mut devices = evdev::enumerate().map(|t| t.1).collect::<Vec<_>>();
    // readdir returns them in reverse order from their eventN names for some reason
    devices.reverse();
    for (i, d) in devices.iter().enumerate() {
        println!("{}: {}", i, d.name().unwrap_or("Unnamed device"));
    }
    print!("Select the {} device [0-{}]: ", name, devices.len());
    let _ = std::io::stdout().flush();
    let mut chosen = String::new();
    std::io::stdin().read_line(&mut chosen).unwrap();
    let n = chosen.trim().parse::<usize>().unwrap();
    devices.into_iter().nth(n).unwrap()
}

pub fn make_keyboard() -> std::io::Result<VirtualDevice> {
    let device = VirtualDeviceBuilder::new()?
        .name("Fake KVM Keyboard")
        .with_keys(&AttributeSet::from_iter(ALL_KEYS))?
        .build();
    thread::sleep(Duration::from_millis(100)); // delay so events will be emitted instantly
    device
}

pub fn make_mouse() -> std::io::Result<VirtualDevice> {
    let device = VirtualDeviceBuilder::new()?
        .name("Fake KVM Mouse")
        .with_relative_axes(&AttributeSet::from_iter([
            RelativeAxisType::REL_X,
            RelativeAxisType::REL_Y,
            RelativeAxisType::REL_WHEEL,
            RelativeAxisType::REL_HWHEEL,
        ]))?
        .build();
    thread::sleep(Duration::from_millis(100)); // delay so events will be emitted instantly
    device
}

pub trait InputDevice {
    fn emit(&mut self, messages: &[InputEvent]) -> io::Result<()>;
}

impl InputDevice for VirtualDevice {
    fn emit(&mut self, messages: &[InputEvent]) -> io::Result<()> {
        self.emit(messages)
    }
}

impl InputDevice for Device {
    fn emit(&mut self, messages: &[InputEvent]) -> io::Result<()> {
        self.send_events(messages)
    }
}

pub fn release_all<T: InputDevice>(device: &mut T) -> Result<(), DynError> {
    // TODO: consider device.supported_keys()
    println!("Releasing all keys");
    ALL_KEYS.iter().for_each(|key| {
        println!("Releasing {key:?}");
        device
            .emit(&[InputEvent::new(EventType::KEY, key.code(), 0)])
            .unwrap();
    });

    Ok(())
}

const ALL_KEYS: [Key; 202] = [
    Key::KEY_RESERVED,
    Key::KEY_ESC,
    Key::KEY_1,
    Key::KEY_2,
    Key::KEY_3,
    Key::KEY_4,
    Key::KEY_5,
    Key::KEY_6,
    Key::KEY_7,
    Key::KEY_8,
    Key::KEY_9,
    Key::KEY_0,
    Key::KEY_MINUS,
    Key::KEY_EQUAL,
    Key::KEY_BACKSPACE,
    Key::KEY_TAB,
    Key::KEY_Q,
    Key::KEY_W,
    Key::KEY_E,
    Key::KEY_R,
    Key::KEY_T,
    Key::KEY_Y,
    Key::KEY_U,
    Key::KEY_I,
    Key::KEY_O,
    Key::KEY_P,
    Key::KEY_LEFTBRACE,
    Key::KEY_RIGHTBRACE,
    Key::KEY_ENTER,
    Key::KEY_LEFTCTRL,
    Key::KEY_A,
    Key::KEY_S,
    Key::KEY_D,
    Key::KEY_F,
    Key::KEY_G,
    Key::KEY_H,
    Key::KEY_J,
    Key::KEY_K,
    Key::KEY_L,
    Key::KEY_SEMICOLON,
    Key::KEY_APOSTROPHE,
    Key::KEY_GRAVE,
    Key::KEY_LEFTSHIFT,
    Key::KEY_BACKSLASH,
    Key::KEY_Z,
    Key::KEY_X,
    Key::KEY_C,
    Key::KEY_V,
    Key::KEY_B,
    Key::KEY_N,
    Key::KEY_M,
    Key::KEY_COMMA,
    Key::KEY_DOT,
    Key::KEY_SLASH,
    Key::KEY_RIGHTSHIFT,
    Key::KEY_KPASTERISK,
    Key::KEY_LEFTALT,
    Key::KEY_SPACE,
    Key::KEY_CAPSLOCK,
    Key::KEY_F1,
    Key::KEY_F2,
    Key::KEY_F3,
    Key::KEY_F4,
    Key::KEY_F5,
    Key::KEY_F6,
    Key::KEY_F7,
    Key::KEY_F8,
    Key::KEY_F9,
    Key::KEY_F10,
    Key::KEY_NUMLOCK,
    Key::KEY_SCROLLLOCK,
    Key::KEY_KP7,
    Key::KEY_KP8,
    Key::KEY_KP9,
    Key::KEY_KPMINUS,
    Key::KEY_KP4,
    Key::KEY_KP5,
    Key::KEY_KP6,
    Key::KEY_KPPLUS,
    Key::KEY_KP1,
    Key::KEY_KP2,
    Key::KEY_KP3,
    Key::KEY_KP0,
    Key::KEY_KPDOT,
    Key::KEY_ZENKAKUHANKAKU,
    Key::KEY_102ND,
    Key::KEY_F11,
    Key::KEY_F12,
    Key::KEY_RO,
    Key::KEY_KATAKANA,
    Key::KEY_HIRAGANA,
    Key::KEY_HENKAN,
    Key::KEY_KATAKANAHIRAGANA,
    Key::KEY_MUHENKAN,
    Key::KEY_KPJPCOMMA,
    Key::KEY_KPENTER,
    Key::KEY_RIGHTCTRL,
    Key::KEY_KPSLASH,
    Key::KEY_SYSRQ,
    Key::KEY_RIGHTALT,
    Key::KEY_LINEFEED,
    Key::KEY_HOME,
    Key::KEY_UP,
    Key::KEY_PAGEUP,
    Key::KEY_LEFT,
    Key::KEY_RIGHT,
    Key::KEY_END,
    Key::KEY_DOWN,
    Key::KEY_PAGEDOWN,
    Key::KEY_INSERT,
    Key::KEY_DELETE,
    Key::KEY_MACRO,
    Key::KEY_MUTE,
    Key::KEY_VOLUMEDOWN,
    Key::KEY_VOLUMEUP,
    Key::KEY_POWER,
    Key::KEY_KPEQUAL,
    Key::KEY_KPPLUSMINUS,
    Key::KEY_PAUSE,
    Key::KEY_SCALE,
    Key::KEY_KPCOMMA,
    Key::KEY_HANGEUL,
    Key::KEY_HANJA,
    Key::KEY_YEN,
    Key::KEY_LEFTMETA,
    Key::KEY_RIGHTMETA,
    Key::KEY_COMPOSE,
    Key::KEY_STOP,
    Key::KEY_AGAIN,
    Key::KEY_PROPS,
    Key::KEY_UNDO,
    Key::KEY_FRONT,
    Key::KEY_COPY,
    Key::KEY_OPEN,
    Key::KEY_PASTE,
    Key::KEY_FIND,
    Key::KEY_CUT,
    Key::KEY_HELP,
    Key::KEY_MENU,
    Key::KEY_CALC,
    Key::KEY_SETUP,
    Key::KEY_SLEEP,
    Key::KEY_WAKEUP,
    Key::KEY_FILE,
    Key::KEY_SENDFILE,
    Key::KEY_DELETEFILE,
    Key::KEY_XFER,
    Key::KEY_PROG1,
    Key::KEY_PROG2,
    Key::KEY_WWW,
    Key::KEY_MSDOS,
    Key::KEY_COFFEE,
    Key::KEY_DIRECTION,
    Key::KEY_ROTATE_DISPLAY,
    Key::KEY_CYCLEWINDOWS,
    Key::KEY_MAIL,
    Key::KEY_BOOKMARKS,
    Key::KEY_COMPUTER,
    Key::KEY_BACK,
    Key::KEY_FORWARD,
    Key::KEY_CLOSECD,
    Key::KEY_EJECTCD,
    Key::KEY_EJECTCLOSECD,
    Key::KEY_NEXTSONG,
    Key::KEY_PLAYPAUSE,
    Key::KEY_PREVIOUSSONG,
    Key::KEY_STOPCD,
    Key::KEY_RECORD,
    Key::KEY_REWIND,
    Key::KEY_PHONE,
    Key::KEY_ISO,
    Key::KEY_CONFIG,
    Key::KEY_HOMEPAGE,
    Key::KEY_REFRESH,
    Key::KEY_EXIT,
    Key::KEY_MOVE,
    Key::KEY_EDIT,
    Key::KEY_SCROLLUP,
    Key::KEY_SCROLLDOWN,
    Key::KEY_KPLEFTPAREN,
    Key::KEY_KPRIGHTPAREN,
    Key::KEY_NEW,
    Key::KEY_REDO,
    Key::KEY_F13,
    Key::KEY_F14,
    Key::KEY_F15,
    Key::KEY_F16,
    Key::KEY_F17,
    Key::KEY_F18,
    Key::KEY_F19,
    Key::KEY_F20,
    Key::KEY_F21,
    Key::KEY_F22,
    Key::KEY_F23,
    Key::KEY_F24,
    Key::BTN_LEFT,
    Key::BTN_RIGHT,
    Key::BTN_MIDDLE,
    Key::BTN_SIDE,
    Key::BTN_EXTRA,
    Key::BTN_FORWARD,
    Key::BTN_BACK,
];
