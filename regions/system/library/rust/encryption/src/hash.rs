use crate::error::EncryptionError;

pub fn hash_password(password: &str) -> Result<String, EncryptionError> {
    bcrypt::hash(password, 12).map_err(|e| EncryptionError::HashFailed(e.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, EncryptionError> {
    bcrypt::verify(password, hash).map_err(|e| EncryptionError::HashFailed(e.to_string()))
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
        // bcrypt uses random salt, so hashes should differ
        assert_ne!(hash1, hash2);
    }
}
