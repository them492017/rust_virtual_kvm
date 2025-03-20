use input_event::{InputEvent, KeyboardEventType};
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
    #[error("Device output error: {0}")]
    DeviceOutputError(#[from] DeviceOutputError),
    #[error("Device input error: {0}")]
    DeviceInputError(#[from] DeviceInputError),
    #[error("Could not send internal message: {0}")]
    InternalMessageSendError(#[from] SendError<InternalMessage>),
    #[error("Could not receive grab request: {0}")]
    GrabRequestRecvError(#[from] RecvError),
}

impl DeviceResource {
    pub async fn start_device_listener(
        mut self,
        event_sender: mpsc::Sender<InternalMessage>,
        mut grab_request_receiver: broadcast::Receiver<bool>,
        cancellation_token: CancellationToken,
    ) -> Result<(), DeviceListenerError> {
        println!("Starting device listeners"); // TODO: add names of devices
        let mut keyboard_state = KeyboardState::default();

        loop {
            tokio::select! {
                // TODO: consider merging the two streams to avoid code repitition
                event = self.kbd_input_stream.next_event() => {
                    match event {
                        Ok(event) => {
                            if let InputEvent::Keyboard(keyboard_event) = event {
                                match keyboard_event.event_type {
                                    KeyboardEventType::KeyPressed => {
                                        // TODO: make keyboard_state use the generic input_event::Key enum
                                        // instead of coupling it to evdev
                                        keyboard_state.press_key(keyboard_event.key.into());
                                        // handle combinations
                                        self.handle_combinations(&mut keyboard_state, &event_sender).await?;
                                    }
                                    KeyboardEventType::KeyReleased => {
                                        keyboard_state.release_key(keyboard_event.key.into());
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
                event = self.mouse_input_stream.next_event() => {
                    match event {
                        Ok(event) => {
                            // if let InputEvent::Keyboard(keyboard_event) = event {
                            //     match keyboard_event {
                            //         KeyboardEvent::KeyPressed(key) => {
                            //             // TODO: make keyboard_state use the generic input_event::Key enum
                            //             // instead of coupling it to evdev
                            //             keyboard_state.press_key(key.into());
                            //             // handle combinations
                            //             self.handle_combinations(&mut keyboard_state, &event_sender).await?;
                            //         }
                            //         KeyboardEvent::KeyReleased(key) => {
                            //             keyboard_state.release_key(key.into());
                            //         }
                            //         _ => {}
                            //     }
                            // }
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
                            self.kbd_input_stream.grab_device()?;
                            self.mouse_input_stream.grab_device()?;
                            self.input_simulator.release_all()?;
                        },
                        Ok(false) => {
                            self.kbd_input_stream.ungrab_device()?;
                            self.mouse_input_stream.ungrab_device()?;
                        },
                        Err(err) => {
                            eprintln!("Grab request receive had an error: {}", err);
                            return Err(err.into())
                        }
                    }
                },
                _ = cancellation_token.cancelled() => {
                    self.input_simulator.release_all()?;
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
