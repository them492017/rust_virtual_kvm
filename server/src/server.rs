use std::{net::SocketAddr, sync::Arc};

use chacha20poly1305::ChaCha20Poly1305;
use common::error::DynError;
use tokio::{net::TcpListener, sync::RwLock};

use crate::{handlers::client::handle_client, state::State};

pub async fn start_listening(
    server_address: SocketAddr,
    state: Arc<RwLock<State<ChaCha20Poly1305>>>,
) -> Result<(), DynError> {
    let tcp_listener = TcpListener::bind(server_address).await?;

    loop {
        let (socket, _) = tcp_listener.accept().await?;
        let state_clone = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(err) = handle_client(socket, state_clone).await {
                println!("Error handling client: {}", err);
            }
        });
    }
}
