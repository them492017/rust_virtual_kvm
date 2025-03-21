use network::Message;
use tokio::sync::mpsc::{error::SendError, Sender};
use uuid::Uuid;

use crate::{InternalMessage, ServerMessage};

pub struct ClientMessageSender {
    client_id: Uuid,
    sender: Sender<InternalMessage>,
}

impl ClientMessageSender {
    pub fn new(client_id: Uuid, sender: Sender<InternalMessage>) -> Self {
        ClientMessageSender { client_id, sender }
    }

    pub async fn send_client_message(
        &self,
        message: Message,
    ) -> Result<(), SendError<InternalMessage>> {
        self.sender
            .send(InternalMessage::ClientMessage {
                message,
                sender: Some(self.client_id),
            })
            .await
    }

    pub async fn send_server_message(
        &self,
        message: ServerMessage,
    ) -> Result<(), SendError<InternalMessage>> {
        self.sender
            .send(InternalMessage::LocalMessage { message })
            .await
    }
}

impl Clone for ClientMessageSender {
    fn clone(&self) -> Self {
        let client_id = self.client_id;
        let sender = self.sender.clone();
        ClientMessageSender { client_id, sender }
    }
}
