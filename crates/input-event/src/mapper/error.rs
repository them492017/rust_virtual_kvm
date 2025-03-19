use thiserror::Error;

use crate::{InputEvent, Key, KeyboardEventType};

#[derive(Debug, Error)]
pub enum EventMappingError {
    #[error("Event type is unsupported")]
    UnsupportedEventType,
    #[error("Key {0} is unsupported")]
    UnsupportedKeyError(Key),
    #[error("Error when mapping an invalid event")]
    InvalidEvent,
}
