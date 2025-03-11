use std::net::SocketAddr;

use chacha20poly1305::Nonce;
use thiserror::Error;
use tokio::net::UdpSocket;

use crate::common::{
    crypto::Crypto,
    error::DynError,
    net::{Message, MessageWithNonce},
};

#[derive(Debug, Error)]
pub enum InputEventError {
    #[error("Encryption error")]
    EncryptionError(#[from] DynError), // should be opaque
    #[error("Serialisation error")]
    BincodeError(#[from] bincode::Error),
    #[error("IO error")]
    IOError(#[from] std::io::Error),
}

pub struct InputEventTransport {
    socket: UdpSocket,
}

impl InputEventTransport {
    pub fn new(socket: UdpSocket) -> Self {
        InputEventTransport { socket }
    }

    pub async fn send_message_to<T: Crypto>(
        &mut self,
        message: Message,
        address: SocketAddr,
        encryptor: Option<T>,
    ) -> Result<(), InputEventError> {
        let encoded_message: Vec<u8> = bincode::serialize(&message)?;

        let (encrypted, nonce) = if let Some(encryptor) = encryptor {
            encryptor.encrypt(encoded_message)?
        } else {
            (encoded_message, Nonce::default())
        };

        let message_with_nonce = MessageWithNonce::new(encrypted, nonce);
        let encoded_with_nonce: Vec<u8> = bincode::serialize(&message_with_nonce)?;

        self.socket.send_to(&encoded_with_nonce, address).await?;
        Ok(())
    }
}
