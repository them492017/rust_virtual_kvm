use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
};

use chacha20poly1305::ChaCha20Poly1305;
use common::{error::DynError, net::Message, transport::Transport, udp2::TargetlessUdpTransport};
use evdev::InputEventKind;
use tokio::sync::RwLock;

use crate::{
    keyboard_state::{KeyboardState, CYCLE_TARGET},
    state::State,
};

pub fn start_device_listener(
    mut device: evdev::Device,
    state: Arc<RwLock<State<ChaCha20Poly1305>>>,
    address: SocketAddr,
) -> Result<(), DynError> {
    println!(
        "Starting device listener for device: {}",
        device.name().unwrap_or("")
    );
    let socket = UdpSocket::bind(address)?;
    let mut transport: TargetlessUdpTransport<ChaCha20Poly1305> =
        TargetlessUdpTransport::new(socket, address);
    let mut keyboard_state = KeyboardState::default();

    loop {
        device.fetch_events().unwrap().for_each(|event| {
            // dbg!(event);
            if let InputEventKind::Key(key) = event.kind() {
                if event.value() == 1 {
                    keyboard_state.press_key(key);
                    // handle combinations
                    if keyboard_state.is_combination_pressed(CYCLE_TARGET.to_vec()) {
                        // TODO: blocking write is almost certainly awful here
                        let _ = state.blocking_write().cycle_target();
                    }
                }
                if event.value() == 0 {
                    keyboard_state.release_key(key);
                }
            }
            // send the event
            if let Some(address) = state
                .blocking_read()
                .get_target()
                .map(|tgt| tgt.blocking_lock().address)
            {
                println!("Sending event to {}", address);
                transport.set_address(address);
                if let Err(err) = transport.send_message(Message::InputEvent {
                    event: event.into(),
                }) {
                    panic!("Could not send input event to target client: {}", err)
                }
            }
        });
    }
}
