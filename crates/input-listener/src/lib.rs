use std::pin::Pin;

use evdev::InputEvent;
use thiserror::Error;

pub mod x11;

// TODO: maybe should have a shared error type for all platforms
#[derive(Debug, Error)]
pub enum DeviceInputError {
    #[error("")]
    IOError(#[from] std::io::Error),
}

// TODO: Consider making a sync version
pub trait DeviceInputStreamTrait: Send {
    fn next_event(
        &mut self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<InputEvent, DeviceInputError>> + Send + '_>>;
    fn grab_device(&mut self) -> Result<(), DeviceInputError>;
    fn ungrab_device(&mut self) -> Result<(), DeviceInputError>;
}

pub struct DeviceInputStream {
    stream: Box<dyn DeviceInputStreamTrait>,
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
