use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventMappingError {
    #[error("Event type is unsupported")]
    UnsupportedEventError,
    #[error("Key is unsupported")]
    UnsupportedKeyError,
    #[error("Invalid event")]
    InvalidEventError,
}
