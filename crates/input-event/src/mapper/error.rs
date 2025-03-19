use thiserror::Error;

use crate::Key;

#[derive(Debug, Error)]
pub enum EventMappingError {
    #[error("Event type is unsupported")]
    UnsupportedEventType,
    #[error("Key {0} is unsupported")]
    UnsupportedKeyError(Key),
    #[error("Error when mapping an invalid event")]
    InvalidEvent,
    #[error("Argument could not be mapped to a key")]
    UnknownKey,
    #[error("Argument could not be mapped to a button")]
    UnknownButton,
}
