use std::time::Duration;

use chacha20poly1305::ChaCha20Poly1305;
use common::{
    error::DynError,
    net::Message,
    tcp2::{TokioTcpTransport, TokioTcpTransportReader, TokioTcpTransportWriter},
    transport::{AsyncTransportReader, AsyncTransportWriter},
};
use tokio::sync::mpsc;

pub async fn special_event_processor(
    transport: TokioTcpTransport<ChaCha20Poly1305>,
) -> Result<(), DynError> {
    println!("Started special event listener");
    let (read_transport, write_transport) = transport.into_split();
    let (tx, rx) = mpsc::channel(8);

    let listener = tokio::spawn(async move { special_event_listener(read_transport, tx).await });
    let sender = tokio::spawn(async move { special_event_sender(write_transport, rx).await });

    tokio::select! {
        result = listener => {
            return result?
        },
        result = sender => {
            return result?
        },
    }
}

pub async fn special_event_listener(
    mut reader: TokioTcpTransportReader<ChaCha20Poly1305>,
    sender: mpsc::Sender<Message>,
) -> Result<(), DynError> {
    loop {
        println!("Listening for message");
        match reader.receive_message().await {
            Ok(event) => {
                println!("{:?}", event);
                match event {
                    Message::InputEvent { event } => {
                        println!("{:?}", event);
                    }
                    Message::ClipboardChanged { content } => {
                        println!("New clipboard item: [{:?}]", content);
                        sender.send(Message::Ack).await?; // TODO: temporary
                    }
                    _ => {}
                }
            }
            Err(err) => {
                println!(
                    "An error has occured when listening to TCP messages: {}",
                    err
                );
            }
        }
    }
}

pub async fn special_event_sender(
    mut writer: TokioTcpTransportWriter<ChaCha20Poly1305>,
    mut receiver: mpsc::Receiver<Message>,
) -> Result<(), DynError> {
    let timeout = Duration::from_secs(3);
    loop {
        tokio::select! {
            Some(message) = receiver.recv() => {
                println!("Received message from channel");
                writer.send_message(message).await?;
            },
            _ = tokio::time::sleep(timeout) => {
                println!("Sending heartbeat");
                writer.send_message(Message::Heartbeat).await?;
            }
        }
    }
}
