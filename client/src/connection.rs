use std::{fmt, net::SocketAddr};

use chacha20poly1305::{aead::OsRng, ChaCha20Poly1305, KeyInit};
use tokio::{net::TcpStream, sync::mpsc};
use x25519_dalek::{EphemeralSecret, PublicKey};

use common::{error::DynError, net::Message, tcp2::TokioTcpTransport, transport::AsyncTransport};

pub struct Connection {
    pub sender: mpsc::Sender<()>, // TODO: maybe use type alias to indicate meaning
    pub receiver: mpsc::Receiver<()>,
    pub symmetric_key: Option<ChaCha20Poly1305>,
    pub is_connected: bool,
}

impl Default for Connection {
    fn default() -> Self {
        let (channel_sender, channel_receiver) = mpsc::channel(1);
        let symmetric_key = None;
        let is_connected = false;

        Connection {
            sender: channel_sender,
            receiver: channel_receiver,
            symmetric_key,
            is_connected,
        }
    }
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Connection")
            .field("channel_sender", &self.sender)
            .field("channel_receiver", &self.receiver)
            .field("is_connected", &self.is_connected)
            .finish()
    }
}

impl Connection {
    pub async fn connect(
        &mut self,
        client_addr: SocketAddr,
        server_addr: SocketAddr,
    ) -> Result<TokioTcpTransport<ChaCha20Poly1305>, DynError> {
        // TODO: could probably add a server secret + client secret to ensure sessions are uniqiue
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
            // TODO: could extract this into a trait for a more generic impl
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
        // TODO: need to set channels and stuff

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
}
