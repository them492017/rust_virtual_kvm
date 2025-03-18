use input_event::{InputEvent, KeyboardEvent};
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

use super::resource::DeviceResource;

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

impl DeviceResource {
    pub async fn start_device_listener(
        mut self,
        event_sender: mpsc::Sender<InternalMessage>,
        mut grab_request_receiver: broadcast::Receiver<bool>,
        cancellation_token: CancellationToken,
    ) -> Result<(), DeviceListenerError> {
        println!("Starting device listener for {}", "[INSERT DEV NAME HERE]"); // TODO: add name
        let mut keyboard_state = KeyboardState::default();

        loop {
            tokio::select! {
                event = self.input_stream.next_event() => {
                    match event {
                        Ok(event) => {
                            if let InputEvent::Keyboard(keyboard_event) = event {
                                match keyboard_event {
                                    KeyboardEvent::KeyPressed(key) => {
                                        // TODO: make keyboard_state use the generic input_event::Key enum
                                        // instead of coupling it to evdev
                                        keyboard_state.press_key(key.into());
                                        // handle combinations
                                        self.handle_combinations(&mut keyboard_state, &event_sender).await?;
                                    }
                                    KeyboardEvent::KeyReleased(key) => {
                                        keyboard_state.release_key(key.into());
                                    }
                                    _ => {}
                                }
                            }
                            let message = InternalMessage::ClientMessage {
                                message: Message::InputEvent { event },
                                sender: None,
                            };
                            event_sender.send(message).await?;
                        },
                        Err(DeviceInputError::InputEventConversionError(_)) => {},
                        Err(err) => {
                            return Err(err.into())
                        },
                    }
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
