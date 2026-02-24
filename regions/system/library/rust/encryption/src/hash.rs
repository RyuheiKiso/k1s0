use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};

use crate::error::EncryptionError;

/// Argon2id recommended parameters:
/// memory = 19456 KiB, iterations = 2, parallelism = 1
fn argon2_instance() -> Argon2<'static> {
    let params = Params::new(19456, 2, 1, None).expect("valid argon2 params");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

pub fn hash_password(password: &str) -> Result<String, EncryptionError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = argon2_instance();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| EncryptionError::HashFailed(e.to_string()))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, EncryptionError> {
    let parsed =
        PasswordHash::new(hash).map_err(|e| EncryptionError::HashFailed(e.to_string()))?;
    let argon2 = argon2_instance();
    Ok(argon2.verify_password(password.as_bytes(), &parsed).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_correct_password() {
        let password = "my-secure-password";
        let hashed = hash_password(password).unwrap();
        assert!(verify_password(password, &hashed).unwrap());
    }

    #[test]
    fn test_verify_wrong_password() {
        let hashed = hash_password("correct-password").unwrap();
        assert!(!verify_password("wrong-password", &hashed).unwrap());
    }

    #[test]
    fn test_hash_produces_different_outputs() {
        let password = "same-password";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();
        // argon2id uses random salt, so hashes should differ
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_contains_argon2id_identifier() {
        let hashed = hash_password("test-password").unwrap();
        assert!(hashed.starts_with("$argon2id$"));
    }
}
