mod client;
mod config;
mod dev;
mod handlers;
mod keyboard_state;
mod server;
mod state;

use chacha20poly1305::ChaCha20Poly1305;
use common::error::DynError;
use config::{init, Config};
use dev::start_device_listener;
use server::start_listening;
use state::State;
use std::sync::Arc;
use tokio::sync::RwLock;

static VAR_NAME: usize = 256;
const CHANNEL_BUF_LEN: usize = VAR_NAME;
// if performance is an issue consider using atomicptr for shared state
// pub struct SharedState<T: Crypto> {
//     clients: Arc<AtomicPtr<Vec<Client<T>>>>,
//     target: Arc<AtomicUsize>, // treat usize::MAX as no target
// }
//
// impl<T: Crypto> SharedState<T> {
//     pub fn new() -> Self {
//         let clients = Arc::new(AtomicPtr::new(Vec::new().as_mut_ptr()));
//         let target = Arc::new(AtomicUsize::new(usize::MAX));
//         SharedState { clients, target }
//     }
// }

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let Config {
        server_address,
        keyboard,
        mouse,
    } = init();
    let initial_state: State<ChaCha20Poly1305> = State::default();
    let state = Arc::new(RwLock::new(initial_state));

    let state_clone = state.clone();
    let _kbd_listener = tokio::task::spawn_blocking(move || {
        let _ = start_device_listener(keyboard, state_clone, server_address);
    });
    let state_clone = state.clone();
    let _mouse_listener = tokio::task::spawn_blocking(move || {
        let _ = start_device_listener(mouse, state_clone, server_address);
    });

    println!("Starting server");
    start_listening(server_address, state).await?;

    // TODO: handle exit gracefully
    // kbd_listener.join();
    // mouse_listener.join();

    Ok(())
}
