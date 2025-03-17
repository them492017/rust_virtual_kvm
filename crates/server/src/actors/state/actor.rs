use std::net::SocketAddr;

use chacha20poly1305::ChaCha20Poly1305;
use network::{input_event::InputEventTransport, Message, TransportError};
use thiserror::Error;
use tokio::{
    net::UdpSocket,
    sync::{broadcast, mpsc},
};
use tokio_util::sync::CancellationToken;

use crate::{client::Client, InternalMessage, ServerMessage};

use super::resource::{StateHandlerError, StateResource};

#[derive(Debug, Error)]
pub enum ProcessorError {
    #[error("Device message listener channel closed")]
    DeviceChannelClosed,
    #[error("Client message listener channel closed")]
    ClientReceiverChannelClosed,
    #[error("New client receiver channel closed")]
    ClientListenerChannelClosed,
    #[error("Invalid argument")]
    InvalidArgument,
    #[error("State error")]
    StateError(#[from] StateHandlerError),
    #[error("Transport error")]
    TransportError(#[from] TransportError),
    #[error("IO error")]
    IOError(#[from] std::io::Error),
}

impl StateResource<ChaCha20Poly1305> {
    // TODO: handle batches of events, not just single events
    pub async fn process(
        mut self,
        server_addr: SocketAddr,
        mut device_message_receiver: mpsc::Receiver<InternalMessage>,
        mut client_message_receiver: mpsc::Receiver<InternalMessage>,
        mut client_receiver: mpsc::Receiver<Client<ChaCha20Poly1305>>,
        mut grab_request_sender: broadcast::Sender<bool>,
        cancellation_token: CancellationToken,
    ) -> Result<(), ProcessorError> {
        // device_receiver receives incoming messages from device listeners
        // message is either of type Message or SpecialEvent
        //
        // client_receiver receives incoming messages from client listeners
        // message is of type Message
        //
        // when sending a message over TCP to a client, should send through a channel to the handler.
        // then, inside the handler, we may have the option to catch some messages before sending them
        // back to the global processor
        let socket = UdpSocket::bind(server_addr).await?;
        let mut transport = InputEventTransport::new(socket);

        loop {
            tokio::select! {
                msg = device_message_receiver.recv() => {
                    if let Some(message) = msg {
                        self.handle_device_message(message, &mut transport, &mut grab_request_sender).await?;
                    } else {
                        eprintln!("Event processor receiver was closed");
                        return Err(ProcessorError::DeviceChannelClosed);
                    }
                },
                msg = client_message_receiver.recv() => {
                    if let Some(message) = msg {
                        self.handle_client_message(message, &mut transport, &mut grab_request_sender).await?;
                    } else {
                        eprintln!("Client receiver was closed");
                        return Err(ProcessorError::ClientListenerChannelClosed);
                    }
                },
                client = client_receiver.recv() => {
                    match client {
                        Some(c) => {
                            self.add_client(c);
                        },
                        None => {
                            eprintln!("Client receiver was closed");
                            return Err(ProcessorError::ClientReceiverChannelClosed);
                        }
                    }
                },
                _ = cancellation_token.cancelled() => {
                    eprintln!("Event processing was cancelled");
                    return Ok(())
                }
            }
        }
    }

    async fn handle_device_message(
        &mut self,
        msg: InternalMessage,
        transport: &mut InputEventTransport,
        grab_request_sender: &mut broadcast::Sender<bool>,
    ) -> Result<(), ProcessorError> {
        match msg {
            InternalMessage::ClientMessage { message, .. } => {
                // send input event to correct client over udp
                match &message {
                    Message::InputEvent { .. } => {
                        if let Some(target) = self.get_target_mut() {
                            if target.can_receive() {
                                transport
                                    .send_message_to(
                                        message,
                                        target.address,
                                        Some(target.key.clone()),
                                    )
                                    .await?;
                            } else {
                                target.buffer_message(message);
                            }
                        }
                    }
                    _ => {
                        // TODO: send over tcp
                        unimplemented!("TCP sending / non-input event processing is unimplemented");
                    }
                }
            }
            InternalMessage::LocalMessage { message } => {
                // forward special event to client handler to be sent over tcp
                match &message {
                    ServerMessage::ClientDisconnect { id } => {
                        self.disconnect_client(*id, grab_request_sender).await?;
                    }
                    ServerMessage::Cycle => {
                        self.cycle_target(grab_request_sender).await?;
                    }
                }
            }
        };
        Ok(())
    }

    async fn handle_client_message(
        &mut self,
        msg: InternalMessage,
        transport: &mut InputEventTransport,
        grab_request_sender: &mut broadcast::Sender<bool>,
    ) -> Result<(), ProcessorError> {
        match msg {
            InternalMessage::ClientMessage { message, sender } => match &message {
                Message::Heartbeat => {}
                Message::ClipboardChanged { content: _ } => {
                    unimplemented!("Clipboard change is not implemented in state actor");
                }
                Message::TargetChangeResponse => {
                    let sender = sender.ok_or(ProcessorError::InvalidArgument)?;
                    self.handle_change_target_response(sender, transport)
                        .await?;
                }
                _ => {
                    unimplemented!("Received unimplemented client message: {:?}", message);
                }
            },
            InternalMessage::LocalMessage { message } => match &message {
                ServerMessage::ClientDisconnect { id } => self
                    .disconnect_client(*id, grab_request_sender)
                    .await
                    .inspect_err(|err| eprintln!("Error while disconnecting client: {}", err))?,
                ServerMessage::Cycle => {
                    self.cycle_target(grab_request_sender).await?;
                }
            },
        };
        Ok(())
    }
}
