pub mod aes;
pub mod error;
pub mod hash;
pub mod rsa;

// Phase B（バッチ再暗号化）完了後、aes_decrypt_with_legacy_fallback を削除し aes_decrypt のみを公開する
pub use aes::{aes_decrypt, aes_encrypt, generate_aes_key};
pub use error::EncryptionError;
pub use hash::{hash_password, verify_password};
pub use rsa::{generate_rsa_key_pair, rsa_decrypt, rsa_encrypt};
