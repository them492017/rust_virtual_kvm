use std::{future::Future, net::SocketAddr};

use chacha20poly1305::{aead::OsRng, ChaCha20Poly1305, KeyInit};
use common::net::Message;
use common::tcp2::TokioTcpTransport;
use common::transport::AsyncTransport;
use common::{crypto::Crypto, error::DynError};
use tokio::sync::mpsc::Sender;
use x25519_dalek::{EphemeralSecret, PublicKey};

#[allow(dead_code)]
pub trait ClientInterface<T: Crypto>: Sized {
    fn connect(
        transport: TokioTcpTransport<T>,
        sender: Sender<Message>,
    ) -> impl Future<Output = Result<Self, DynError>>;
    // called by keyboard / mouse listener threads
    // send things on special patterns (eg clipboard)
    // maybe heartbeats will be sent from main thread
    // probably doesn't work since we need a mutable reference...
    fn send_message(&mut self, message: Message) -> impl Future<Output = Result<(), DynError>>;
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Client<T: Crypto> {
    pub connected: bool,
    pub address: SocketAddr,
    pub transport: TokioTcpTransport<T>,
    pub sender: Sender<Message>,
}

impl ClientInterface<ChaCha20Poly1305> for Client<ChaCha20Poly1305> {
    async fn connect(
        mut transport: TokioTcpTransport<ChaCha20Poly1305>,
        sender: Sender<Message>,
    ) -> Result<Self, DynError> {
        println!("Initialising client");

        let addr = match transport.receive_message().await {
            Ok(Message::ClientInit { addr }) => {
                println!("Received addr: {}", addr);
                addr
            }
            Ok(message) => {
                println!("Received message: {}", message);
                return Err("Unrecognised initial message from client".into());
            }
            Err(err) => {
                println!("Did not receive init message");
                return Err(err);
            }
        };

        // generate pub key
        // TODO: should sign this
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let pub_key = PublicKey::from(&secret);

        transport
            .send_message(Message::ExchangePubKey { pub_key })
            .await?;
        println!("Send pub key to client");

        let client_pub_key = match transport.receive_message().await {
            Ok(Message::ExchangePubKey { pub_key }) => {
                println!("Received public key from client");
                pub_key
            }
            Ok(message) => {
                println!("Received message: {}", message);
                return Err("Client did not exchange public keys".into());
            }
            Err(err) => {
                println!("Did not receive pub key message");
                return Err(err);
            }
        };

        transport.send_message(Message::Ack).await?;
        println!("Sent ack to client");

        let cipher = {
            let shared_secret = secret.diffie_hellman(&client_pub_key);
            if !shared_secret.was_contributory() {
                return Err("Shared secret was not contributory".into());
            }

            Ok::<_, DynError>(
                ChaCha20Poly1305::new_from_slice(&shared_secret.to_bytes())
                    .expect("Could not generate cipher from shared secret"),
            )
        }?;

        transport.set_key(cipher);

        // send handshake with encryption enabled
        transport.send_message(Message::Handshake).await?;

        println!("Waiting for client handshake");
        if let Message::Handshake = transport.receive_message().await? {
            println!("Received handshake from client");
        } else {
            return Err("Client did not initiate handshake".into());
        };

        println!("Successfully connected to client at address {:?}", addr);

        Ok(Client {
            connected: true,
            address: addr,
            transport,
            sender,
        })
    }

    async fn send_message(&mut self, message: Message) -> Result<(), DynError> {
        self.transport.send_message(message).await
    }
}
