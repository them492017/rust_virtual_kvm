use std::net::SocketAddr;

use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use crate::actors::{
    device::resource::DeviceResource, server::resource::ServerResource,
    state::resource::StateResource,
};

pub async fn run(server_addr: SocketAddr) {
    let (event_tx1, event_rx) = mpsc::channel(32);
    let (client_tx, client_rx) = mpsc::channel(32);
    let (client_message_tx, client_message_rx) = mpsc::channel(32);
    let (grab_request_tx, grab_request_rx1) = broadcast::channel(32);
    let cancellation_token = CancellationToken::new();

    let cancellation_token_clone = cancellation_token.clone();
    let event_processor = tokio::spawn(async move {
        let state = StateResource::default();
        state
            .process(
                server_addr,
                event_rx,
                client_message_rx,
                client_rx,
                grab_request_tx,
                cancellation_token_clone,
            )
            .await
    });

    let devices = DeviceResource::new();
    let cancellation_token_clone = cancellation_token.clone();
    let device_listener = tokio::spawn(async move {
        devices
            .start_device_listener(event_tx1, grab_request_rx1, cancellation_token_clone)
            .await
    });

    let server = ServerResource::new(server_addr).await;
    let client_tx_clone = client_tx.clone();
    let cancellation_token_clone = cancellation_token.clone();
    let server_actor =
        server.start_listening(client_tx_clone, client_message_tx, cancellation_token_clone);

    println!("Initialised server");
    tokio::select! {
        result = server_actor => {
            match result {
                Ok(()) => println!("Server closed gracefully"),
                Err(err) => eprintln!("Server exited with error: {}", err),
            }
        },
        result = device_listener => {
            match result {
                Ok(Ok(())) => println!("Device listener closed gracefully"),
                Ok(Err(err)) => eprintln!("Device listener exited with error: {}", err),
                Err(err) => eprintln!("Device listener panicked: {}", err),
            }
        },
        result = event_processor => {
            match result {
                Ok(Ok(())) => println!("Event processor closed gracefully"),
                Ok(Err(err)) => eprintln!("Event processor exited with error: {}", err),
                Err(err) => eprintln!("Event processor panicked: {}", err),
            }
        },
    }
    println!("Shutting down server");
    cancellation_token.cancel();
}
