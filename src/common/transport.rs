use super::{
    crypto::Crypto,
    error::DynError,
    net::{Message, MessageWithNonce},
};

pub trait Transport {
    fn send_message(
        &mut self,
        message: Message,
    ) -> impl std::future::Future<Output = Result<(), DynError>>;
    fn receive_message(&mut self) -> impl std::future::Future<Output = Result<Message, DynError>>;
}

pub trait TransportReader {
    fn receive_message(&mut self) -> impl std::future::Future<Output = Result<Message, DynError>>;
}

pub trait TransportWriter {
    fn send_message(
        &mut self,
        message: Message,
    ) -> impl std::future::Future<Output = Result<(), DynError>>;
}

pub fn decrypt_and_deserialise_message<T: Crypto>(
    bytes: &[u8],
    key: &Option<T>,
) -> Result<Message, DynError> {
    let message_with_nonce: MessageWithNonce = bincode::deserialize(bytes)?;

    let decrypted = if let Some(key) = &key {
        key.decrypt(message_with_nonce.message, message_with_nonce.nonce.into())?
    } else {
        message_with_nonce.message
    };

    Ok(bincode::deserialize::<Message>(&decrypted)?)
}

pub fn print_debug_bytes(data: &[u8]) {
    const BYTES_PER_ROW: usize = 16;

    for (i, chunk) in data.chunks(BYTES_PER_ROW).enumerate() {
        // Print the byte index in memory
        print!("{:04x}  ", i * BYTES_PER_ROW);

        // Print hex values
        for byte in chunk {
            print!("{:02x} ", byte);
        }

        // Align ASCII section
        for _ in 0..(BYTES_PER_ROW - chunk.len()) {
            print!("   ");
        }

        // Print ASCII representation
        print!(" ");

        for byte in chunk {
            let c = if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '.'
            };
            print!("{}", c);
        }

        println!();
    }
}
