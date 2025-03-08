use std::net::SocketAddr;

use tokio::sync::{broadcast, mpsc};

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
    let (event_tx, event_rx) = mpsc::channel(32);
    let (client_tx, client_rx) = mpsc::channel(32);
    let (client_message_tx, client_message_rx) = mpsc::channel(32);
    let (grab_request_tx, _) = broadcast::channel(32);

    // TODO: rename
    let rx1 = grab_request_tx.subscribe();
    let rx2 = grab_request_tx.subscribe();

    let event_processor = tokio::spawn(async move {
        event_processor(
            server_addr,
            event_rx,
            client_message_rx,
            client_rx,
            grab_request_tx,
        )
        .await
    });

    // TODO: rename
    let tx = event_tx.clone();
    let kbd_listener = tokio::spawn(async { start_device_listener(keyboard, tx, rx1).await });
    let tx = event_tx.clone();
    let mouse_listener = tokio::spawn(async { start_device_listener(mouse, tx, rx2).await });

    println!("Starting server");
    let client_tx_clone = client_tx.clone();
    tokio::select! {
        result = start_listening(server_addr, client_tx_clone, client_message_tx) => {
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

    todo!("Add shutdown tokens to force shutdown");
}
