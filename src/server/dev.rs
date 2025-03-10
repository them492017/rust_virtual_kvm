use evdev::InputEventKind;
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use crate::{
    common::{dev::release_all, error::DynError, net::Message},
    server::keyboard_state::KeyboardState,
};

use super::{
    keyboard_state::CYCLE_TARGET, processor::InternalMessage, server_message::ServerMessage,
};

pub async fn start_device_listener(
    device: evdev::Device,
    event_sender: mpsc::Sender<InternalMessage>,
    mut grab_request_receiver: broadcast::Receiver<bool>,
    cancellation_token: CancellationToken,
) -> Result<(), DynError> {
    println!(
        "Starting device listener for device: {}",
        device.name().unwrap_or("")
    );
    let mut keyboard_state = KeyboardState::default();
    let mut device_stream = device.into_event_stream()?;

    loop {
        tokio::select! {
            event = device_stream.next_event() => {
                let event = event?;
                if let InputEventKind::Key(key) = event.kind() {
                    if event.value() == 1 {
                        keyboard_state.press_key(key);
                        // handle combinations
                        handle_combinations(&mut keyboard_state, &event_sender).await?;
                    }
                    if event.value() == 0 {
                        keyboard_state.release_key(key);
                    }
                }
                let message = InternalMessage::ClientMessage { message: Message::InputEvent { event: event.into() }, sender: None };
                event_sender.send(message).await?;
            },
            request = grab_request_receiver.recv() => {
                match request {
                    Ok(true) => {
                        device_stream.device_mut().grab().unwrap();
                    },
                    Ok(false) => {
                        device_stream.device_mut().ungrab().unwrap();
                        release_all(device_stream.device_mut())?;
                    },
                    Err(err) => {
                        eprintln!("Grab request receive had an error: {}", err);
                        panic!();
                    }
                }
            },
            _ = cancellation_token.cancelled() => {
                release_all(device_stream.device_mut())?;
                return Ok(())
            },
        }
    }
}

async fn handle_combinations(
    keyboard_state: &mut KeyboardState,
    event_sender: &mpsc::Sender<InternalMessage>,
) -> Result<(), DynError> {
    if keyboard_state.is_combination_pressed(CYCLE_TARGET.to_vec()) {
        event_sender
            .send(InternalMessage::LocalMessage {
                message: ServerMessage::Cycle,
            })
            .await?
    }

    Ok(())
}
