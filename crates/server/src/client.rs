use std::collections::VecDeque;
use std::net::SocketAddr;

use chacha20poly1305::{aead::OsRng, ChaCha20Poly1305, KeyInit};
use crypto::Crypto;
use network::{
    input_event::InputEventTransport, tcp::TokioTcpTransport, transport::Transport, Message,
    TransportError,
};
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;
use x25519_dalek::{EphemeralSecret, PublicKey};

const RING_BUFFER_LEN: usize = 1024;

#[derive(Debug, Error)]
pub enum ClientConnectionError {
    #[error("Client not ready to receive messages")]
    NotReady,
    #[error("Transport error during connection")]
    TransportError(#[from] TransportError),
    #[error("Invalid message received during connection")]
    InvalidMessageError,
    #[error("Shared Diffe-Hellman secret was not contributory")]
    DHContributionError,
}

pub trait Connection<T: Crypto>: Sized {
    fn connect(
        transport: &mut TokioTcpTransport<T>,
        message_sender: Sender<Message>,
    ) -> impl std::future::Future<Output = Result<Self, ClientConnectionError>> + Send + Sync;
}

#[derive(Debug)]
pub struct Client<T: Crypto> {
    pub id: Uuid,
    pub connected: bool,
    pub address: SocketAddr,
    pub key: T,
    pub message_sender: Sender<Message>,
    pub pending_target_change_responses: u32,
    pending_messages: VecDeque<Message>,
}

// TODO: extract connection logic into another crate
impl Connection<ChaCha20Poly1305> for Client<ChaCha20Poly1305> {
    async fn connect(
        transport: &mut TokioTcpTransport<ChaCha20Poly1305>,
        message_sender: Sender<Message>,
    ) -> Result<Self, ClientConnectionError> {
        println!("Initialising client");

        let addr = match transport.receive_message().await {
            Ok(Message::ClientInit { addr }) => {
                println!("Received addr: {}", addr);
                addr
            }
            Ok(message) => {
                println!("Received message: {}", message);
                return Err(ClientConnectionError::InvalidMessageError);
            }
            Err(err) => {
                println!("Did not receive init message");
                return Err(err.into());
            }
        };

        // generate pub key
        // TODO: should sign this
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let pub_key = PublicKey::from(&secret);

        transport
            .send_message(Message::ExchangePubKey { pub_key })
            .await?;
        println!("Sent pub key to client");

        let client_pub_key = match transport.receive_message().await {
            Ok(Message::ExchangePubKey { pub_key }) => {
                println!("Received public key from client");
                pub_key
            }
            Ok(message) => {
                println!("Received message: {}", message);
                return Err(ClientConnectionError::InvalidMessageError);
            }
            Err(err) => {
                println!("Did not receive pub key message");
                return Err(err.into());
            }
        };

        transport
            .send_message(Message::ExchangePubKeyResponse)
            .await?;
        println!("Sent ack to client");

        let cipher = {
            let shared_secret = secret.diffie_hellman(&client_pub_key);
            if !shared_secret.was_contributory() {
                return Err(ClientConnectionError::DHContributionError);
            }

            Ok::<_, ClientConnectionError>(
                ChaCha20Poly1305::new_from_slice(&shared_secret.to_bytes())
                    .expect("Could not generate cipher from shared secret"),
            )
        }?;

        transport.set_key(cipher.clone());

        // send handshake with encryption enabled
        transport.send_message(Message::Handshake).await?;

        println!("Waiting for client handshake");
        if let Message::Handshake = transport.receive_message().await? {
            println!("Received handshake from client");
        } else {
            return Err(ClientConnectionError::InvalidMessageError);
        };

        println!("Successfully connected to client at address {:?}", addr);

        Ok(Client {
            id: Uuid::new_v4(),
            connected: true,
            key: cipher,
            address: addr,
            message_sender,
            pending_target_change_responses: 0,
            pending_messages: VecDeque::with_capacity(RING_BUFFER_LEN),
        })
    }
}

impl<T: Crypto> Client<T> {
    // TODO: update to send batches of messages
    pub async fn flush_pending_messages(
        &mut self,
        transport: &mut InputEventTransport,
    ) -> Result<(), ClientConnectionError> {
        if !self.can_receive() {
            return Err(ClientConnectionError::NotReady);
        }
        while let Some(message) = self.pending_messages.pop_front() {
            transport
                .send_message_to(message, self.address, Some(self.key.clone()))
                .await?;
        }
        Ok(())
    }

    pub fn buffer_message(&mut self, message: Message) {
        self.pending_messages.push_back(message);
    }

    pub fn can_receive(&self) -> bool {
        self.pending_target_change_responses == 0 && self.pending_messages.is_empty()
    }
}

#[cfg(test)]
pub mod test {
    use network::Message;
    use tokio::sync::mpsc;

    use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
    use uuid::Uuid;

    use super::Client;

    pub fn test_client_fixture(message_sender: mpsc::Sender<Message>) -> Client<ChaCha20Poly1305> {
        Client {
            id: Uuid::new_v4(),
            connected: true,
            address: "127.0.0.1:34567".parse().unwrap(),
            key: ChaCha20Poly1305::new_from_slice(&[0; 32]).unwrap(),
            message_sender,
            pending_target_change_responses: 0,
            pending_messages: Vec::new().into(),
        }
    }
}
