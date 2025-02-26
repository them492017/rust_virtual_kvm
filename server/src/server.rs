use std::net::SocketAddr;

use chacha20poly1305::ChaCha20Poly1305;
use common::error::DynError;
use tokio::{net::TcpListener, sync::mpsc::Sender};

use crate::{client::Client, handlers::client::handle_client, processor::InternalMessage};

pub async fn start_listening(
    server_address: SocketAddr,
    client_sender: Sender<Client<ChaCha20Poly1305>>,
    client_message_sender: Sender<InternalMessage>,
) -> Result<(), DynError> {
    let tcp_listener = TcpListener::bind(server_address).await?;

    loop {
        let (socket, _) = tcp_listener.accept().await?;
        let client_sender_clone = client_sender.clone();
        let client_message_sender_clone = client_message_sender.clone();
        tokio::spawn(async move {
            if let Err(err) =
                handle_client(socket, client_sender_clone, client_message_sender_clone).await
            {
                println!("Error handling client: {}", err);
            }
        });
    }
}
