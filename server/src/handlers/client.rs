use std::time::Duration;

use chacha20poly1305::ChaCha20Poly1305;
use common::{
    error::DynError,
    net::Message,
    tcp::{TokioTcpTransport, TokioTcpTransportReader, TokioTcpTransportWriter},
    transport::{AsyncTransportReader, AsyncTransportWriter},
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
};
use uuid::Uuid;

use crate::{
    client::{Client, Connection},
    handlers::client_message_sender::ClientMessageSender,
    processor::InternalMessage,
    server_message::ServerMessage,
    CHANNEL_BUF_LEN,
};

// TODO: refactor to a common location
const HEARTBEAT_INTERVAL: u64 = 3;
const NUM_RETRIES: u64 = 3;

pub async fn handle_client(
    stream: TcpStream,
    client_sender: Sender<Client<ChaCha20Poly1305>>,
    client_message_sender: Sender<InternalMessage>,
) -> Result<(), DynError> {
    let mut transport = TokioTcpTransport::new(stream);
    let (message_sender, message_receiver) = mpsc::channel(CHANNEL_BUF_LEN);
    let client: Client<ChaCha20Poly1305> = Client::connect(&mut transport, message_sender).await?;

    // send client to event processor
    let id = client.id;
    client_sender.send(client).await?;

    process_events(id, transport, message_receiver, client_message_sender).await
}

async fn process_events(
    id: Uuid,
    transport: TokioTcpTransport<ChaCha20Poly1305>,
    message_receiver: Receiver<Message>,
    client_message_sender: Sender<InternalMessage>,
) -> Result<(), DynError> {
    let (reader_transport, writer_transport) = transport.into_split();

    let client_message_sender = ClientMessageSender::new(id, client_message_sender);
    let client_message_sender_clone = client_message_sender.clone();
    let listener =
        tokio::spawn(
            async move { tcp_listener(reader_transport, client_message_sender_clone).await },
        );
    let sender = tokio::spawn(async move {
        tcp_sender(
            id,
            writer_transport,
            message_receiver,
            client_message_sender,
        )
        .await
    });

    tokio::select! {
        result = listener => {
            return result?
        },
        result = sender => {
            return result?
        },
    }
}

async fn tcp_listener(
    mut listener: TokioTcpTransportReader<ChaCha20Poly1305>,
    client_message_sender: ClientMessageSender,
) -> Result<(), DynError> {
    loop {
        let message = listener.receive_message().await?;
        println!("Received from TCP: {:?}", message);
        client_message_sender.send_client_message(message).await?;
    }
}

async fn tcp_sender(
    id: Uuid,
    mut sender: TokioTcpTransportWriter<ChaCha20Poly1305>,
    mut message_receiver: Receiver<Message>,
    client_message_sender: ClientMessageSender,
) -> Result<(), DynError> {
    let duration = Duration::from_secs(HEARTBEAT_INTERVAL);
    let mut fail_count = 0;
    let num_retries = NUM_RETRIES;

    loop {
        tokio::select! {
            Some(message) = message_receiver.recv() => {
                println!("Received message from channel");
                sender.send_message(message).await?;
                // TODO: figure out what to do about ? here...
                // defeats the purpose of the failure count
                fail_count = 0;
            },
            _ = tokio::time::sleep(duration) => {
                println!("Sending heartbeat");
                if let Err(err) = sender.send_message(Message::Heartbeat).await {
                    fail_count += 1;
                    println!(
                        "Failed hearbeats {}/{} for client: {}",
                        fail_count, num_retries, err
                    );

                    if fail_count >= num_retries {
                        let message = ServerMessage::ClientDisconnect { id };
                        client_message_sender.send_server_message(message).await?;
                        return Err("Heartbeat failed".into());
                    }
                } else {
                    fail_count = 0;
                }
            }
        }
    }
}
