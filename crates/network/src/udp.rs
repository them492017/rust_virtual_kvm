use std::net::SocketAddr;

use chacha20poly1305::Nonce;
use crypto::Crypto;
use tokio::net::UdpSocket;

use crate::{Message, MessageWithNonce, TransportError};

use super::transport::{decrypt_and_deserialise_message, Transport};

const BUFFER_LEN: usize = 256;

pub struct TokioUdpTransport<T: Crypto> {
    socket: UdpSocket,
    server_addr: SocketAddr,
    symmetric_key: Option<T>,
}

impl<T: Crypto> TokioUdpTransport<T> {
    pub fn new(socket: UdpSocket, server_addr: SocketAddr, symmetric_key: Option<T>) -> Self {
        TokioUdpTransport {
            socket,
            server_addr,
            symmetric_key,
        }
    }
}

impl<T: Crypto> Transport for TokioUdpTransport<T> {
    async fn send_message(&mut self, message: Message) -> Result<(), TransportError> {
        let encoded_message: Vec<u8> = bincode::serialize(&message)?;

        let (encrypted, nonce) = if let Some(encryptor) = &self.symmetric_key {
            encryptor.encrypt(encoded_message)?
        } else {
            (encoded_message, Nonce::default())
        };

        let message_with_nonce = MessageWithNonce::new(encrypted, nonce);
        let encoded_with_nonce: Vec<u8> = bincode::serialize(&message_with_nonce)?;

        self.socket
            .send_to(&encoded_with_nonce, self.server_addr)
            .await?;
        Ok(())
    }

    async fn receive_message(&mut self) -> Result<Message, TransportError> {
        let mut buf = [0; BUFFER_LEN];
        let bytes_read = self.socket.recv(&mut buf).await?;

        if bytes_read == 0 {
            return Err(TransportError::ConnectionClosed);
        }

        decrypt_and_deserialise_message(&buf[..bytes_read], &self.symmetric_key)
            .inspect_err(|e| eprintln!("Error while decrypting and deserialising: {}", e))
    }
}
