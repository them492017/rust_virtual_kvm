use chacha20poly1305::Nonce;
use thiserror::Error;

pub mod chacha;

#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("Encryption error")]
    EncryptionError,
    #[error("Decryption error")]
    DecryptionError,
}

pub trait Crypto: Encryptor + Decryptor {}

pub trait Encryptor: Clone {
    // TODO: remove chacha20poly1305::Nonce from return type
    fn encrypt(&self, bytes: Vec<u8>) -> Result<(Vec<u8>, Nonce), EncryptionError>;
}

pub trait Decryptor: Clone {
    fn decrypt(&self, bytes: Vec<u8>, nonce: Nonce) -> Result<Vec<u8>, EncryptionError>;
}
