use evdev::InputEventKind;
use input_listener::DeviceInputError;
use input_simulator::DeviceOutputError;
use network::Message;
use thiserror::Error;
use tokio::sync::{
    broadcast::{self, error::RecvError},
    mpsc::{self, error::SendError},
};
use tokio_util::sync::CancellationToken;

use crate::{
    keyboard_state::{KeyboardState, CYCLE_TARGET},
    InternalMessage, ServerMessage,
};

use super::resource::Devices;

// TODO: rename error
#[derive(Debug, Error)]
pub enum DeviceListenerError {
    #[error("Device output error")]
    DeviceOutputError(#[from] DeviceOutputError),
    #[error("Device input error")]
    DeviceInputError(#[from] DeviceInputError),
    #[error("Could not send internal message")]
    InternalMessageSendError(#[from] SendError<InternalMessage>),
    #[error("Could not receive grab request")]
    GrabRequestRecvError(#[from] RecvError),
}

impl Devices {
    pub async fn start_device_listener(
        mut self,
        event_sender: mpsc::Sender<InternalMessage>,
        mut grab_request_receiver: broadcast::Receiver<bool>,
        cancellation_token: CancellationToken,
    ) -> Result<(), DeviceListenerError> {
        println!("Starting device listener for {}", self.device_type);
        let mut keyboard_state = KeyboardState::default();

        loop {
            tokio::select! {
                event = self.input_stream.next_event() => {
                    let event = event?;
                    if let InputEventKind::Key(key) = event.kind() {
                        if event.value() == 1 {
                            keyboard_state.press_key(key);
                            // handle combinations
                            self.handle_combinations(&mut keyboard_state, &event_sender).await?;
                        }
                        if event.value() == 0 {
                            keyboard_state.release_key(key);
                        }
                    }
                    let message = InternalMessage::ClientMessage {
                        message: Message::InputEvent { event: event.into() },
                        sender: None,
                    };
                    event_sender.send(message).await?;
                },
                request = grab_request_receiver.recv() => {
                    match request {
                        Ok(true) => {
                            self.virtual_kbd.release_all()?;
                            self.input_stream.grab_device()?;
                        },
                        Ok(false) => {
                            self.input_stream.ungrab_device()?;
                        },
                        Err(err) => {
                            eprintln!("Grab request receive had an error: {}", err);
                            return Err(err.into())
                        }
                    }
                },
                _ = cancellation_token.cancelled() => {
                    self.virtual_kbd.release_all()?;
                    return Ok(())
                },
            }
        }
    }

    async fn handle_combinations(
        &self,
        keyboard_state: &mut KeyboardState,
        event_sender: &mpsc::Sender<InternalMessage>,
    ) -> Result<(), DeviceListenerError> {
        if keyboard_state.is_combination_pressed(CYCLE_TARGET.to_vec()) {
            event_sender
                .send(InternalMessage::LocalMessage {
                    message: ServerMessage::Cycle,
                })
                .await?
        }

        Ok(())
    }
}
