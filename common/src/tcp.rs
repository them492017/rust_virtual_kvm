use std::{
    io::{Read, Write},
    net::TcpStream,
};

use chacha20poly1305::Nonce;

use crate::{
    crypto::Crypto,
    net::{Message, MessageWithNonce},
    transport::{decrypt_and_deserialise_message, print_debug_bytes},
};
use crate::{error::DynError, transport::Transport};

const HEADER_LEN: usize = 4;
const BUFFER_LEN: usize = 256;

pub struct TcpTransport<T: Crypto> {
    socket: TcpStream,
    pub key: Option<T>,
    pub curr: Vec<u8>,
}

impl<T: Crypto> TcpTransport<T> {
    pub fn new(socket: TcpStream) -> Self {
        TcpTransport {
            socket,
            key: None,
            curr: Vec::new(),
        }
    }

    fn extract_msg_len(&self) -> Result<usize, DynError> {
        let prefix_bytes: [u8; HEADER_LEN] = self.curr[..HEADER_LEN].try_into()?;
        Ok(u32::from_le_bytes(prefix_bytes).try_into()?)
    }
}

impl<T: Crypto> Transport for TcpTransport<T> {
    fn send_message(&mut self, message: Message) -> Result<(), DynError> {
        println!("Sending message {}", message);
        let encoded_message: Vec<u8> = bincode::serialize(&message)?;

        println!("Decrypted bytes");
        print_debug_bytes(&encoded_message);
        println!("======================");

        let (encrypted, nonce) = if let Some(encryptor) = &self.key {
            encryptor.encrypt(encoded_message)?
        } else {
            (encoded_message, Nonce::default())
        };

        println!("Encrypted bytes");
        print_debug_bytes(&encrypted);
        println!("======================");

        let message_with_nonce = MessageWithNonce::new(encrypted, nonce);
        let encoded_with_nonce: Vec<u8> = bincode::serialize(&message_with_nonce)?;

        let message_len = encoded_with_nonce.len() as u32;
        let final_message: Vec<u8> = message_len
            .to_le_bytes()
            .into_iter()
            .chain(encoded_with_nonce)
            .collect();

        self.socket.write_all(&final_message)?;
        Ok(())
    }

    fn receive_message(&mut self) -> Result<Message, DynError> {
        let mut message_len: Option<usize> = None;

        loop {
            let mut buf = [0; BUFFER_LEN];
            println!("reading!!!");
            let bytes_read = self.socket.read(&mut buf)?;
            print_debug_bytes(&buf);

            if bytes_read == 0 {
                return Err("Connection closed".into());
            }

            self.curr.extend_from_slice(&buf[..bytes_read]);

            if message_len.is_none() && self.curr.len() >= HEADER_LEN {
                message_len = Some(self.extract_msg_len()?);
                self.curr.drain(..HEADER_LEN);
            }

            if let Some(len) = message_len {
                if self.curr.len() >= len {
                    // TODO: fix errors with short circuiting
                    let message = decrypt_and_deserialise_message(&self.curr[..len], &self.key)?;
                    self.curr.drain(..len);
                    return Ok(message);
                }
            }
        }
    }
}
