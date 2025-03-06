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
        println!("Cycling target");
        let prev_idx = self.get_target_idx().unwrap_or(0);
        let len = self.get_num_clients();
        for i in 0..(len + 1) {
            // TODO: this shouldn't start at 0
            println!("Checking {i}");
            let idx = (prev_idx + i) % (len + 1);
            if idx == len {
                self.change_target(None, grab_request_sender)?;
                return Ok(());
            }
            if self
                .get_client(idx)
                .map(|client| client.connected)
                .unwrap_or(false)
            {
                self.change_target(Some(idx), grab_request_sender)?;
                return Ok(());
            }
        }
        Err("Could not find target to swap to")? // TODO: need to catch this
    }

    fn send_change_target_notification(&mut self, idx: usize) -> Result<(), DynError> {
        let client = self.get_client_mut(idx)?;
        client.pending_target_change_responses += 1;
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
