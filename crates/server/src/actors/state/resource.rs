use crypto::Crypto;
use network::{input_event::InputEventTransport, Message};
use thiserror::Error;
use tokio::sync::broadcast;
use uuid::Uuid;

use super::client::{Client, ClientConnectionError};

#[derive(Debug, Error)]
pub enum StateHandlerError {
    #[error("Client is disconnected")]
    ClientDisconnected,
    #[error("Client not found")]
    NotFound,
    #[error("Grab request channel send error: {0}")]
    GrabSendError(#[from] tokio::sync::broadcast::error::SendError<bool>),
    #[error("Grab request channel send error: {0}")]
    MessageSendError(#[from] tokio::sync::mpsc::error::SendError<Message>),
    #[error("Client error: {0}")]
    ClientError(#[from] ClientConnectionError),
}

pub struct StateResource<T: Crypto> {
    clients: Vec<Client<T>>,
    pub clipboard_contents: Option<String>,
    target_idx: Option<usize>,
}

impl<T: Crypto> Default for StateResource<T> {
    fn default() -> Self {
        StateResource {
            clients: Vec::new(),
            clipboard_contents: None,
            target_idx: None,
        }
    }
}

impl<T: Crypto> StateResource<T> {
    pub fn add_client(&mut self, client: Client<T>) -> usize {
        self.clients.push(client);
        self.clients.len() - 1
    }

    pub fn get_target(&self) -> Option<&Client<T>> {
        self.target_idx.map(|idx| &self.clients[idx])
    }

    pub fn get_target_mut(&mut self) -> Option<&mut Client<T>> {
        self.target_idx.map(|idx| &mut self.clients[idx])
    }

    pub fn get_target_idx(&self) -> Option<usize> {
        self.target_idx
    }

    pub fn get_num_clients(&self) -> usize {
        self.clients.len()
    }

    pub fn get_client(&self, client_idx: usize) -> Result<&Client<T>, StateHandlerError> {
        if client_idx >= self.clients.len() {
            return Err(StateHandlerError::NotFound);
        }
        Ok(&self.clients[client_idx])
    }

    pub fn get_client_mut(
        &mut self,
        client_idx: usize,
    ) -> Result<&mut Client<T>, StateHandlerError> {
        if client_idx >= self.clients.len() {
            return Err(StateHandlerError::NotFound);
        }
        Ok(&mut self.clients[client_idx])
    }

    pub fn get_client_by_id(&self, id: Uuid) -> Option<&Client<T>> {
        self.clients.iter().find(|client| client.id == id)
    }

    pub fn get_client_by_id_mut(&mut self, id: Uuid) -> Option<&mut Client<T>> {
        self.clients.iter_mut().find(|client| client.id == id)
    }

    pub async fn update_client(
        &mut self,
        client_idx: usize,
        new_client: Client<T>,
    ) -> Result<(), StateHandlerError> {
        if client_idx >= self.clients.len() {
            return Err(StateHandlerError::NotFound);
        }
        self.clients[client_idx] = new_client;
        Ok(())
    }

    pub fn set_target(&mut self, idx: Option<usize>) -> Result<(), StateHandlerError> {
        if let Some(i) = idx {
            if i >= self.clients.len() {
                return Err(StateHandlerError::NotFound);
            }
        }
        self.target_idx = idx;
        Ok(())
    }

    pub async fn mark_disconnected(&mut self, client_idx: usize) -> Result<(), StateHandlerError> {
        if client_idx >= self.clients.len() {
            return Err(StateHandlerError::NotFound);
        }
        self.clients[client_idx].connected = false;
        Ok(())
    }

    pub async fn mark_disconnected_by_id(&mut self, id: Uuid) -> Result<(), StateHandlerError> {
        if let Some(client) = self.get_client_by_id_mut(id) {
            client.connected = false;
            Ok(())
        } else {
            Err(StateHandlerError::NotFound)
        }
    }
}

// TODO: maybe make a trait for this
impl<T: Crypto + Clone> StateResource<T> {
    pub async fn change_target(
        &mut self,
        new_idx: Option<usize>,
        grab_request_sender: &mut broadcast::Sender<bool>,
    ) -> Result<(), StateHandlerError> {
        println!("Changing target index to {:?}", new_idx);
        let prev = self.get_target().is_none();
        let prev_idx = self.get_target_idx();
        self.set_target(new_idx)?;
        if let Some(idx) = prev_idx {
            match self.send_change_target_notification(idx).await {
                Ok(()) | Err(StateHandlerError::ClientDisconnected) => {}
                Err(err) => return Err(err),
            }
        }
        let curr = self.get_target().is_none();
        if prev && !curr {
            // should grab
            grab_request_sender.send(true)?;
        }
        if !prev && curr {
            // should ungrab
            grab_request_sender.send(false)?;
        }
        Ok(())
    }

    pub async fn cycle_target(
        &mut self,
        grab_request_sender: &mut broadcast::Sender<bool>,
    ) -> Result<(), StateHandlerError> {
        let len = self.get_num_clients();
        let prev_idx = self.get_target_idx().unwrap_or(len);

        let target_idx = (0..=len)
            .map(|i| (prev_idx + i + 1) % (len + 1))
            .find(|&idx| {
                idx == len
                    || self
                        .get_client(idx)
                        .map(|client| client.connected)
                        .unwrap_or(false)
            })
            .ok_or(StateHandlerError::NotFound)
            .map(|idx| if idx == len { None } else { Some(idx) })?;

        self.change_target(target_idx, grab_request_sender).await
    }

    async fn send_change_target_notification(
        &mut self,
        idx: usize,
    ) -> Result<(), StateHandlerError> {
        let client = self.get_client_mut(idx)?;

        if !client.connected {
            return Err(StateHandlerError::ClientDisconnected);
        }

        println!("Sending target change notif to client at index {}", idx);
        client.pending_target_change_responses += 1;
        client
            .message_sender
            .send(Message::TargetChangeNotification)
            .await?;
        Ok(())
    }

    pub async fn handle_change_target_response(
        &mut self,
        id: Uuid,
        transport: &mut InputEventTransport,
    ) -> Result<(), StateHandlerError> {
        let client = self
            .get_client_by_id_mut(id)
            .ok_or(StateHandlerError::NotFound)?;
        client.pending_target_change_responses -= 1;
        if client.pending_target_change_responses == 0 {
            client.flush_pending_messages(transport).await?;
        }
        Ok(())
    }

    pub async fn disconnect_client(
        &mut self,
        id: Uuid,
        grab_request_sender: &mut broadcast::Sender<bool>,
    ) -> Result<(), StateHandlerError> {
        println!("Client {} disconnected", id);
        self.get_client_by_id_mut(id)
            .expect("Client with given id should exist")
            .connected = false;
        // swap target to server if target just disconnected
        if !self.get_target().map(|tgt| tgt.connected).unwrap_or(false) {
            self.change_target(None, grab_request_sender).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod test {
    pub mod fixtures {
        use chacha20poly1305::ChaCha20Poly1305;
        use network::Message;
        use tokio::sync::mpsc;

        use crate::actors::state::{client::test::test_client_fixture, resource::StateResource};

        pub fn test_state_fixture(
            client_channels: Vec<mpsc::Sender<Message>>,
            target_idx: Option<usize>,
        ) -> StateResource<ChaCha20Poly1305> {
            let mut state = StateResource::default();
            client_channels.into_iter().for_each(|channel| {
                state.add_client(test_client_fixture(channel));
            });
            state.target_idx = target_idx;
            state
        }
    }

    mod change_target {
        use futures::FutureExt;
        use network::Message;
        use tokio::sync::{broadcast, mpsc};

        use crate::actors::state::resource::test::fixtures::test_state_fixture;

        #[tokio::test]
        async fn given_valid_target_idx_should_change_target() {
            // Given
            let old_target_idx = None;
            let new_target_idx = Some(1);
            let (client_message_senders, _client_message_receivers): (Vec<_>, Vec<_>) =
                (0..3).map(|_| mpsc::channel(10)).unzip();
            let (mut grab_request_sender, _grab_request_receiver) = broadcast::channel(10);
            let mut state = test_state_fixture(client_message_senders, old_target_idx);

            // When
            let response = state
                .change_target(new_target_idx, &mut grab_request_sender)
                .await;
            tokio::task::yield_now().await;

            // Then
            assert!(response.is_ok());
            assert_eq!(state.get_target_idx(), new_target_idx);
        }

        #[tokio::test]
        async fn given_no_current_target_should_issue_grab_request_on_change_and_no_notification() {
            // Given
            let old_target_idx = None;
            let new_target_idx = Some(1);
            let (client_message_senders, mut client_message_receivers): (Vec<_>, Vec<_>) =
                (0..3).map(|_| mpsc::channel(10)).unzip();
            let (mut grab_request_sender, mut grab_request_receiver) = broadcast::channel(10);
            let mut state = test_state_fixture(client_message_senders, old_target_idx);

            // When
            let response = state
                .change_target(new_target_idx, &mut grab_request_sender)
                .await;
            tokio::task::yield_now().await;

            // Then
            assert!(response.is_ok());
            assert_eq!(state.get_target_idx(), new_target_idx);
            client_message_receivers.iter_mut().for_each(|rx| {
                let msg = rx.recv().now_or_never();
                assert_eq!(msg, None);
            });
            let grab_request = grab_request_receiver
                .recv()
                .now_or_never()
                .expect("No grab request received")
                .expect("Grab request channel was closed");
            assert!(grab_request)
        }

        #[tokio::test]
        async fn given_some_current_target_should_issue_notification_on_change_and_no_grab_request()
        {
            // Given
            let old_target_idx = Some(1);
            let new_target_idx = Some(2);
            let (client_message_senders, mut client_message_receivers): (Vec<_>, Vec<_>) =
                (0..3).map(|_| mpsc::channel(10)).unzip();
            let (mut grab_request_sender, mut grab_request_receiver) = broadcast::channel(10);
            let mut state = test_state_fixture(client_message_senders, old_target_idx);

            // When
            let response = state
                .change_target(new_target_idx, &mut grab_request_sender)
                .await;
            tokio::task::yield_now().await;

            // Then
            assert!(response.is_ok());
            assert_eq!(state.get_target_idx(), new_target_idx);
            let client_notif = client_message_receivers[old_target_idx.unwrap()]
                .recv()
                .now_or_never()
                .expect("No client message received")
                .expect("Client message channel was closed");
            assert_eq!(client_notif, Message::TargetChangeNotification);
            let grab_request = grab_request_receiver.recv().now_or_never();
            assert_eq!(grab_request, None)
        }

        #[tokio::test]
        async fn given_some_current_target_and_changing_to_no_target_should_issue_notification_on_change_and_ungrab_request(
        ) {
            // Given
            let old_target_idx = Some(1);
            let new_target_idx = None;
            let (client_message_senders, mut client_message_receivers): (Vec<_>, Vec<_>) =
                (0..3).map(|_| mpsc::channel(10)).unzip();
            let (mut grab_request_sender, mut grab_request_receiver) = broadcast::channel(10);
            let mut state = test_state_fixture(client_message_senders, old_target_idx);

            // When
            let response = state
                .change_target(new_target_idx, &mut grab_request_sender)
                .await;
            tokio::task::yield_now().await;

            // Then
            assert!(response.is_ok());
            assert_eq!(state.get_target_idx(), new_target_idx);
            let client_notif = client_message_receivers[old_target_idx.unwrap()]
                .recv()
                .now_or_never()
                .expect("No client message received")
                .expect("Client message channel was closed");
            assert_eq!(client_notif, Message::TargetChangeNotification);
            let grab_request = grab_request_receiver
                .recv()
                .now_or_never()
                .expect("No grab request received")
                .expect("Grab request channel was closed");
            assert!(!grab_request);
        }
    }

    mod cycle_target {
        use network::Message;
        use tokio::sync::{broadcast, mpsc};

        use crate::actors::state::resource::test::fixtures::test_state_fixture;

        #[tokio::test]
        async fn cycle_from_none() {
            // Given
            let (client_message_senders, _client_message_receivers): (Vec<_>, Vec<_>) =
                (0..3).map(|_| mpsc::channel(10)).unzip();
            let (mut grab_request_sender, _grab_request_receiver) = broadcast::channel(10);
            let target_idx = None;
            let mut state = test_state_fixture(client_message_senders, target_idx);

            let expected_target_idx = Some(0);

            // When
            let response = state.cycle_target(&mut grab_request_sender).await;
            tokio::task::yield_now().await;

            // Then
            assert!(response.is_ok());
            assert_eq!(state.get_target_idx(), expected_target_idx);
        }

        #[tokio::test]
        async fn cycle_from_first() {
            // Given
            let (client_message_senders, _client_message_receivers): (Vec<_>, Vec<_>) =
                (0..3).map(|_| mpsc::channel(10)).unzip();
            let (mut grab_request_sender, _grab_request_receiver) = broadcast::channel(10);
            let target_idx = Some(0);
            let mut state = test_state_fixture(client_message_senders, target_idx);

            let expected_target_idx = Some(1);

            // When
            let response = state.cycle_target(&mut grab_request_sender).await;
            tokio::task::yield_now().await;

            // Then
            assert!(response.is_ok());
            assert_eq!(state.get_target_idx(), expected_target_idx);
        }

        #[tokio::test]
        async fn cycle_from_last() {
            // Given
            let (client_message_senders, _client_message_receivers): (Vec<_>, Vec<_>) =
                (0..3).map(|_| mpsc::channel(10)).unzip();
            let (mut grab_request_sender, _grab_request_receiver) = broadcast::channel(10);
            let target_idx = Some(2);
            let mut state = test_state_fixture(client_message_senders, target_idx);

            let expected_target_idx = None;

            // When
            let response = state.cycle_target(&mut grab_request_sender).await;
            tokio::task::yield_now().await;

            // Then
            assert!(response.is_ok());
            assert_eq!(state.get_target_idx(), expected_target_idx);
        }

        #[tokio::test]
        async fn cycle_with_no_clients() {
            // Given
            let (client_message_senders, _client_message_receivers): (Vec<_>, Vec<Message>) =
                (Vec::new(), Vec::new());
            let (mut grab_request_sender, _grab_request_receiver) = broadcast::channel(10);
            let target_idx = None;
            let mut state = test_state_fixture(client_message_senders, target_idx);

            let expected_target_idx = None;

            // When
            let response = state.cycle_target(&mut grab_request_sender).await;
            tokio::task::yield_now().await;

            // Then
            assert!(response.is_ok());
            assert_eq!(state.get_target_idx(), expected_target_idx);
        }
    }

    mod send_change_target_notification {}

    mod handle_change_target_response {}
}
