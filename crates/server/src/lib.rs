use network::Message;
use uuid::Uuid;

pub mod actors;
pub mod client;
pub mod handlers;
pub mod keyboard_state;
pub mod server;
pub mod server_loop;

#[derive(Debug)]
pub enum ServerMessage {
    Cycle,
    ClientDisconnect { id: Uuid },
}

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
