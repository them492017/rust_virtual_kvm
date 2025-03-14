use chacha20poly1305::{
    aead::{Aead, OsRng},
    AeadCore, ChaCha20Poly1305, Nonce,
};

use crate::{Crypto, Decryptor, EncryptionError, Encryptor};

impl Encryptor for ChaCha20Poly1305 {
    fn encrypt(&self, bytes: Vec<u8>) -> Result<(Vec<u8>, Nonce), EncryptionError> {
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        if let Ok(encrypted) = Aead::encrypt(self, &nonce, bytes.as_slice()) {
            Ok((encrypted, nonce))
        } else {
            Err(EncryptionError::EncryptionError)
        }
    }
}

impl Decryptor for ChaCha20Poly1305 {
    fn decrypt(&self, bytes: Vec<u8>, nonce: Nonce) -> Result<Vec<u8>, EncryptionError> {
        if let Ok(result) = Aead::decrypt(self, &nonce, bytes.as_slice()) {
            Ok(result)
        } else {
            Err(EncryptionError::DecryptionError)
        }
    }
}

impl Crypto for ChaCha20Poly1305 {}
