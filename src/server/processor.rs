use std::net::SocketAddr;

use chacha20poly1305::ChaCha20Poly1305;
use tokio::{
    net::UdpSocket,
    sync::{broadcast, mpsc},
};
use uuid::Uuid;

use crate::common::{error::DynError, net::Message};

use super::{
    client::Client, input_event_transport::InputEventTransport, server_message::ServerMessage,
    state::state::State,
};

#[derive(Debug)]
pub enum InternalMessage {
    ClientMessage {
        message: Message,
        sender: Option<Uuid>,
    },
    LocalMessage {
        message: ServerMessage,
    },
}

// TODO: handle batches of events, not just single events
pub async fn event_processor(
    server_addr: SocketAddr,
    mut device_message_receiver: mpsc::Receiver<InternalMessage>,
    mut client_message_receiver: mpsc::Receiver<InternalMessage>,
    mut client_receiver: mpsc::Receiver<Client<ChaCha20Poly1305>>,
    mut grab_request_sender: broadcast::Sender<bool>,
) -> Result<(), DynError> {
    // device_receiver receives incoming messages from device listeners
    // message is either of type Message or SpecialEvent
    //
    // client_receiver receives incoming messages from client listeners
    // message is of type Message
    //
    // when sending a message over TCP to a client, should send through a channel to the handler.
    // then, inside the handler, we may have the option to catch some messages before sending them
    // back to the global processor
    let mut state: State<ChaCha20Poly1305> = State::default();
    let socket = UdpSocket::bind(server_addr).await?;
    // let mut transport: TargetlessUdpTransport<ChaCha20Poly1305> =
    //     TargetlessUdpTransport::new(socket, server_addr);
    let mut transport = InputEventTransport::new(socket);

    loop {
        tokio::select! {
            msg = device_message_receiver.recv() => {
                if let Some(message) = msg {
                    handle_device_message(message, &mut state, &mut transport, &mut grab_request_sender).await?;
                } else {
                    eprintln!("Event processor receiver was closed");
                    return Err("Event processor receiver was closed".into());
                }
            },
            msg = client_message_receiver.recv() => {
                if let Some(message) = msg {
                    handle_client_message(message, &mut state, &mut transport).await?;
                } else {
                    eprintln!("Client receiver was closed");
                    return Err("Client receiver was closed".into());
                }
            },
            client = client_receiver.recv() => {
                match client {
                    Some(c) => {
                        state.add_client(c);
                    },
                    None => {
                        eprintln!("Client receiver was closed");
                        return Err("Client receiver was closed".into());
                    }
                }
            },
        }
    }
}

async fn handle_device_message(
    msg: InternalMessage,
    state: &mut State<ChaCha20Poly1305>,
    transport: &mut InputEventTransport,
    grab_request_sender: &mut broadcast::Sender<bool>,
) -> Result<(), DynError> {
    match msg {
        InternalMessage::ClientMessage { message, .. } => {
            // send input event to correct client over udp
            match &message {
                Message::InputEvent { .. } => {
                    if let Some(target) = state.get_target_mut() {
                        if target.can_receive() {
                            transport
                                .send_message_to(message, target.address, Some(target.key.clone()))
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
                    state.disconnect_client(*id);
                }
                ServerMessage::Cycle => {
                    state.cycle_target(grab_request_sender).await?;
                }
            }
        }
    };
    Ok(())
}

async fn handle_client_message(
    msg: InternalMessage,
    state: &mut State<ChaCha20Poly1305>,
    transport: &mut InputEventTransport,
) -> Result<(), DynError> {
    match msg {
        InternalMessage::ClientMessage { message, sender } => match &message {
            Message::Heartbeat => {}
            Message::ClipboardChanged { content } => {
                state.clipboard_contents = Some(content.to_string()); // TODO: handle race condition...
            }
            Message::TargetChangeResponse => {
                let sender =
                    sender.ok_or::<DynError>("No sender provided for client message".into())?;
                state
                    .handle_change_target_response(sender, transport)
                    .await?;
            }
            _ => {
                unimplemented!("Received unimplemented client message: {:?}", message);
            }
        },
        InternalMessage::LocalMessage { message } => {
            unimplemented!("Received local message: {:?}", message)
        }
    };
    Ok(())
}
