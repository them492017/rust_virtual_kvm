use std::{
    fmt,
    net::TcpStream,
    sync::mpsc,
    thread,
};

use chacha20poly1305::{
    aead::OsRng,
    ChaCha20Poly1305, KeyInit,
};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::Config;
use common::{
    error::DynError,
    net::Message,
    tcp::TcpTransport,
    transport::{EventListener, Transport},
};

pub struct Connection {
    // TODO: should have a tcp socket for the connection
    pub listener_thread: Option<thread::JoinHandle<Result<(), DynError>>>,
    pub channel_sender: mpsc::Sender<Message>,
    pub channel_receiver: mpsc::Receiver<Message>,
    pub symmetric_key: Option<ChaCha20Poly1305>,
    pub is_connected: bool,
}

impl Default for Connection {
    fn default() -> Self {
        let listener_thread = None;
        let (channel_sender, channel_receiver) = mpsc::channel();
        let symmetric_key = None;
        let is_connected = false;

        Connection {
            listener_thread,
            channel_sender,
            channel_receiver,
            symmetric_key,
            is_connected,
        }
    }
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Connection")
            .field("listener_thread", &self.listener_thread)
            .field("channel_sender", &self.channel_sender)
            .field("channel_receiver", &self.channel_receiver)
            .field("is_connected", &self.is_connected)
            .finish()
    }
}

impl Connection {
    pub fn connect(&mut self, config: &Config) -> Result<(), DynError> {
        println!("Retrying connection to server");

        if self.is_connected {
            return Ok(());
        }

        // TODO: could try to reuse symmetric key on reconnect
        // if let Some(key) = self.symmetric_key.as_ref() {
        //     // do something
        //     println!("Attempting to reuse symmetric key")
        // } else {
        //     // do something else
        //     println!("Fully retrying connection")
        // }

        let socket = TcpStream::connect(config.server_addr)?;
        let mut transport: TcpTransport<ChaCha20Poly1305> =
            TcpTransport::new(socket.try_clone().unwrap());

        println!("Sending ClientInit message to server");
        transport.send_message(Message::ClientInit {
            addr: config.client_addr,
        })?;

        let server_pub_key =
            if let Message::ExchangePubKey { pub_key } = transport.receive_message()? {
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
        transport.send_message(Message::ExchangePubKey {
            pub_key: public_key,
        })?;

        // wait for ack
        println!("Waiting for server ack");
        if let Message::Ack = transport.receive_message()? {
            println!("Received ack from server");
        } else {
            return Err("Server did not acknowledge client public key".into());
        };

        // generate cipher
        let cipher = {
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
        transport.key = Some(cipher.clone());
        self.is_connected = true;

        // TODO: refactor to use tokio and clean this up
        let listener_channel = self.channel_sender.clone();
        self.listener_thread = Some(thread::spawn(move || {
            let mut transport: TcpTransport<ChaCha20Poly1305> = TcpTransport::new(socket);
            transport.key = Some(cipher);
            println!(
                "Before starting to listen, the tcp transport had read: {:?}",
                transport.curr
            );
            println!("Transport key is set: {}", transport.key.is_some());
            let mut listener = EventListener::new(transport);
            listener.listen(listener_channel)
        }));

        // send handshake with encryption enabled
        transport.send_message(Message::Handshake)?;

        println!("Waiting for server handshake");
        if let Message::Handshake = self.channel_receiver.recv()? {
            println!("Received handshake from server");
        } else {
            return Err("Server did not initiate handshake".into());
        };

        println!(
            "Successfully connected to server at address {}",
            config.server_addr
        );

        Ok(())
    }
}
