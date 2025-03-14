use evdev::InputEvent;
use thiserror::Error;

pub mod x11;

#[derive(Debug, Error)]
pub enum DeviceOutputError {
    #[error("")]
    IOError(#[from] std::io::Error),
}

pub trait VirtualDevice {
    fn emit(&mut self, event: InputEvent) -> Result<(), DeviceOutputError>;
    fn release_all(&mut self) -> Result<(), DeviceOutputError>;
}
