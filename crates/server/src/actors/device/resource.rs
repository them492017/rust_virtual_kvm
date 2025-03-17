use input_listener::{x11::dev::pick_device, DeviceInputStream};
use input_simulator::{
    x11::dev::{make_keyboard, make_mouse},
    VirtualDevice,
};

pub struct DeviceResource {
    pub input_stream: DeviceInputStream,
    pub virtual_kbd: Box<dyn VirtualDevice + Send + Sync>,
    pub virtual_mouse: Box<dyn VirtualDevice + Send + Sync>,
}

impl DeviceResource {
    pub fn new(device_type: &str) -> Self {
        // TODO: remove unwraps?
        let input_stream =
            DeviceInputStream::new(pick_device(device_type).into_event_stream().unwrap());
        let virtual_kbd = Box::new(make_keyboard().unwrap());
        let virtual_mouse = Box::new(make_mouse().unwrap());
        DeviceResource {
            input_stream,
            virtual_kbd,
            virtual_mouse,
        }
    }
}
