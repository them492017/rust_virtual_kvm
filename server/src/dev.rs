use common::{error::DynError, net::Message};
use evdev::InputEventKind;
use tokio::sync::mpsc;

use crate::{
    keyboard_state::{KeyboardState, CYCLE_TARGET},
    processor::InternalMessage,
    server_message::ServerMessage,
};

pub fn start_device_listener(
    mut device: evdev::Device,
    event_sender: mpsc::Sender<InternalMessage>,
) -> Result<(), DynError> {
    println!(
        "Starting device listener for device: {}",
        device.name().unwrap_or("")
    );
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
                        event_sender
                            .blocking_send(InternalMessage::LocalMessage {
                                message: ServerMessage::Cycle,
                            })
                            .unwrap();
                    }
                }
                if event.value() == 0 {
                    keyboard_state.release_key(key);
                }
            }
            // send the event
            event_sender
                .blocking_send(InternalMessage::ClientMessage {
                    message: Message::InputEvent {
                        event: event.into(),
                    },
                })
                .unwrap();
        });
    }
}
