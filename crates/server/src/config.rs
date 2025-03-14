use input_listener::{x11::dev::pick_device, DeviceInputStreamTrait};

pub struct Config {
    pub keyboard_stream: Box<dyn DeviceInputStreamTrait>,
    pub mouse_stream: Box<dyn DeviceInputStreamTrait>,
}

pub fn init() -> Config {
    let keyboard_stream = Box::new(pick_device("keyboard").into_event_stream().unwrap());
    let mouse_stream = Box::new(pick_device("mouse").into_event_stream().unwrap());

    Config {
        keyboard_stream,
        mouse_stream,
    }
}
