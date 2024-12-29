use evdev::InputEvent;
use serde::{Deserialize, Serialize};
use std::{error::Error, net::UdpSocket};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::tcp::{OwnedReadHalf, OwnedWriteHalf}, sync::mpsc};

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
    EdgeReached { side: Direction },
    ClipboardChanged { content: String }, // TODO: content could be an image
    ChangedActiveIndex { idx: usize },
}

pub async fn send_message(writer: &mut OwnedWriteHalf, message: Message) -> bincode::Result<()> {
    let encoded: Vec<u8> = bincode::serialize(&message)?;
    writer.write_all(&encoded).await?;
    Ok(())
}

pub async fn message_producer(
    reader: &mut OwnedReadHalf,
    channel: mpsc::Sender<Message>,
) -> Result<(), Box<dyn Error>> {
    let mut curr: Vec<u8> = Vec::new();
    let mut message_len: Option<usize> = None;

    loop {
        read_tcp_socket(reader, &mut curr).await?;

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

async fn read_tcp_socket(reader: &mut OwnedReadHalf, curr: &mut Vec<u8>) -> Result<(), Box<dyn Error>> {
    let mut buf = [0; 256];
    reader.read(&mut buf).await?;
    curr.append(&mut buf.to_vec());
    Ok(())
}

fn process_msg_len(curr: &[u8]) -> Result<usize, Box<dyn Error>> {
    let prefix_bytes: [u8; 4] = curr[..4].try_into()?;
    Ok(u32::from_le_bytes(prefix_bytes).try_into()?)
}

pub fn send_event(socket: &UdpSocket, addr: &str, event: &InputEvent) -> bincode::Result<()> {
    let serialised_event: SerializableInputEvent = event.into();
    let encoded: Vec<u8> = bincode::serialize(&serialised_event)?;
    socket.send_to(&encoded, addr)?;
    Ok(())
}

pub fn recv_event(socket: &UdpSocket) -> bincode::Result<InputEvent> {
    let mut buf = [0; 128];
    socket.recv(&mut buf).unwrap();

    let serialised_event: SerializableInputEvent = bincode::deserialize(&buf)?;

    Ok(serialised_event.into())
}
