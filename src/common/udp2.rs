// TODO: This should be removed
use std::net::SocketAddr;

use chacha20poly1305::Nonce;
use tokio::net::UdpSocket;

use super::{
    crypto::Crypto,
    error::DynError,
    net::{Message, MessageWithNonce},
    transport::{decrypt_and_deserialise_message, AsyncTransport},
};

const BUFFER_LEN: usize = 256;

pub struct TargetlessUdpTransport<T: Crypto> {
    socket: UdpSocket,
    address: SocketAddr,
    symmetric_key: Option<T>,
}

impl<T: Crypto> TargetlessUdpTransport<T> {
    pub fn new(socket: UdpSocket, address: SocketAddr) -> Self {
        TargetlessUdpTransport {
            socket,
            address,
            symmetric_key: None,
        }
    }

    pub fn set_address(&mut self, address: SocketAddr) {
        self.address = address
    }

    pub fn set_key(&mut self, key: T) {
        self.symmetric_key = Some(key)
    }
}

impl<T: Crypto> AsyncTransport for TargetlessUdpTransport<T> {
    async fn send_message(&mut self, message: Message) -> Result<(), DynError> {
        let encoded_message: Vec<u8> = bincode::serialize(&message)?;

        let (encrypted, nonce) = if let Some(encryptor) = &self.symmetric_key {
            encryptor.encrypt(encoded_message)?
        } else {
            (encoded_message, Nonce::default())
        };

        let message_with_nonce = MessageWithNonce::new(encrypted, nonce);
        let encoded_with_nonce: Vec<u8> = bincode::serialize(&message_with_nonce)?;

        self.socket
            .send_to(&encoded_with_nonce, self.address)
            .await?;
        Ok(())
    }

    async fn receive_message(&mut self) -> Result<Message, DynError> {
        let mut buf = [0; BUFFER_LEN];
        let bytes_read = self.socket.recv(&mut buf).await?;

        if bytes_read == 0 {
            return Err("Connection closed".into());
        }

        decrypt_and_deserialise_message(&buf[..bytes_read], &self.symmetric_key)
    }
}
