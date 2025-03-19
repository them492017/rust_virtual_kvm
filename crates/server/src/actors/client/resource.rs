use chacha20poly1305::ChaCha20Poly1305;
use crypto::Crypto;
use network::{
    tcp::{TokioTcpTransport, TokioTcpTransportReader, TokioTcpTransportWriter},
    Message,
};
use thiserror::Error;
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, error::SendError, Receiver, Sender},
};
use uuid::Uuid;

use crate::{
    actors::state::client::{Client, ClientConnectionError, Connection},
    InternalMessage,
};

const CHANNEL_BUF_LEN: usize = 256;

#[derive(Debug, Error)]
pub enum ConnectionResourceError {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] ClientConnectionError),
    #[error("Could not send client to state actor: {0}")]
    ClientSendError(#[from] SendError<Client<ChaCha20Poly1305>>),
}

pub struct ConnectionResource<T: Crypto> {
    pub id: Uuid,
    pub transport_writer: TokioTcpTransportWriter<T>,
    pub transport_reader: TokioTcpTransportReader<T>,
    pub message_receiver: Receiver<Message>,
    pub client_message_sender: Sender<InternalMessage>,
}

impl ConnectionResource<ChaCha20Poly1305> {
    pub async fn new(
        stream: TcpStream,
        client_sender: Sender<Client<ChaCha20Poly1305>>,
        client_message_sender: Sender<InternalMessage>,
    ) -> Result<Self, ConnectionResourceError> {
        // TODO: fix error type
        let mut transport = TokioTcpTransport::new(stream);
        let (message_sender, message_receiver) = mpsc::channel(CHANNEL_BUF_LEN);
        let client: Client<ChaCha20Poly1305> =
            Client::connect(&mut transport, message_sender).await?;

        // send client to event processor
        let id = client.id;
        client_sender.send(client).await?;

        let (transport_reader, transport_writer) = transport.into_split();

        Ok(ConnectionResource {
            id,
            transport_writer,
            transport_reader,
            message_receiver,
            client_message_sender,
        })
    }
}
