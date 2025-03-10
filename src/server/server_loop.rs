use std::net::SocketAddr;

use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use crate::{
    common::error::DynError,
    server::{
        config::{init, Config},
        dev::start_device_listener,
        processor::event_processor,
        server::start_listening,
    },
};

pub async fn run(server_addr: SocketAddr) -> Result<(), DynError> {
    let Config { keyboard, mouse } = init();
    let (event_tx1, event_rx) = mpsc::channel(32);
    let event_tx2 = event_tx1.clone();
    let (client_tx, client_rx) = mpsc::channel(32);
    let (client_message_tx, client_message_rx) = mpsc::channel(32);
    let (grab_request_tx, grab_request_rx1) = broadcast::channel(32);
    let grab_request_rx2 = grab_request_tx.subscribe();
    let cancellation_token = CancellationToken::new();

    let cancellation_token_clone = cancellation_token.clone();
    let event_processor = tokio::spawn(async move {
        event_processor(
            server_addr,
            event_rx,
            client_message_rx,
            client_rx,
            grab_request_tx,
            cancellation_token_clone,
        )
        .await
    });

    let cancellation_token_clone = cancellation_token.clone();
    let kbd_listener = tokio::spawn(async {
        start_device_listener(
            keyboard,
            event_tx1,
            grab_request_rx1,
            cancellation_token_clone,
        )
        .await
    });
    let cancellation_token_clone = cancellation_token.clone();
    let mouse_listener = tokio::spawn(async {
        start_device_listener(mouse, event_tx2, grab_request_rx2, cancellation_token_clone).await
    });

    println!("Starting server");
    let client_tx_clone = client_tx.clone();
    let cancellation_token_clone = cancellation_token.clone();
    tokio::select! {
        result = start_listening(server_addr, client_tx_clone, client_message_tx, cancellation_token_clone) => {
            match result {
                Ok(()) => println!("Server closed gracefully"),
                Err(err) => eprintln!("Server exited with error: {}", err),
            }
        },
        result = kbd_listener => {
            match result {
                Ok(Ok(())) => println!("Keyboard listener closed gracefully"),
                Ok(Err(err)) => eprintln!("Keyboard listener exited with error: {}", err),
                Err(err) => eprintln!("Keyboard listener panicked: {}", err),
            }
        },
        result = mouse_listener => {
            match result {
                Ok(Ok(())) => println!("Mouse listener closed gracefully"),
                Ok(Err(err)) => eprintln!("Mouse listener exited with error: {}", err),
                Err(err) => eprintln!("Mouse listener panicked: {}", err),
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

    Ok(())
}
