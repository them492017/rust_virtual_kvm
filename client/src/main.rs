mod state;

use chacha20poly1305::aead::OsRng;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
use common::dev::{make_keyboard, make_mouse};
use evdev::uinput::VirtualDevice;
use state::State;
use tokio::net::tcp::OwnedWriteHalf;
use x25519_dalek::{EphemeralSecret, PublicKey};
use std::error::Error;
use std::{env, net::UdpSocket, sync::Arc};
use common::net::{self, send_message, Message};
use common::{net::recv_event, dev::pick_device};
use evdev::{Device, EventType};
use tokio::net::TcpSocket;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let (server_addr, client_addr) = parse_args(&args);

    // Open devices
    println!("Pick a keyboard");
    let keyboard = pick_device();
    println!("Pick a mouse");
    let mouse = pick_device();

    let physical_devices: Vec<_> = vec![&keyboard.physical_path(), &mouse.physical_path()] // TODO: physical path not wroking????
        .iter_mut()
        .map(|path| Device::open(dbg!(path.unwrap())).unwrap())
        .collect();

    let mut keyboard = make_keyboard().expect("Could not create virtual keyboard");
    let mut mouse = make_mouse().expect("Could not create virtual mouse");

    let tcp_socket = TcpSocket::new_v4().expect("Could not create TCP socket");
    let stream = tcp_socket.connect(server_addr.parse().expect("Could not parse server address")).await?;
    let (reader, writer) = stream.into_split();
    let (channel_sender, message_channel) = tokio::sync::mpsc::channel(1024);

    tokio::spawn(async move {
        if let Err(e) = net::message_producer(reader, channel_sender).await {
            eprintln!("Error in message producer: {}", e);
        }
    });

    // init connection
    let cipher = get_cipher_from_server(client_addr, writer, message_channel).await?;

    // Initialise state
    let mut state = State::new(server_addr.to_string(), physical_devices, cipher.clone()); // TODO: remove clone
    state.add_target(client_addr.to_string());
    let _state = Arc::new(RwLock::new(state));

    handle_input_events(server_addr, &mut keyboard, &mut mouse, cipher.clone());

    Ok(())
}

// TODO: same function is used in server.rs
fn parse_args(args: &[String]) -> (&str, &str) {
    if args.len() < 3 {
        panic!("Not enough arguments. Please provide a server address and client address");
    }

    (&args[1], &args[2])
}

fn handle_input_events(server_addr: &str, keyboard: &mut VirtualDevice, mouse: &mut VirtualDevice, mut cipher: ChaCha20Poly1305) {
    let socket = UdpSocket::bind(server_addr).expect("Couldn't bind to address");

    loop {
        if let Ok(event) = recv_event(&socket, &mut cipher) {
            match event.event_type() {
                EventType::KEY => {
                    println!("Emitting: {event:?}");
                    keyboard.emit(&[event]).unwrap();
                }
                EventType::RELATIVE => {
                    println!("Emitting: {event:?}");
                    mouse.emit(&[event]).unwrap();
                }
                _ => {
                    // let t = event.event_type();
                    // println!("Ignoring {t:?} events");
                }
            }
        } else {
            println!("Error in received event");
        }
    }
}

async fn get_cipher_from_server(client_addr: &str, mut socket_writer: OwnedWriteHalf, mut message_channel: tokio::sync::mpsc::Receiver<Message>) -> Result<ChaCha20Poly1305, Box<dyn Error>> {
    // TODO: do this in main function so we can create message channel in main and borrow it here
    send_message(&mut socket_writer, Message::ClientInit { addr: client_addr.to_string() }).await?;

    // Receive public key from server
    let mut server_public_key: Option<PublicKey> = None;
    while server_public_key.is_none() {
        let message = message_channel.recv().await.expect("Message channel was closed unexpectedly");

        if let Message::ExchangePubKey { pub_key } = message {
            server_public_key = Some(pub_key)
        }
    }

    // Generate own public key
    let secret = EphemeralSecret::random_from_rng(OsRng);
    let public_key = PublicKey::from(&secret);

    let shared_secret = secret.diffie_hellman(&server_public_key.unwrap());

    if !shared_secret.was_contributory() {
        panic!("Shared secret was not contributory!")
    }

    send_message(&mut socket_writer, Message::ExchangePubKey { pub_key: public_key }).await?;

    // Create chacha cipher
    let cipher = ChaCha20Poly1305::new_from_slice(&shared_secret.to_bytes())
        .expect("Could not create cipher");

    // // Receive acknowledgement from server
    // loop {
    //     let message = message_channel.recv().await.expect("Message channel was closed unexpectedly");
    //
    //     if let Message::Ack { encrypted_msg } = message {
    //         println!("{}", encrypted_msg);
    //         todo!()
    //         break
    //     }
    // }

    Ok(cipher)
}
