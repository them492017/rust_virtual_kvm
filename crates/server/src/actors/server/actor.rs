use chacha20poly1305::ChaCha20Poly1305;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::{
    actors::{
        client::{actor::ClientHandlerError, resource::ConnectionResource},
        state::client::Client,
    },
    InternalMessage,
};

use super::resource::ServerResource;

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
            let cancellation_token_clone1 = cancellation_token.clone();

            tokio::spawn(async move {
                let result: Result<(), ClientHandlerError> = async {
                    let connection = ConnectionResource::new(
                        socket,
                        client_sender_clone,
                        client_message_sender_clone,
                    )
                    .await?;
                    connection.process_events(cancellation_token_clone1).await?;
                    Ok(())
                }
                .await;

                if let Err(err) = result {
                    eprintln!("Error processing client events: {}", err);
                    // Server should ignore the processing error
                }
            });
        }
    }
}
