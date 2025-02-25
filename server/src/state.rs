use std::sync::Arc;

use common::{crypto::Crypto, error::DynError, net::Message, transport::AsyncTransport};
use tokio::sync::Mutex;

use crate::client::Client;

#[allow(dead_code)]
pub struct State<T: Crypto> {
    clients: Vec<Arc<Mutex<Client<T>>>>,
    pub clipboard_contents: Option<String>,
    target_idx: Option<usize>,
}

#[allow(dead_code)]
impl<T: Crypto> State<T> {
    pub fn add_client(&mut self, client: Client<T>) -> usize {
        self.clients.push(Arc::new(Mutex::new(client)));
        self.clients.len() - 1
    }

    pub fn get_target(&self) -> Option<Arc<Mutex<Client<T>>>> {
        self.target_idx.map(|idx| self.clients[idx].clone())
    }

    pub fn get_client(&self, client_idx: usize) -> Result<Arc<Mutex<Client<T>>>, DynError> {
        if client_idx >= self.clients.len() {
            return Err("Index is out of bounds".into());
        }
        Ok(self.clients[client_idx].clone())
    }

    pub async fn update_client(
        &mut self,
        client_idx: usize,
        new_client: Client<T>,
    ) -> Result<(), DynError> {
        if client_idx >= self.clients.len() {
            return Err("Index is out of bounds".into());
        }
        {
            let mut client_lock = self.clients[client_idx].lock().await;
            *client_lock = new_client;
        }
        Ok(())
    }

    pub fn set_target(&mut self, idx: Option<usize>) -> Result<(), DynError> {
        if let Some(i) = idx {
            if i >= self.clients.len() {
                return Err("Target index is out of bounds".into());
            }
        } else {
            // TODO: need to grab physical devices
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

    pub async fn mark_disconnected(&self, client_idx: usize) -> Result<(), DynError> {
        if client_idx >= self.clients.len() {
            return Err("Index is out of bounds".into());
        }
        {
            let mut client_lock = self.clients[client_idx].lock().await;
            client_lock.connected = false;
        }
        Ok(())
    }

    pub async fn send_message_to_client(
        &self,
        client_idx: usize,
        message: Message,
    ) -> Result<(), DynError> {
        let client_mutex = self.get_client(client_idx)?;
        let mut client_lock = client_mutex.lock().await;
        client_lock.transport.send_message(message).await
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
