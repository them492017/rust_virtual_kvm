use input_event::InputEvent;
use thiserror::Error;
use x11::xtest::X11VirtualDevice;

pub mod x11;

#[derive(Debug, Error)]
pub enum DeviceOutputError {
    #[error("IO error")]
    IOError(#[from] std::io::Error),
    #[error("Event emitting error")]
    EmitError(String), // TODO: update this
}

pub trait VirtualDevice: Send {
    fn emit(&mut self, event: InputEvent) -> Result<(), DeviceOutputError>;
    fn release_all(&mut self) -> Result<(), DeviceOutputError>;
}

pub struct InputSimulator {
    virtual_device: Box<dyn VirtualDevice + Send>,
}

impl InputSimulator {
    // TODO: make this platform agnostic
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let virtual_device = Box::new(X11VirtualDevice::new());
        InputSimulator { virtual_device }
    }

    pub fn emit(&mut self, event: InputEvent) -> Result<(), DeviceOutputError> {
        self.virtual_device.emit(event)
    }

    pub fn release_all(&mut self) -> Result<(), DeviceOutputError> {
        self.virtual_device.release_all()
    }
}
