use evdev::Device;

use crate::common::dev::pick_device;

pub struct Config {
    pub keyboard: Device,
    pub mouse: Device,
}

pub fn init() -> Config {
    let keyboard = pick_device("keyboard");
    let mouse = pick_device("mouse");

    Config {
        keyboard,
        mouse,
    }
}

