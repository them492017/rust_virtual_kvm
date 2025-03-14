use crypto::Crypto;

use crate::{Message, MessageWithNonce, TransportError};

pub trait Transport {
    fn send_message(
        &mut self,
        message: Message,
    ) -> impl std::future::Future<Output = Result<(), TransportError>>;
    fn receive_message(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Message, TransportError>>;
}

pub trait TransportReader {
    fn receive_message(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Message, TransportError>>;
}

pub trait TransportWriter {
    fn send_message(
        &mut self,
        message: Message,
    ) -> impl std::future::Future<Output = Result<(), TransportError>>;
}

pub fn decrypt_and_deserialise_message<T: Crypto>(
    bytes: &[u8],
    key: &Option<T>,
) -> Result<Message, TransportError> {
    let message_with_nonce: MessageWithNonce = bincode::deserialize(bytes)?;

    let decrypted = if let Some(key) = &key {
        key.decrypt(message_with_nonce.message, message_with_nonce.nonce.into())?
    } else {
        message_with_nonce.message
    };

    Ok(bincode::deserialize::<Message>(&decrypted)?)
}
