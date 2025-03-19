use std::pin::Pin;

use input_event::{mapper::error::EventMappingError, InputEvent};
use thiserror::Error;

pub mod x11;

#[derive(Debug, Error)]
pub enum DeviceInputError {
    #[error("IO error")]
    IOError(#[from] std::io::Error),
    #[error("Error when converting backend specific input event to generic event")]
    InputEventConversionError(#[from] EventMappingError), // TODO: figure out how to enforce that
                                                          // this is a mapping error only
}

// TODO: Consider making a sync version
// TODO: Consider implementing the Stream<Item = Result<InputEvent, DeviceInputError>> trait
pub trait DeviceInputStreamTrait: Send {
    fn next_event(
        &mut self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<InputEvent, DeviceInputError>> + Send + '_>>;
    fn grab_device(&mut self) -> Result<(), DeviceInputError>;
    fn ungrab_device(&mut self) -> Result<(), DeviceInputError>;
}

pub struct DeviceInputStream {
    stream: Box<dyn DeviceInputStreamTrait + Send + Sync>,
}

impl DeviceInputStream {
    // TODO: support multiple platforms
    pub fn new(stream: evdev::EventStream) -> Self {
        let stream = Box::new(stream);
        DeviceInputStream { stream }
    }

    pub async fn next_event(&mut self) -> Result<InputEvent, DeviceInputError> {
        self.stream.next_event().await
    }

    pub fn grab_device(&mut self) -> Result<(), DeviceInputError> {
        self.stream.grab_device()
    }

    pub fn ungrab_device(&mut self) -> Result<(), DeviceInputError> {
        self.stream.ungrab_device()
    }
}
