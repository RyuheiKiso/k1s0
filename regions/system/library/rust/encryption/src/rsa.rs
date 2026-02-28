use crate::error::EncryptionError;
use rand::rngs::OsRng;
use rsa::{
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding},
    Oaep, RsaPrivateKey, RsaPublicKey,
};
use sha2::Sha256;

const KEY_BITS: usize = 2048;

pub fn generate_rsa_key_pair() -> Result<(String, String), EncryptionError> {
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, KEY_BITS)
        .map_err(|e| EncryptionError::RsaKeyGenerationFailed(e.to_string()))?;
    let public_key = RsaPublicKey::from(&private_key);

    let public_pem = public_key
        .to_public_key_pem(LineEnding::LF)
        .map_err(|e| EncryptionError::RsaKeyGenerationFailed(e.to_string()))?;
    let private_pem = private_key
        .to_pkcs8_pem(LineEnding::LF)
        .map_err(|e| EncryptionError::RsaKeyGenerationFailed(e.to_string()))?;

    Ok((public_pem, private_pem.to_string()))
}

pub fn rsa_encrypt(public_key_pem: &str, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    let public_key = RsaPublicKey::from_public_key_pem(public_key_pem)
        .map_err(|e| EncryptionError::RsaEncryptFailed(e.to_string()))?;
    let mut rng = OsRng;
    let padding = Oaep::new::<Sha256>();
    public_key
        .encrypt(&mut rng, padding, plaintext)
        .map_err(|e| EncryptionError::RsaEncryptFailed(e.to_string()))
}

pub fn rsa_decrypt(private_key_pem: &str, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    let private_key = RsaPrivateKey::from_pkcs8_pem(private_key_pem)
        .map_err(|e| EncryptionError::RsaDecryptFailed(e.to_string()))?;
    let padding = Oaep::new::<Sha256>();
    private_key
        .decrypt(padding, ciphertext)
        .map_err(|e| EncryptionError::RsaDecryptFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsa_roundtrip() {
        let (pub_pem, priv_pem) = generate_rsa_key_pair().unwrap();
        let plaintext = b"hello RSA-OAEP world";
        let ciphertext = rsa_encrypt(&pub_pem, plaintext).unwrap();
        let decrypted = rsa_decrypt(&priv_pem, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_rsa_wrong_key_fails() {
        let (pub_pem, _) = generate_rsa_key_pair().unwrap();
        let (_, priv_pem2) = generate_rsa_key_pair().unwrap();
        let ciphertext = rsa_encrypt(&pub_pem, b"secret").unwrap();
        let result = rsa_decrypt(&priv_pem2, &ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_rsa_encrypt_invalid_pem() {
        let result = rsa_encrypt("not-a-valid-pem", b"data");
        assert!(matches!(result, Err(EncryptionError::RsaEncryptFailed(_))));
    }

    #[test]
    fn test_rsa_decrypt_invalid_pem() {
        let result = rsa_decrypt("not-a-valid-pem", b"data");
        assert!(matches!(result, Err(EncryptionError::RsaDecryptFailed(_))));
    }
}
