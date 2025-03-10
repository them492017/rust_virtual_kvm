use std::time::Duration;

use chacha20poly1305::ChaCha20Poly1305;
use evdev::uinput::VirtualDevice;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::common::{
    dev::{make_keyboard, release_all},
    error::DynError,
    net::Message,
    tcp::{TokioTcpTransport, TokioTcpTransportReader, TokioTcpTransportWriter},
    transport::{AsyncTransportReader, AsyncTransportWriter},
};

pub async fn special_event_processor(
    transport: TokioTcpTransport<ChaCha20Poly1305>,
    cancellation_token: CancellationToken,
) -> Result<(), DynError> {
    println!("Started special event listener");
    let virtual_keyboard = make_keyboard().expect("Could not create virtual keyboard");

    let (read_transport, write_transport) = transport.into_split();
    let (tx, rx) = mpsc::channel(8);

    let cloned_token = cancellation_token.clone();
    let listener = tokio::spawn(async move {
        special_event_listener(read_transport, tx, virtual_keyboard, cloned_token).await
    });
    let sender =
        tokio::spawn(
            async move { special_event_sender(write_transport, rx, cancellation_token).await },
        );

    tokio::select! {
        result = listener => {
            result??
        },
        result = sender => {
            result??
        },
    }
    Ok(())
}

pub async fn special_event_listener(
    mut reader: TokioTcpTransportReader<ChaCha20Poly1305>,
    sender: mpsc::Sender<Message>,
    mut virtual_keyboard: VirtualDevice,
    cancellation_token: CancellationToken,
) -> Result<(), DynError> {
    loop {
        println!("Listening for message");
        tokio::select! {
            message = reader.receive_message() => {
                if let Ok(event) = message {
                        println!("{:?}", event);
                        match event {
                            Message::InputEvent { event } => {
                                println!("{:?}", event);
                            }
                            Message::ClipboardChanged { content } => {
                                println!("New clipboard item: [{:?}]", content);
                                sender.send(Message::ExchangePubKeyResponse).await?; // TODO: temporary response
                            }
                            Message::TargetChangeNotification => {
                                release_all(&mut virtual_keyboard)?;
                                sender.send(Message::TargetChangeResponse).await?;
                            }
                            Message::Heartbeat => {
                                println!("Received heartbeat");
                            }
                            _ => {
                                unimplemented!("Received unimplemented event")
                            }
                        }
                } else {
                    eprintln!(
                        "An error has occured when listening to TCP messages: {:?}",
                        message.as_ref().err()
                    );
                    return Err(message.err().unwrap())
                }
            },
            _ = cancellation_token.cancelled() => {
                return Ok(())
            },
        }
    }
}

pub async fn special_event_sender(
    mut writer: TokioTcpTransportWriter<ChaCha20Poly1305>,
    mut receiver: mpsc::Receiver<Message>,
    cancellation_token: CancellationToken,
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
            },
            _ = cancellation_token.cancelled() => {
                return Ok(())
            },
        }
    }
}
