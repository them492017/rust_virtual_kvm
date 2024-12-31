use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct InputEvent {
    pub nonce: [u8;12],
    pub encrypted_event: Vec<u8>,
}
