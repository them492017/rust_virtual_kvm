use chacha20poly1305::Nonce;
use crypto::Crypto;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

use crate::{Message, MessageWithNonce, TransportError};

use super::transport::{
    decrypt_and_deserialise_message, Transport, TransportReader, TransportWriter,
};

const HEADER_LEN: usize = 4;
const BUFFER_LEN: usize = 256;

#[derive(Debug)]
pub struct TokioTcpTransport<T: Crypto> {
    socket: TcpStream,
    key: Option<T>,
    curr: Vec<u8>,
}

impl<T: Crypto> TokioTcpTransport<T> {
    pub fn new(socket: TcpStream) -> Self {
        TokioTcpTransport {
            socket,
            key: None,
            curr: Vec::new(),
        }
    }

    pub fn set_key(&mut self, key: T) {
        self.key = Some(key);
    }

    fn extract_msg_len(&self) -> Result<usize, TransportError> {
        let prefix_bytes: [u8; HEADER_LEN] = self.curr[..HEADER_LEN]
            .try_into()
            .map_err(|_| TransportError::InvalidMessageStructure)?;
        u32::from_le_bytes(prefix_bytes)
            .try_into()
            .map_err(|_| TransportError::ByteArrayConversionError)
    }
}

impl<T: Crypto + Clone> TokioTcpTransport<T> {
    pub fn into_split(self) -> (TokioTcpTransportReader<T>, TokioTcpTransportWriter<T>) {
        let (reader, writer) = self.socket.into_split();
        let reader_transport = TokioTcpTransportReader::new(reader, self.key.clone(), self.curr);
        let writer_transport = TokioTcpTransportWriter::new(writer, self.key);
        (reader_transport, writer_transport)
    }
}

impl<T: Crypto + Clone> Transport for TokioTcpTransport<T> {
    async fn send_message(&mut self, message: Message) -> Result<(), TransportError> {
        let encoded_message: Vec<u8> = bincode::serialize(&message)?;

        let (encrypted, nonce) = if let Some(encryptor) = &self.key {
            encryptor.encrypt(encoded_message)?
        } else {
            (encoded_message, Nonce::default())
        };

        let message_with_nonce = MessageWithNonce::new(encrypted, nonce);
        let encoded_with_nonce: Vec<u8> = bincode::serialize(&message_with_nonce)?;

        let message_len = encoded_with_nonce.len() as u32;
        let final_message: Vec<u8> = message_len
            .to_le_bytes()
            .into_iter()
            .chain(encoded_with_nonce)
            .collect();

        self.socket.write_all(&final_message).await?;
        Ok(())
    }

    async fn receive_message(&mut self) -> Result<Message, TransportError> {
        let mut message_len: Option<usize> = None;

        loop {
            let mut buf = [0; BUFFER_LEN];
            let bytes_read = self.socket.read(&mut buf).await?;

            if bytes_read == 0 {
                return Err(TransportError::ConnectionClosed);
            }

            self.curr.extend_from_slice(&buf[0..bytes_read]);

            if message_len.is_none() && self.curr.len() >= HEADER_LEN {
                message_len = Some(self.extract_msg_len()?);
                self.curr.drain(..HEADER_LEN);
            }

            if let Some(len) = message_len {
                if self.curr.len() >= len {
                    // TODO: fix errors with short circuiting
                    let message = decrypt_and_deserialise_message(&self.curr[..len], &self.key)?;
                    self.curr.drain(..len);
                    return Ok(message);
                }
            }
        }
    }
}

pub struct TokioTcpTransportWriter<T: Crypto> {
    socket: OwnedWriteHalf,
    key: Option<T>,
}

impl<T: Crypto> TokioTcpTransportWriter<T> {
    pub fn new(socket: OwnedWriteHalf, key: Option<T>) -> Self {
        TokioTcpTransportWriter { socket, key }
    }
}

impl<T: Crypto> TransportWriter for TokioTcpTransportWriter<T> {
    async fn send_message(&mut self, message: Message) -> Result<(), TransportError> {
        let encoded_message: Vec<u8> = bincode::serialize(&message)?;

        let (encrypted, nonce) = if let Some(encryptor) = &self.key {
            encryptor.encrypt(encoded_message)?
        } else {
            (encoded_message, Nonce::default())
        };

        let message_with_nonce = MessageWithNonce::new(encrypted, nonce);
        let encoded_with_nonce: Vec<u8> = bincode::serialize(&message_with_nonce)?;

        let message_len = encoded_with_nonce.len() as u32;
        let final_message: Vec<u8> = message_len
            .to_le_bytes()
            .into_iter()
            .chain(encoded_with_nonce)
            .collect();

        self.socket.write_all(&final_message).await?;
        Ok(())
    }
}

pub struct TokioTcpTransportReader<T: Crypto> {
    socket: OwnedReadHalf,
    key: Option<T>,
    curr: Vec<u8>,
}

impl<T: Crypto> TokioTcpTransportReader<T> {
    pub fn new(socket: OwnedReadHalf, key: Option<T>, curr: Vec<u8>) -> Self {
        TokioTcpTransportReader { socket, key, curr }
    }

    fn extract_msg_len(&self) -> Result<usize, TransportError> {
        let prefix_bytes: [u8; HEADER_LEN] = self.curr[..HEADER_LEN]
            .try_into()
            .map_err(|_| TransportError::InvalidMessageStructure)?;
        u32::from_le_bytes(prefix_bytes)
            .try_into()
            .map_err(|_| TransportError::ByteArrayConversionError)
    }
}

impl<T: Crypto> TransportReader for TokioTcpTransportReader<T> {
    async fn receive_message(&mut self) -> Result<Message, TransportError> {
        let mut message_len: Option<usize> = None;

        loop {
            let mut buf = [0; BUFFER_LEN];
            let bytes_read = self.socket.read(&mut buf).await?;

            if bytes_read == 0 {
                return Err(TransportError::ConnectionClosed);
            }

            self.curr.extend_from_slice(&buf[0..bytes_read]);

            if message_len.is_none() && self.curr.len() >= HEADER_LEN {
                message_len = Some(self.extract_msg_len()?);
                self.curr.drain(..HEADER_LEN);
            }

            if let Some(len) = message_len {
                if self.curr.len() >= len {
                    // TODO: fix errors with short circuiting
                    let message = decrypt_and_deserialise_message(&self.curr[..len], &self.key)?;
                    self.curr.drain(..len);
                    return Ok(message);
                }
            }
        }
    }
}
