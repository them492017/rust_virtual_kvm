use std::net::SocketAddr;

use chacha20poly1305::ChaCha20Poly1305;
use common::{
    error::DynError, net::Message, transport::AsyncTransport, udp2::TargetlessUdpTransport,
};
use tokio::{net::UdpSocket, sync::mpsc};

use crate::{client::Client, server_message::ServerMessage, state::State};

#[derive(Debug)]
pub enum InternalMessage {
    ClientMessage { message: Message },
    LocalMessage { message: ServerMessage },
}

// TODO: at some point should handle batches of events, not just single events
pub async fn event_processor(
    server_addr: SocketAddr,
    mut device_message_receiver: mpsc::Receiver<InternalMessage>,
    mut client_message_receiver: mpsc::Receiver<InternalMessage>,
    mut client_receiver: mpsc::Receiver<Client<ChaCha20Poly1305>>,
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
    let mut transport: TargetlessUdpTransport<ChaCha20Poly1305> =
        TargetlessUdpTransport::new(socket, server_addr);

    loop {
        tokio::select! {
            msg = device_message_receiver.recv() => {
                match msg {
                    Some(InternalMessage::ClientMessage { message }) => {
                        // send input event to correct client over udp
                        match &message {
                            Message::InputEvent { .. } => if let Some(target) = state.get_target() {
                                {
                                    transport.set_address(target.address);
                                    transport.set_key(target.key.clone());
                                }
                                transport
                                    .send_message(message)
                                    .await?;
                            },
                            _ => {
                                // TODO: send over tcp
                                unimplemented!("TCP sending / non-input event processing is unimplemented");
                            },
                        }
                    }
                    Some(InternalMessage::LocalMessage { message }) => {
                        // forward special event to client handler to be sent over tcp
                        // TODO: probably need to pattern match on the type of special event
                        println!("Local message: {:?}", message);
                        match &message {
                            ServerMessage::ClientDisconnect { id } => {
                                println!("Client {} disconnected", id);
                                // TODO: sending this makes no sense
                                state
                                    .get_client_by_id(*id)
                                    .expect("Client with given id should exist")
                                    .message_sender
                                    .clone()
                                    .send(Message::Heartbeat)
                                    .await?;
                            },
                            _ => {
                                unimplemented!("TCP processing is unimplemented");
                            },
                        }
                    }
                    None => {
                        println!("Event processor receiver was closed");
                        return Err("Event processor receiver was closed".into());
                    }
                }
            },
            msg = client_message_receiver.recv() => {
                match msg {
                    Some(InternalMessage::ClientMessage { message }) => {
                        match &message {
                            Message::Heartbeat => {
                                println!("Received heartbeat from client");
                            },
                            _ => {
                                unimplemented!("Received unimplemented client message: {:?}", message);
                            },
                        }
                    },
                    Some(InternalMessage::LocalMessage { message }) => {
                        unimplemented!("Received client message: {:?}", message)
                    },
                    None => {
                        println!("Client message receiver was closed");
                        return Err("Client message receiver was closed".into());
                    }
                }
            },
            client = client_receiver.recv() => {
                match client {
                    Some(c) => {
                        state.add_client(c);
                    },
                    None => {
                        println!("Client receiver was closed");
                        return Err("Client receiver was closed".into());
                    }
                }
            },
        }
    }
}
