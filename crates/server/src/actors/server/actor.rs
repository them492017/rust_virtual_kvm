use chacha20poly1305::ChaCha20Poly1305;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::{client::Client, handlers::client::handle_client, InternalMessage};

use super::resource::ServerResource;

// TODO: rename and move to actor module
impl ServerResource {
    pub async fn start_listening(
        self,
        client_sender: Sender<Client<ChaCha20Poly1305>>,
        client_message_sender: Sender<InternalMessage>,
        cancellation_token: CancellationToken,
    ) -> Result<(), std::io::Error> {
        loop {
            let (socket, addr) = self.listener.accept().await?;
            println!("Received incoming connection on {}", addr);

            let client_sender_clone = client_sender.clone();
            let client_message_sender_clone = client_message_sender.clone();
            let cancellation_token_clone = cancellation_token.clone();

            tokio::spawn(async move {
                // TODO: move this handling function into this file
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
}
