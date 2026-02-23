use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::RngCore;

use crate::error::EncryptionError;

pub fn generate_aes_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

pub fn aes_encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<String, EncryptionError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| EncryptionError::EncryptFailed(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| EncryptionError::EncryptFailed(e.to_string()))?;

    // Prepend nonce to ciphertext, then base64 encode
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(&combined))
}

pub fn aes_decrypt(key: &[u8; 32], ciphertext: &str) -> Result<Vec<u8>, EncryptionError> {
    let combined = STANDARD
        .decode(ciphertext)
        .map_err(|e| EncryptionError::DecryptFailed(e.to_string()))?;

    if combined.len() < 12 {
        return Err(EncryptionError::DecryptFailed(
            "ciphertext too short".to_string(),
        ));
    }

    let (nonce_bytes, encrypted) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| EncryptionError::DecryptFailed(e.to_string()))?;

    cipher
        .decrypt(nonce, encrypted)
        .map_err(|e| EncryptionError::DecryptFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = generate_aes_key();
        let plaintext = b"Hello, World!";

        let encrypted = aes_encrypt(&key, plaintext).unwrap();
        let decrypted = aes_decrypt(&key, &encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_with_wrong_key() {
        let key1 = generate_aes_key();
        let key2 = generate_aes_key();
        let plaintext = b"secret data";

        let encrypted = aes_encrypt(&key1, plaintext).unwrap();
        assert!(aes_decrypt(&key2, &encrypted).is_err());
    }

    #[test]
    fn test_decrypt_tampered_ciphertext() {
        let key = generate_aes_key();
        let plaintext = b"tamper test";

        let encrypted = aes_encrypt(&key, plaintext).unwrap();
        let mut bytes = STANDARD.decode(&encrypted).unwrap();
        if let Some(last) = bytes.last_mut() {
            *last ^= 0xFF;
        }
        let tampered = STANDARD.encode(&bytes);
        assert!(aes_decrypt(&key, &tampered).is_err());
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        let key = generate_aes_key();
        assert!(aes_decrypt(&key, "!!!invalid!!!").is_err());
    }
}
