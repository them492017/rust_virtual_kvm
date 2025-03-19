use input_event::{mapper::error::EventMappingError, InputEvent};
use thiserror::Error;
use x11::xtest::X11VirtualDevice;

pub mod x11;

#[derive(Debug, Error)]
pub enum DeviceOutputError {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Event emitting error: {0}")]
    EmitError(String), // TODO: update this
    #[error("Event conversion error: {0}")]
    ConversionError(#[from] EventMappingError),
}

pub trait VirtualDevice: Send {
    fn emit(&mut self, event: InputEvent) -> Result<(), DeviceOutputError>;
    fn release_all(&mut self) -> Result<(), DeviceOutputError>;
}

pub struct InputSimulator {
    virtual_device: Box<dyn VirtualDevice + Send + Sync>,
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
