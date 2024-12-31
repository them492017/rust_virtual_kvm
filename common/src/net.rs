use chacha20poly1305::{aead::{Aead, AeadMutInPlace}, ChaCha20Poly1305, Nonce};
use evdev::InputEvent;
use serde::{Deserialize, Serialize};
use std::{error::Error, io::{self, Read}, net::UdpSocket};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::tcp::{OwnedReadHalf, OwnedWriteHalf}, sync::mpsc};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::SerializableInputEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Swap { new_target: usize },
    ClipboardChanged { content: String }, // TODO: content could be an image
    ChangedActiveIndex { idx: usize },
    ClientInit { addr: String },
    ExchangePubKey { pub_key: PublicKey },
    // Ack { encrypted_msg: String },
}

pub async fn send_message(writer: &mut OwnedWriteHalf, message: Message) -> bincode::Result<()> {
    let encoded: Vec<u8> = bincode::serialize(&message)?;
    writer.write_all(&encoded).await?;
    Ok(())
}

pub async fn message_producer(
    mut reader: OwnedReadHalf,
    channel: mpsc::Sender<Message>,
) -> Result<(), Box<dyn Error>> {
    let mut curr: Vec<u8> = Vec::new();
    let mut message_len: Option<usize> = None;

    loop {
        let mut buf = [0; 256];
        read_tcp_socket(&mut reader, &mut buf).await?;
        curr.append(&mut buf.to_vec());

        if message_len.is_none() && curr.len() >= 4 {
            message_len = Some(process_msg_len(&curr)?);
            curr.drain(..4);
        }

        if let Some(len) = message_len {
            if curr.len() >= len
                .try_into()
                .expect("Couldn't convert message_len to usize")
            {
                if let Ok(message) = bincode::deserialize::<Message>(&curr[..len]) {
                    channel.send(message).await?;
                    // Reset buffer
                    curr.drain(..len);
                    message_len = None;
                }
            }
        }
    }
}

pub async fn read_tcp_socket(reader: &mut OwnedReadHalf, buf: &mut [u8]) -> Result<(), Box<dyn Error>> {
    reader.read(buf).await?;
    Ok(())
}

fn process_msg_len(curr: &[u8]) -> Result<usize, Box<dyn Error>> {
    let prefix_bytes: [u8; 4] = curr[..4].try_into()?;
    Ok(u32::from_le_bytes(prefix_bytes).try_into()?)
}

pub fn send_event(socket: &UdpSocket, addr: &str, event: &crate::event::InputEvent) -> bincode::Result<()> {
    let encoded: Vec<u8> = bincode::serialize(&event)?;
    socket.send_to(&encoded, addr)?;
    Ok(())
}

pub fn recv_event(socket: &UdpSocket, cipher: &mut ChaCha20Poly1305) -> bincode::Result<InputEvent> {
    let mut buf = [0; 128];
    socket.recv(&mut buf).unwrap();

    let deserialised_message: crate::event::InputEvent = bincode::deserialize(&buf)?;

    let nonce = Nonce::from_slice(&deserialised_message.nonce);
    let decrypted_event = cipher.decrypt(&nonce, deserialised_message.encrypted_event.as_slice()).map_err(|_| {
        bincode::Error::new(bincode::ErrorKind::Io(std::io::Error::new(io::ErrorKind::Other, "Decrypt failed")))
    })?;

    let deserialised_event: SerializableInputEvent = bincode::deserialize(decrypted_event.as_slice())?;

    Ok(deserialised_event.into())
}
