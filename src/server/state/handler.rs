use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    common::{crypto::Crypto, error::DynError},
    server::input_event_transport::InputEventTransport,
};

use super::state::State;

impl<T: Crypto + Clone> State<T> {
    pub fn change_target(
        &mut self,
        new_idx: Option<usize>,
        grab_request_sender: &mut broadcast::Sender<bool>,
    ) -> Result<(), DynError> {
        println!("Changing target index to {:?}", new_idx);
        let prev = self.get_target().is_none();
        self.set_target(new_idx)?;
        if let Some(idx) = new_idx {
            self.send_change_target_notification(idx)?;
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

    pub fn cycle_target(
        &mut self,
        grab_request_sender: &mut broadcast::Sender<bool>,
    ) -> Result<(), DynError> {
        let len = self.get_num_clients();
        let prev_idx = self.get_target_idx().unwrap_or(len);

        (0..=len)
            .map(|i| (prev_idx + i + 1) % (len + 1))
            .find(|&idx| {
                idx == len
                    || self
                        .get_client(idx)
                        .map(|client| client.connected)
                        .unwrap_or(false)
            })
            .ok_or("Could not find target to swap to".into())
            .map(|idx| if idx == len { None } else { Some(idx) })
            .and_then(|target_idx| self.change_target(target_idx, grab_request_sender))
    }

    fn send_change_target_notification(&mut self, idx: usize) -> Result<(), DynError> {
        let client = self.get_client_mut(idx)?;
        println!("Sending target change notif to client at index {}", idx);
        client.pending_target_change_responses += 1;
        // TODO: actually send notificatoin
        Ok(())
    }

    pub async fn handle_change_target_response(
        &mut self,
        id: Uuid,
        transport: &mut InputEventTransport,
    ) -> Result<(), DynError> {
        let client = self
            .get_client_by_id_mut(id)
            .ok_or::<DynError>("Not found error".into())?;
        client.pending_target_change_responses -= 1;
        if client.pending_target_change_responses == 0 {
            client.flush_pending_messages(transport).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    mod change_target {
        use futures::FutureExt;
        use tokio::sync::{broadcast, mpsc};

        use crate::{common::net::Message, server::state::state::test::test_state_fixture};

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
            let response = state.change_target(new_target_idx, &mut grab_request_sender);
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
            let response = state.change_target(new_target_idx, &mut grab_request_sender);
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
            let response = state.change_target(new_target_idx, &mut grab_request_sender);
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
            let response = state.change_target(new_target_idx, &mut grab_request_sender);
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
        use tokio::sync::{broadcast, mpsc};

        use crate::{common::net::Message, server::state::state::test::test_state_fixture};

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
            state.cycle_target(&mut grab_request_sender).unwrap();
            tokio::task::yield_now().await;

            // Then
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
            state.cycle_target(&mut grab_request_sender).unwrap();
            tokio::task::yield_now().await;

            // Then
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
            state.cycle_target(&mut grab_request_sender).unwrap();
            tokio::task::yield_now().await;

            // Then
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
            state.cycle_target(&mut grab_request_sender).unwrap();
            tokio::task::yield_now().await;

            // Then
            assert_eq!(state.get_target_idx(), expected_target_idx);
        }
    }

    mod send_change_target_notification {}

    mod handle_change_target_response {}
}
