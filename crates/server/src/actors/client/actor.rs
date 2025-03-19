use std::time::Duration;

use chacha20poly1305::ChaCha20Poly1305;
use network::{
    tcp::{TokioTcpTransportReader, TokioTcpTransportWriter},
    transport::{TransportReader, TransportWriter},
    Message, TransportError,
};
use thiserror::Error;
use tokio::{
    sync::mpsc::{error::SendError, Receiver},
    task::JoinError,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{actors::state::client::Client, InternalMessage, ServerMessage};

use super::{
    client_message_sender::ClientMessageSender,
    resource::{ConnectionResource, ConnectionResourceError},
};

// TODO: refactor to a common location
const HEARTBEAT_INTERVAL: u64 = 3;
const MAX_RETRIES: u64 = 3;

#[derive(Debug, Error)]
pub enum ClientHandlerError {
    #[error("Could not send new client through channel: {0}")]
    ClientSendError(#[from] SendError<Client<ChaCha20Poly1305>>),
    #[error("Could not connect to client: {0}")]
    ClientConnectionError(#[from] ConnectionResourceError),
    #[error("Transport error: {0}")]
    TransportError(#[from] TransportError),
    #[error("Could not send internal message through channel: {0}")]
    InternalMessageSendError(#[from] SendError<InternalMessage>),
    #[error("Could not send heartbeat")]
    HeartbeatFail,
    #[error("A subtask panicked: {0}")]
    SubTaskPanickedError(#[from] JoinError),
}

impl ConnectionResource<ChaCha20Poly1305> {
    pub async fn process_events(
        self,
        cancellation_token: CancellationToken,
    ) -> Result<(), ClientHandlerError> {
        let client_message_sender = ClientMessageSender::new(self.id, self.client_message_sender);
        let client_message_sender_clone = client_message_sender.clone();
        let listener = tokio::spawn(async move {
            tcp_listener(self.transport_reader, client_message_sender_clone).await
        });
        let sender = tokio::spawn(async move {
            tcp_sender(
                self.id,
                self.transport_writer,
                self.message_receiver,
                client_message_sender,
            )
            .await
        });

        tokio::select! {
            result = listener => {
                result?
            },
            result = sender => {
                result?
            },
            _ = cancellation_token.cancelled() => {
                Ok(())
            },
        }
    }
}

async fn tcp_listener(
    mut listener: TokioTcpTransportReader<ChaCha20Poly1305>,
    client_message_sender: ClientMessageSender,
) -> Result<(), ClientHandlerError> {
    loop {
        let message = listener.receive_message().await?;
        client_message_sender.send_client_message(message).await?;
    }
}

async fn tcp_sender(
    id: Uuid,
    mut sender: TokioTcpTransportWriter<ChaCha20Poly1305>,
    mut message_receiver: Receiver<Message>,
    client_message_sender: ClientMessageSender,
) -> Result<(), ClientHandlerError> {
    let duration = Duration::from_secs(HEARTBEAT_INTERVAL);
    let mut fail_count = 0;

    loop {
        tokio::select! {
            Some(message) = message_receiver.recv() => {
                handle_send_result(
                    sender.send_message(message).await,
                    &mut fail_count,
                    id,
                    &client_message_sender,
                ).await?;
            },
            _ = tokio::time::sleep(duration) => {
                handle_send_result(
                    sender.send_message(Message::Heartbeat).await,
                    &mut fail_count,
                    id,
                    &client_message_sender,
                ).await?;
            }
        }
    }
}

async fn handle_send_result(
    result: Result<(), TransportError>,
    fail_count: &mut u64,
    id: Uuid,
    client_message_sender: &ClientMessageSender,
) -> Result<(), ClientHandlerError> {
    if let Err(err) = result {
        *fail_count += 1;
        eprintln!(
            "Failed hearbeats {}/{} for client: {}",
            fail_count, MAX_RETRIES, err
        );

        if *fail_count >= MAX_RETRIES {
            let message = ServerMessage::ClientDisconnect { id };
            client_message_sender.send_server_message(message).await?;
            return Err(ClientHandlerError::HeartbeatFail);
        }
    } else {
        *fail_count = 0;
    }

    Ok(())
}
