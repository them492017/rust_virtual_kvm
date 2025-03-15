use std::net::SocketAddr;

use chacha20poly1305::ChaCha20Poly1305;
use tokio::{net::TcpListener, sync::mpsc::Sender};
use tokio_util::sync::CancellationToken;

use crate::InternalMessage;

use super::{client::Client, handlers::client::handle_client};

// TODO: rename and move to actor module
pub async fn start_listening(
    server_address: SocketAddr,
    client_sender: Sender<Client<ChaCha20Poly1305>>,
    client_message_sender: Sender<InternalMessage>,
    cancellation_token: CancellationToken,
) -> Result<(), std::io::Error> {
    let tcp_listener = TcpListener::bind(server_address).await?;
    println!("Bound TCP listener to {}", server_address);

    loop {
        let (socket, _) = tcp_listener.accept().await?;
        let client_sender_clone = client_sender.clone();
        let client_message_sender_clone = client_message_sender.clone();
        let cancellation_token_clone = cancellation_token.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_client(
                socket,
                client_sender_clone,
                client_message_sender_clone,
                cancellation_token_clone,
            )
            .await
            {
                eprintln!("Error handling client: {}", err);
            }
        });
    }
}
