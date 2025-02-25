mod connection;

use chacha20poly1305::ChaCha20Poly1305;
use common::dev::{make_keyboard, make_mouse};
use common::error::DynError;
use common::net::Message;
use common::transport::EventListener;
use common::udp::UdpTransport;
use evdev::{EventType, InputEvent};
use std::env;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc;

use crate::connection::Connection;

fn main() -> Result<(), DynError> {
    let mut config = init();
    let mut connection: Connection = Connection::default();

    println!("Beginning main loop");
    loop {
        if connection.is_connected {
            // process events
            println!("{:?}", connection);
            if let Err(error) = process_events(&connection, &mut config) {
                connection.is_connected = false;
                println!("{:?}", error);
                todo!("So far don't want to test reconnect logic")
            } else {
                println!("Connection closed gracefully");
            }
        } else {
            // try to connect
            connection
                .connect(&config)
                .inspect_err(|err| println!("Could not reconnect: {}", err))?;
            dbg!(&connection);
        }
    }
}

fn init() -> Config {
    println!("Initialising Virtual KVM Client");

    let args: Vec<String> = env::args().collect();
    let Ok((server_addr, client_addr)) = parse_args(&args) else {
        panic!("Could not parse addresses")
    };

    let udp_socket = UdpSocket::bind(client_addr).expect("Could not bind socket");
    println!("Started udp_socket on address {}", client_addr);

    Config {
        server_addr,
        client_addr,
        udp_socket,
    }
}

fn process_events(conn: &Connection, config: &mut Config) -> Result<(), DynError> {
    // start a listener thread which sends messages via channel
    let udp_transport: UdpTransport<ChaCha20Poly1305> =
        UdpTransport::new(config.udp_socket.try_clone().unwrap(), config.server_addr);
    input_event_listener(udp_transport);

    // get event from channel
    while let Ok(event) = conn.channel_receiver.recv() {
        println!("Received from TCP: {:?}", event);
        match event {
            Message::InputEvent { event, .. } => {
                println!("{:?}", event);
            }
            Message::ClipboardChanged { content } => {
                println!("New clipboard item: [{:?}]", content);
            }
            _ => {}
        }
    }

    // when channel is closed return err or return () on shutdown
    println!("Connection failed / channel closed or something???");
    Ok(()) // TODO: connection failed?
}

fn input_event_listener(transport: UdpTransport<ChaCha20Poly1305>) {
    let mut virtual_keyboard = make_keyboard().expect("Could not create virtual keyboard");
    let mut virtual_mouse = make_mouse().expect("Could not create virtual mouse");
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let mut listener = EventListener::new(transport);
        if let Err(err) = listener.listen(tx) {
            println!("Something went wrong in UDP listener: {}", err);
            unimplemented!("error handling for listener error not implemented")
        }
    });

    println!("Started input event listener");
    loop {
        println!("Listening for message");
        match rx.recv() {
            Ok(event) => {
                println!("{:?}", event);
                match event {
                    Message::InputEvent { event, .. } => {
                        println!("{:?}", event);
                        let input_event: InputEvent = event.into();
                        match input_event.event_type() {
                            EventType::KEY => {
                                println!("Key event");
                                let _ = virtual_keyboard.emit(&[input_event]);
                            }
                            EventType::RELATIVE => {
                                println!("Mouse event");
                                let _ = virtual_mouse.emit(&[input_event]);
                            }
                            _ => {
                                println!("Unimplemented event type");
                            }
                        }
                    }
                    _ => {
                        println!("Event is not a keyboard event: {:?}", event);
                    }
                }
            }
            Err(err) => {
                println!(
                    "An error has occured when listening to UDP messages:\n{}",
                    err
                );
                unimplemented!("error handling for UDP listener failing not implememnted")
            }
        }
    }
}

fn parse_args(args: &[String]) -> Result<(SocketAddr, SocketAddr), DynError> {
    if args.len() < 3 {
        panic!("Not enough arguments. Please provide a server address and client address");
    }

    Ok((args[1].parse()?, args[2].parse()?))
}

struct Config {
    server_addr: SocketAddr,
    client_addr: SocketAddr,
    udp_socket: UdpSocket,
}
