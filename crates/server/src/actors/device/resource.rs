use input_listener::{x11::dev::pick_device, DeviceInputStream};
use input_simulator::InputSimulator;

pub struct DeviceResource {
    pub kbd_input_stream: DeviceInputStream,
    pub mouse_input_stream: DeviceInputStream,
    pub input_simulator: InputSimulator,
}

impl DeviceResource {
    pub fn new() -> Self {
        // TODO: remove unwraps?
        let kbd_input_stream =
            DeviceInputStream::new(pick_device("Keyboard").into_event_stream().unwrap());
        let mouse_input_stream =
            DeviceInputStream::new(pick_device("Mouse").into_event_stream().unwrap());
        let input_simulator = InputSimulator::new();
        DeviceResource {
            kbd_input_stream,
            mouse_input_stream,
            input_simulator,
        }
    }
}
