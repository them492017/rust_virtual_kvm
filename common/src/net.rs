use std::{fmt, net::SocketAddr};

use chacha20poly1305::Nonce;
use evdev::{EventType, InputEvent};
use serde::{Deserialize, Serialize};
use x25519_dalek::PublicKey;

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

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    InputEvent { event: SerializableInputEvent },
    TargetChangeNotification,
    TargetChangeResponse,
    ClipboardChanged { content: String }, // TODO: content could be an image
    ClientInit { addr: SocketAddr },
    ExchangePubKey { pub_key: PublicKey },
    Ack,
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
            Message::Ack => write!(f, "Ack"),
            Message::Handshake => write!(f, "Handshake"),
            Message::Heartbeat => write!(f, "Heartbeat"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableInputEvent {
    type_: EventType,
    code: u16,
    value: i32,
}

impl From<InputEvent> for SerializableInputEvent {
    fn from(value: InputEvent) -> Self {
        SerializableInputEvent {
            type_: value.event_type(),
            code: value.code(),
            value: value.value(),
        }
    }
}

impl From<SerializableInputEvent> for InputEvent {
    fn from(val: SerializableInputEvent) -> Self {
        InputEvent::new(val.type_, val.code, val.value)
    }
}
