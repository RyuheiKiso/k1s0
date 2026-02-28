use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("encrypt failed: {0}")]
    EncryptFailed(String),
    #[error("decrypt failed: {0}")]
    DecryptFailed(String),
    #[error("hash failed: {0}")]
    HashFailed(String),
    #[error("RSA key generation failed: {0}")]
    RsaKeyGenerationFailed(String),
    #[error("RSA encrypt failed: {0}")]
    RsaEncryptFailed(String),
    #[error("RSA decrypt failed: {0}")]
    RsaDecryptFailed(String),
}
