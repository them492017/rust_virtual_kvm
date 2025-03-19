use std::time::Duration;

use chacha20poly1305::ChaCha20Poly1305;
use input_simulator::DeviceOutputError;
use network::{
    tcp::{TokioTcpTransport, TokioTcpTransportReader, TokioTcpTransportWriter},
    transport::{TransportReader, TransportWriter},
    Message, TransportError,
};
use thiserror::Error;
use tokio::{
    sync::mpsc::{self, error::SendError, Sender},
    task::JoinError,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Error)]
pub enum SpecialEventProcessorError {
    // TODO: check a join error implies a panic occured
    #[error("Special event processor subtask panicked")]
    SubTaskPanicError(#[from] JoinError),
    #[error("Device output error")]
    DeviceOutputError(#[from] DeviceOutputError),
    #[error("Could not send special event")]
    MessageSendError(#[from] SendError<Message>),
    #[error("Could not send release request")]
    ReleaseRequesetSendError(#[from] SendError<()>),
    #[error("Transport error")]
    TransportError(#[from] TransportError),
}

pub async fn special_event_processor(
    transport: TokioTcpTransport<ChaCha20Poly1305>,
    release_request_sender: Sender<()>,
    cancellation_token: CancellationToken,
) -> Result<(), SpecialEventProcessorError> {
    let (read_transport, write_transport) = transport.into_split();
    let (message_sender, message_receiver) = mpsc::channel(8);

    let cloned_token = cancellation_token.clone();
    let listener = tokio::spawn(async move {
        special_event_listener(
            read_transport,
            message_sender,
            release_request_sender,
            cloned_token,
        )
        .await
    });
    let sender = tokio::spawn(async move {
        special_event_sender(write_transport, message_receiver, cancellation_token).await
    });

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
    message_sender: mpsc::Sender<Message>,
    release_request_sender: Sender<()>,
    cancellation_token: CancellationToken,
) -> Result<(), SpecialEventProcessorError> {
    loop {
        tokio::select! {
            message = reader.receive_message() => {
                if let Ok(event) = message {
                        match event {
                            Message::ClipboardChanged { content } => {
                                println!("New clipboard item: [{:?}]", content);
                                message_sender.send(Message::ExchangePubKeyResponse).await?; // TODO: temporary response
                            }
                            Message::TargetChangeNotification => {
                                println!("Releasing all keys");
                                release_request_sender.send(()).await?;
                                message_sender.send(Message::TargetChangeResponse).await?;
                            }
                            Message::Heartbeat => {}
                            _ => {
                                unimplemented!("Received unimplemented special event")
                            }
                        }
                } else {
                    eprintln!(
                        "An error has occured when listening to TCP messages: {:?}",
                        message.as_ref().err()
                    );
                    return Err(message.err().unwrap().into())
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
    mut message_receiver: mpsc::Receiver<Message>,
    cancellation_token: CancellationToken,
) -> Result<(), SpecialEventProcessorError> {
    let timeout = Duration::from_secs(3);
    loop {
        tokio::select! {
            Some(message) = message_receiver.recv() => {
                writer.send_message(message).await?;
            },
            _ = tokio::time::sleep(timeout) => {
                writer.send_message(Message::Heartbeat).await?;
            },
            _ = cancellation_token.cancelled() => {
                return Ok(())
            },
        }
    }
}
