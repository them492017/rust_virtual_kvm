use std::{fmt, net::SocketAddr};

use ::input_event::InputEvent;
use chacha20poly1305::Nonce;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use x25519_dalek::PublicKey;

pub mod input_event;
pub mod tcp;
pub mod transport;
pub mod udp;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Serialization error")]
    SerializationError(#[from] bincode::Error),
    #[error("Encryption error")]
    EncryptionError(#[from] crypto::EncryptionError),
    #[error("Invalid message structure with no message length")]
    InvalidMessageStructure,
    #[error("Error when converting message length byte array to integer")]
    ByteArrayConversionError,
    #[error("IO error")]
    IOError(#[from] std::io::Error),
    #[error("Decryption error")]
    ConnectionClosed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageWithNonce {
    pub message: Vec<u8>,
    pub nonce: [u8; 12],
}

impl MessageWithNonce {
    pub fn new(message: Vec<u8>, nonce: Nonce) -> Self {
        MessageWithNonce {
            message,
            nonce: nonce.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Message {
    InputEvent { event: InputEvent },
    TargetChangeNotification,
    TargetChangeResponse,
    ClipboardChanged { content: String }, // TODO: content could be an image
    ClientInit { addr: SocketAddr },
    ExchangePubKey { pub_key: PublicKey },
    ExchangePubKeyResponse,
    Handshake,
    Heartbeat,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::InputEvent { event } => {
                write!(f, "InputEvent: event = {:?}", event)
            }
            Message::TargetChangeNotification => write!(f, "TargetChangeNotification"),
            Message::TargetChangeResponse => write!(f, "TargetChangeResponse"),
            Message::ClipboardChanged { content } => {
                write!(f, "ClipboardChanged: content = {}", content)
            }
            Message::ClientInit { addr } => write!(f, "ClientInit: addr = {}", addr),
            Message::ExchangePubKey { pub_key } => {
                write!(f, "ExchangePubKey: pub_key = {:?}", pub_key)
            }
            Message::ExchangePubKeyResponse => write!(f, "Ack"),
            Message::Handshake => write!(f, "Handshake"),
            Message::Heartbeat => write!(f, "Heartbeat"),
        }
    }
}
