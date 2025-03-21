use std::net::SocketAddr;

use chacha20poly1305::ChaCha20Poly1305;
use input_simulator::{DeviceOutputError, InputSimulator};
use network::{transport::Transport, udp::TokioUdpTransport, Message, TransportError};
use thiserror::Error;
use tokio::{net::UdpSocket, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Error)]
pub enum InputEventListenerError {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Transport error: {0}")]
    TransportError(#[from] TransportError),
    #[error("Transport error: {0}")]
    DeviceOutputError(#[from] DeviceOutputError),
}

pub async fn input_event_listener(
    key: Option<ChaCha20Poly1305>,
    client_addr: SocketAddr,
    server_addr: SocketAddr,
    release_request_receiver: Receiver<()>,
    cancellation_token: CancellationToken,
) -> Result<(), InputEventListenerError> {
    let simulator = InputSimulator::new();

    println!("Creating UDP transport for server at {}", server_addr);
    let udp_socket = UdpSocket::bind(client_addr).await?;
    let udp_transport: TokioUdpTransport<ChaCha20Poly1305> =
        TokioUdpTransport::new(udp_socket, server_addr, key);

    input_event_processor(
        udp_transport,
        simulator,
        release_request_receiver,
        cancellation_token,
    )
    .await
}

async fn input_event_processor(
    mut transport: TokioUdpTransport<ChaCha20Poly1305>,
    mut simulator: InputSimulator,
    mut release_request_receiver: Receiver<()>,
    cancellation_token: CancellationToken,
) -> Result<(), InputEventListenerError> {
    loop {
        tokio::select! {
            message = transport.receive_message() => {
                match message {
                    Ok(event) => match event {
                        Message::InputEvent { event } => {
                            simulator.emit(event)?;
                        }
                        _ => {
                            eprintln!("Event is not an input event: {:?}", event);
                        }
                    },
                    Err(err) => {
                        eprintln!(
                            "An error has occured when listening to UDP messages: {:?}",
                            err
                        );
                        return Err(err.into())
                    }
                }
            },
            _ = release_request_receiver.recv() => {
                simulator.release_all()?;
            },
            _ = cancellation_token.cancelled() => {
                simulator.release_all()?;
                return Ok(())
            },
        }
    }
}
