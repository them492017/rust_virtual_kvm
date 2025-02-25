use std::io::ErrorKind;

use chacha20poly1305::{
    aead::{Aead, OsRng},
    AeadCore, ChaCha20Poly1305, Nonce,
};

use crate::error::DynError;

pub trait Crypto: Encryptor + Decryptor {}

pub trait Encryptor {
    fn encrypt(&self, bytes: Vec<u8>) -> Result<(Vec<u8>, Nonce), DynError>;
}

pub trait Decryptor {
    fn decrypt(&self, bytes: Vec<u8>, nonce: Nonce) -> Result<Vec<u8>, DynError>;
}

impl Crypto for ChaCha20Poly1305 {}

impl Encryptor for ChaCha20Poly1305 {
    fn encrypt(&self, bytes: Vec<u8>) -> Result<(Vec<u8>, Nonce), DynError> {
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let encrypted = Aead::encrypt(self, &nonce, bytes.as_slice())
            .map_err(|_| std::io::Error::new(ErrorKind::Other, "Encryption error"))?;

        Ok((encrypted, nonce))
    }
}

impl Decryptor for ChaCha20Poly1305 {
    fn decrypt(&self, bytes: Vec<u8>, nonce: Nonce) -> Result<Vec<u8>, DynError> {
        Ok(Aead::decrypt(self, &nonce, bytes.as_slice())
            .map_err(|_| std::io::Error::new(ErrorKind::Other, "Encryption error"))?)
    }
}
