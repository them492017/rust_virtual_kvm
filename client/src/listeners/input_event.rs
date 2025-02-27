use std::net::SocketAddr;

use chacha20poly1305::ChaCha20Poly1305;
use common::{
    dev::{make_keyboard, make_mouse},
    error::DynError,
    net::Message,
    transport::AsyncTransport,
    udp::TokioUdpTransport,
};
use evdev::{uinput::VirtualDevice, EventType, InputEvent};
use tokio::net::UdpSocket;

pub async fn input_event_listener(
    key: Option<ChaCha20Poly1305>,
    client_addr: SocketAddr,
    server_addr: SocketAddr,
) -> Result<(), DynError> {
    let virtual_keyboard = make_keyboard().expect("Could not create virtual keyboard");
    let virtual_mouse = make_mouse().expect("Could not create virtual mouse");

    println!("Creating UDP transport for server at {}", server_addr);
    let udp_socket = UdpSocket::bind(client_addr).await?;
    let udp_transport: TokioUdpTransport<ChaCha20Poly1305> =
        TokioUdpTransport::new(udp_socket, server_addr, key);

    println!("Starting to listen for input events over UDP");
    input_event_processor(udp_transport, virtual_keyboard, virtual_mouse).await
}

async fn input_event_processor(
    mut transport: TokioUdpTransport<ChaCha20Poly1305>,
    mut virtual_keyboard: VirtualDevice,
    mut virtual_mouse: VirtualDevice,
) -> Result<(), DynError> {
    loop {
        println!("Listening for message");
        match transport.receive_message().await {
            Ok(event) => {
                println!("{:?}", event);
                match event {
                    Message::InputEvent { event, .. } => {
                        println!("{:?}", event);
                        let input_event: InputEvent = event.into();
                        match input_event.event_type() {
                            EventType::KEY => {
                                println!("Emitting key event: {:?}", input_event);
                                virtual_keyboard.emit(&[input_event]).unwrap();
                            }
                            EventType::RELATIVE => {
                                println!("Emitting mouse event: {:?}", input_event);
                                virtual_mouse.emit(&[input_event]).unwrap();
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
