use std::net::SocketAddr;

use chacha20poly1305::ChaCha20Poly1305;
use evdev::{uinput::VirtualDevice, EventType, InputEvent};
use input_simulator::{
    x11::dev::{make_keyboard, make_mouse, release_all},
    DeviceOutputError,
};
use network::{transport::Transport, udp::TokioUdpTransport, Message, TransportError};
use thiserror::Error;
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Error)]
pub enum InputEventListenerError {
    #[error("IO error")]
    IOError(#[from] std::io::Error),
    #[error("Transport error")]
    TransportError(#[from] TransportError),
    #[error("Transport error")]
    DeviceOutputError(#[from] DeviceOutputError),
}

pub async fn input_event_listener(
    key: Option<ChaCha20Poly1305>,
    client_addr: SocketAddr,
    server_addr: SocketAddr,
    cancellation_token: CancellationToken,
) -> Result<(), InputEventListenerError> {
    let virtual_keyboard = make_keyboard().expect("Could not create virtual keyboard");
    let virtual_mouse = make_mouse().expect("Could not create virtual mouse");

    println!("Creating UDP transport for server at {}", server_addr);
    let udp_socket = UdpSocket::bind(client_addr).await?;
    let udp_transport: TokioUdpTransport<ChaCha20Poly1305> =
        TokioUdpTransport::new(udp_socket, server_addr, key);

    input_event_processor(
        udp_transport,
        virtual_keyboard,
        virtual_mouse,
        cancellation_token,
    )
    .await
}

async fn input_event_processor(
    mut transport: TokioUdpTransport<ChaCha20Poly1305>,
    mut virtual_keyboard: VirtualDevice,
    mut virtual_mouse: VirtualDevice,
    cancellation_token: CancellationToken,
) -> Result<(), InputEventListenerError> {
    loop {
        tokio::select! {
            message = transport.receive_message() => {
                if let Ok(event) = message {
                    match event {
                        Message::InputEvent { event } => {
                            let input_event: InputEvent = event.into();
                            match input_event.event_type() {
                                EventType::KEY => {
                                    virtual_keyboard.emit(&[input_event]).unwrap();
                                }
                                EventType::RELATIVE => {
                                    virtual_mouse.emit(&[input_event]).unwrap();
                                }
                                _ => {
                                    eprintln!("Unimplemented event type: {input_event:?}");
                                }
                            }
                        }
                        _ => {
                            eprintln!("Event is not a keyboard event: {:?}", event);
                        }
                    }
                } else {
                    eprintln!(
                        "An error has occured when listening to UDP messages: {:?}",
                        message.as_ref().err()
                    );
                    return Err(message.err().unwrap().into())
                }
            },
            _ = cancellation_token.cancelled() => {
                release_all(&mut virtual_keyboard)?;
                return Ok(())
            },
        }
    }
}
