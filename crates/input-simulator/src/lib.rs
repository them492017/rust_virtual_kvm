use std::fmt;

use evdev::InputEvent;
use thiserror::Error;

pub mod x11;

#[derive(Debug, Error)]
pub enum DeviceOutputError {
    #[error("")]
    IOError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Keyboard,
    Mouse,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeviceType::Keyboard => write!(f, "Keyboard"),
            DeviceType::Mouse => write!(f, "Mouse"),
        }
    }
}

pub trait VirtualDevice {
    fn emit(&mut self, event: InputEvent) -> Result<(), DeviceOutputError>;
    fn release_all(&mut self) -> Result<(), DeviceOutputError>;
    fn device_type(&self) -> DeviceType;
}
