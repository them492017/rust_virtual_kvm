use std::net::SocketAddr;

use chacha20poly1305::{aead::OsRng, ChaCha20Poly1305, KeyInit};
use tokio::{net::TcpStream, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use x25519_dalek::{EphemeralSecret, PublicKey};

use common::{error::DynError, net::Message, tcp::TokioTcpTransport, transport::AsyncTransport};

use crate::listeners::{input_event::input_event_listener, special_event::special_event_processor};

pub struct Connection {
    pub is_connected: bool,
    symmetric_key: Option<ChaCha20Poly1305>,
    transport: Option<TokioTcpTransport<ChaCha20Poly1305>>,
}

impl Default for Connection {
    fn default() -> Self {
        let symmetric_key = None;
        let is_connected = false;
        let transport = None;

        Connection {
            is_connected,
            symmetric_key,
            transport,
        }
    }
}

impl Connection {
    pub async fn connect(
        &mut self,
        client_addr: SocketAddr,
        server_addr: SocketAddr,
    ) -> Result<TokioTcpTransport<ChaCha20Poly1305>, DynError> {
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
                return Err("Invalid response from server. Expected public key exchange.".into());
            };

        // TODO: first we should verify pub key signature

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
        if let Message::Ack = transport.receive_message().await? {
            println!("Received ack from server");
        } else {
            return Err("Server did not acknowledge client public key".into());
        };

        // generate cipher
        let cipher = {
            // extract this into a trait method if supporting different keys
            let shared_secret = secret.diffie_hellman(&server_pub_key);
            if !shared_secret.was_contributory() {
                return Err("Shared secret was not contributory".into());
            }

            Ok::<_, DynError>(
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
            return Err("Server did not initiate handshake".into());
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
    ) -> Result<ListenerHandles, DynError> {
        let key = self.symmetric_key.clone();
        let cancellation_token = CancellationToken::new();
        let cloned_token = cancellation_token.clone();
        let input_event = tokio::spawn(async move {
            input_event_listener(key, client_addr, server_addr, cloned_token).await
        });
        let cloned_token = cancellation_token.clone();
        let special_event =
            tokio::spawn(async move { special_event_processor(transport, cloned_token).await });

        Ok(ListenerHandles {
            input_event,
            special_event,
            cancellation_token,
        })
    }
}

pub struct ListenerHandles {
    pub input_event: JoinHandle<Result<(), DynError>>,
    pub special_event: JoinHandle<Result<(), DynError>>,
    pub cancellation_token: CancellationToken,
}
