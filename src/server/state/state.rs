use uuid::Uuid;

use crate::{common::crypto::Crypto, server::client::Client};

use super::error::StateHandlerError;

#[allow(dead_code)]
pub struct State<T: Crypto> {
    pub clients: Vec<Client<T>>,
    pub clipboard_contents: Option<String>,
    target_idx: Option<usize>,
}

#[allow(dead_code)]
impl<T: Crypto> State<T> {
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

impl<T: Crypto> Default for State<T> {
    fn default() -> Self {
        State {
            clients: Vec::new(),
            clipboard_contents: None,
            target_idx: None,
        }
    }
}

#[cfg(test)]
pub mod test {
    use chacha20poly1305::ChaCha20Poly1305;
    use tokio::sync::mpsc;

    use crate::{common::net::Message, server::client::test::test_client_fixture};

    use super::*;

    pub fn test_state_fixture(
        client_channels: Vec<mpsc::Sender<Message>>,
        target_idx: Option<usize>,
    ) -> State<ChaCha20Poly1305> {
        let mut state = State::default();
        client_channels.into_iter().for_each(|channel| {
            state.add_client(test_client_fixture(channel));
        });
        state.target_idx = target_idx;
        state
    }
}
