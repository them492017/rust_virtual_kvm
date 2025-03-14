use network::Message;
use thiserror::Error;

use crate::client::ClientConnectionError;

#[derive(Debug, Error)]
pub enum StateHandlerError {
    #[error("Client is disconnected")]
    ClientDisconnected,
    #[error("Client not found")]
    NotFound,
    // TODO: consider separating idx error
    #[error("Grab request channel send error")]
    GrabSendError(#[from] tokio::sync::broadcast::error::SendError<bool>),
    #[error("Grab request channel send error")]
    MessageSendError(#[from] tokio::sync::mpsc::error::SendError<Message>),
    #[error("Client error")]
    ClientError(#[from] ClientConnectionError),
}
