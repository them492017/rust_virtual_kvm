use evdev::{InputEventKind, RelativeAxisType};
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use crate::{
    common::{dev::{make_keyboard, pick_device, release_all}, error::DynError, net::Message},
    server::keyboard_state::KeyboardState,
};

use super::{
    keyboard_state::CYCLE_TARGET, processor::InternalMessage, server_message::ServerMessage,
};

fn is_mouse(device: &evdev::Device) -> bool {
    device
        .supported_relative_axes()
        .map_or(false, |axes| axes.contains(RelativeAxisType::REL_X))
}

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
    let mut virtual_kbd = make_keyboard()?;
    let m = is_mouse(&device);
    let mut device_stream = device.into_event_stream()?;

    // TODO: REMOVE TIS
    if m {
        let mut kbd_2 = pick_device("extra kbd").into_event_stream()?;

        loop {
            tokio::select! {
                event = kbd_2.next_event() => {
                    dbg!(event?);
                },
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
                            release_all(&mut virtual_kbd)?;
                            device_stream.device_mut().grab().unwrap();
                        },
                        Ok(false) => {
                            device_stream.device_mut().ungrab().unwrap();
                        },
                        Err(err) => {
                            eprintln!("Grab request receive had an error: {}", err);
                            return Err(err.into())
                        }
                    }
                },
                _ = cancellation_token.cancelled() => {
                    release_all(&mut virtual_kbd)?;
                    return Ok(())
                },
            }
        }
    } else {
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
                        },
                        Err(err) => {
                            eprintln!("Grab request receive had an error: {}", err);
                            return Err(err.into())
                        }
                    }
                },
                _ = cancellation_token.cancelled() => {
                    release_all(&mut virtual_kbd)?;
                    return Ok(())
                },
            }
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
