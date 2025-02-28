use std::net::SocketAddr;

use chacha20poly1305::{aead::OsRng, ChaCha20Poly1305, KeyInit};
use common::net::Message;
use common::tcp::TokioTcpTransport;
use common::transport::AsyncTransport;
use common::{crypto::Crypto, error::DynError};
use tokio::sync::mpsc::Sender;
use uuid::Uuid;
use x25519_dalek::{EphemeralSecret, PublicKey};

#[derive(Debug)]
pub struct Client<T: Crypto> {
    pub id: Uuid,
    pub connected: bool,
    pub address: SocketAddr,
    pub key: T,
    pub message_sender: Sender<Message>,
}

impl Client<ChaCha20Poly1305> {
    pub async fn connect(
        transport: &mut TokioTcpTransport<ChaCha20Poly1305>,
        message_sender: Sender<Message>,
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

        transport.set_key(cipher.clone());

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
            id: Uuid::new_v4(),
            connected: true,
            key: cipher,
            address: addr,
            message_sender,
        })
    }
}
