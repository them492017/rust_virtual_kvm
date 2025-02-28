use common::{crypto::Crypto, error::DynError};
use uuid::Uuid;

use crate::client::Client;

#[allow(dead_code)]
pub struct State<T: Crypto> {
    clients: Vec<Client<T>>,
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

    pub fn get_client(&self, client_idx: usize) -> Result<&Client<T>, DynError> {
        if client_idx >= self.clients.len() {
            return Err("Index is out of bounds".into());
        }
        Ok(&self.clients[client_idx])
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
    ) -> Result<(), DynError> {
        if client_idx >= self.clients.len() {
            return Err("Index is out of bounds".into());
        }
        self.clients[client_idx] = new_client;
        Ok(())
    }

    pub fn set_target(&mut self, idx: Option<usize>) -> Result<(), DynError> {
        if let Some(i) = idx {
            if i >= self.clients.len() {
                return Err("Target index is out of bounds".into());
            }
        }
        self.target_idx = idx;
        Ok(())
    }

    pub fn cycle_target(&mut self) -> Result<(), DynError> {
        // TODO: implement this properly
        // the logic for choosing the next target should probably be moved
        if self.clients.is_empty() {
            return Ok(());
        }

        self.set_target(if self.target_idx.is_none() {
            Some(0)
        } else {
            None
        })

        // for i in if let Some(idx) = self.target_idx {
        //     idx..(idx + self.clients.len() - 1)
        // } else {
        //     0..(self.clients.len() - 1)
        // } {
        //     if self.clients[i].blocking_lock().connected {
        //         self.target_idx = Some(i % self.clients.len())
        //     }
        // }
    }

    pub async fn mark_disconnected(&mut self, client_idx: usize) -> Result<(), DynError> {
        if client_idx >= self.clients.len() {
            return Err("Index is out of bounds".into());
        }
        self.clients[client_idx].connected = false;
        Ok(())
    }

    pub async fn mark_disconnected_by_id(&mut self, id: Uuid) -> Result<(), DynError> {
        if let Some(client) = self.get_client_by_id_mut(id) {
            client.connected = false;
            Ok(())
        } else {
            Err(format!("Could not find client with id {id}").into())
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
