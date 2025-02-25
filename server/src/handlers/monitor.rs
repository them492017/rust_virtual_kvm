use std::{sync::Arc, time::Duration};

use chacha20poly1305::ChaCha20Poly1305;
use common::{error::DynError, net::Message};
use tokio::{sync::RwLock, time::interval};

use crate::state::State;

pub async fn monitor_client(
    client_idx: usize,
    state: Arc<RwLock<State<ChaCha20Poly1305>>>,
) -> Result<(), DynError> {
    // TODO: refactor magic numbers
    let mut interval = interval(Duration::from_secs(3));
    let mut fail_count = 0;
    let num_retries = 3;

    loop {
        interval.tick().await;
        let state_reader = state.read().await;
        if let Err(err) = state_reader
            .send_message_to_client(client_idx, Message::Heartbeat)
            .await
        {
            // TODO: things like broken pipe should probably skip retries
            fail_count += 1;
            println!(
                "Failed hearbeats {}/{} for client {}: {}",
                fail_count, num_retries, client_idx, err
            );

            if fail_count >= num_retries {
                let _ = state_reader.mark_disconnected(client_idx).await;
                return Err("Heartbeat failed".into());
            }
        }
    }
}
