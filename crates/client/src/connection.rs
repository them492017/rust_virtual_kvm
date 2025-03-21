use std::net::SocketAddr;

use chacha20poly1305::{aead::OsRng, ChaCha20Poly1305, KeyInit};
use network::{tcp::TokioTcpTransport, transport::Transport, Message, TransportError};
use thiserror::Error;
use tokio::{net::TcpStream, sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::listeners::{
    input_event::InputEventListenerError, special_event::SpecialEventProcessorError,
};

use super::listeners::{input_event::input_event_listener, special_event::special_event_processor};

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Transport error: {0}")]
    TransportError(#[from] TransportError),
    #[error("Invalid message received from server during connection process: '{0}'")]
    InvalidMessage(String),
    #[error("Shared Diffe-Hellman secret was not contributory")]
    DHContributionError,
}

pub struct Connection {
    pub is_connected: bool,
    symmetric_key: Option<ChaCha20Poly1305>,
}

impl Default for Connection {
    fn default() -> Self {
        let symmetric_key = None;
        let is_connected = false;

        Connection {
            is_connected,
            symmetric_key,
        }
    }
}

impl Connection {
    pub async fn connect(
        &mut self,
        client_addr: SocketAddr,
        server_addr: SocketAddr,
    ) -> Result<TokioTcpTransport<ChaCha20Poly1305>, ConnectionError> {
        // TODO: add a server secret + client secret to ensure sessions are uniqiue
        // as in TCP 1.3
        println!("Retrying connection to server");

        let socket = TcpStream::connect(server_addr).await?;
        let mut transport: TokioTcpTransport<ChaCha20Poly1305> = TokioTcpTransport::new(socket);

        println!("Sending ClientInit message to server");
        transport
            .send_message(Message::ClientInit { addr: client_addr })
            .await?;

        let server_pub_key =
            if let Message::ExchangePubKey { pub_key } = transport.receive_message().await? {
                println!("Received pub key from server");
                pub_key
            } else {
                return Err(ConnectionError::InvalidMessage(
                    "Expected public key exchange".into(),
                ));
            };

        // generate public key
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public_key = PublicKey::from(&secret);

        // send pub key to server
        // TODO: sign this message
        println!("Sending pub key to server");
        transport
            .send_message(Message::ExchangePubKey {
                pub_key: public_key,
            })
            .await?;

        // wait for ack
        println!("Waiting for server ack");
        if let Message::ExchangePubKeyResponse = transport.receive_message().await? {
            println!("Received ack from server");
        } else {
            return Err(ConnectionError::InvalidMessage(
                "Server did not acknowledge client public key".into(),
            ));
        };

        // generate cipher
        let cipher = {
            // extract this into a trait method if supporting different types of keys
            let shared_secret = secret.diffie_hellman(&server_pub_key);
            if !shared_secret.was_contributory() {
                return Err(ConnectionError::DHContributionError);
            }

            Ok::<_, ConnectionError>(
                ChaCha20Poly1305::new_from_slice(&shared_secret.to_bytes())
                    .expect("Could not generate cipher from shared secret"),
            )
        }?;

        self.symmetric_key = Some(cipher.clone());
        transport.set_key(cipher);
        self.is_connected = true;

        // send handshake with encryption enabled
        transport.send_message(Message::Handshake).await?;

        println!("Waiting for server handshake");
        if let Message::Handshake = transport.receive_message().await? {
            println!("Received handshake from server");
        } else {
            return Err(ConnectionError::InvalidMessage(
                "Server did not initiate handshake".into(),
            ));
        };

        println!(
            "Successfully connected to server at address {}",
            server_addr
        );

        Ok(transport)
    }

    pub async fn spawn_listeners(
        &self,
        transport: TokioTcpTransport<ChaCha20Poly1305>,
        server_addr: SocketAddr,
        client_addr: SocketAddr,
    ) -> Result<ListenerHandles, ConnectionError> {
        let key = self.symmetric_key.clone();
        let (release_request_sender, release_request_receiver) = mpsc::channel(8);
        let cancellation_token = CancellationToken::new();
        let cloned_token = cancellation_token.clone();

        let input_event = tokio::spawn(async move {
            input_event_listener(
                key,
                client_addr,
                server_addr,
                release_request_receiver,
                cloned_token,
            )
            .await
        });
        let cloned_token = cancellation_token.clone();
        let special_event = tokio::spawn(async move {
            special_event_processor(transport, release_request_sender, cloned_token).await
        });

        Ok(ListenerHandles {
            input_event,
            special_event,
            cancellation_token,
        })
    }
}

pub struct ListenerHandles {
    pub input_event: JoinHandle<Result<(), InputEventListenerError>>,
    pub special_event: JoinHandle<Result<(), SpecialEventProcessorError>>,
    pub cancellation_token: CancellationToken,
}
