mod client;
mod config;
mod dev;
mod handlers;
mod keyboard_state;
mod processor;
mod server;
mod server_message;
mod state;

use common::error::DynError;
use config::{init, Config};
use dev::start_device_listener;
use processor::event_processor;
use server::start_listening;
use tokio::sync::mpsc;

const CHANNEL_BUF_LEN: usize = 256;

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let Config {
        server_address,
        keyboard,
        mouse,
    } = init();
    let (event_tx, event_rx) = mpsc::channel(32);
    let (client_tx, client_rx) = mpsc::channel(32);
    let (client_message_tx, client_message_rx) = mpsc::channel(32);

    let event_processor = tokio::spawn(async move {
        event_processor(server_address, event_rx, client_message_rx, client_rx).await
    });

    let tx = event_tx.clone();
    let kbd_listener = tokio::task::spawn_blocking(move || start_device_listener(keyboard, tx));
    let tx = event_tx.clone();
    let mouse_listener = tokio::task::spawn_blocking(move || start_device_listener(mouse, tx));

    println!("Starting server");
    let client_tx_clone = client_tx.clone();
    tokio::select! {
        result = start_listening(server_address, client_tx_clone, client_message_tx) => {
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

    Ok(())
}
