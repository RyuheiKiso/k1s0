use k1s0_encryption::{
    aes_decrypt, aes_encrypt, generate_aes_key, generate_rsa_key_pair, hash_password,
    rsa_decrypt, rsa_encrypt, verify_password, EncryptionError,
};

// ─── AES ────────────────────────────────────────────────────────────────────

#[test]
fn aes_encrypt_decrypt_roundtrip() {
    let key = generate_aes_key();
    let plaintext = b"Hello, World!";

    let encrypted = aes_encrypt(&key, plaintext).unwrap();
    let decrypted = aes_decrypt(&key, &encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn aes_encrypt_decrypt_roundtrip_binary_data() {
    let key = generate_aes_key();
    let plaintext: Vec<u8> = (0..=255).collect();

    let encrypted = aes_encrypt(&key, &plaintext).unwrap();
    let decrypted = aes_decrypt(&key, &encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn aes_decrypt_with_wrong_key_fails() {
    let key1 = generate_aes_key();
    let key2 = generate_aes_key();
    let plaintext = b"secret data";

    let encrypted = aes_encrypt(&key1, plaintext).unwrap();
    let result = aes_decrypt(&key2, &encrypted);

    assert!(result.is_err());
}

#[test]
fn aes_encrypt_decrypt_empty_data() {
    let key = generate_aes_key();
    let plaintext = b"";

    let encrypted = aes_encrypt(&key, plaintext).unwrap();
    let decrypted = aes_decrypt(&key, &encrypted).unwrap();

    assert_eq!(decrypted, plaintext.to_vec());
}

#[test]
fn aes_encrypt_produces_different_ciphertexts_for_same_input() {
    let key = generate_aes_key();
    let plaintext = b"same input";

    let encrypted1 = aes_encrypt(&key, plaintext).unwrap();
    let encrypted2 = aes_encrypt(&key, plaintext).unwrap();

    // Due to random nonce, encryptions of the same plaintext differ
    assert_ne!(encrypted1, encrypted2);
}

#[test]
fn aes_decrypt_invalid_base64_returns_error() {
    let key = generate_aes_key();
    let result = aes_decrypt(&key, "!!!not-base64!!!");
    assert!(result.is_err());
}

#[test]
fn aes_decrypt_too_short_ciphertext_returns_error() {
    let key = generate_aes_key();
    // base64 of a few bytes (less than 12-byte nonce)
    let short = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &[1, 2, 3]);
    let result = aes_decrypt(&key, &short);
    assert!(result.is_err());
}

#[test]
fn aes_large_data_roundtrip() {
    let key = generate_aes_key();
    let plaintext = vec![0xABu8; 1_000_000]; // 1 MB

    let encrypted = aes_encrypt(&key, &plaintext).unwrap();
    let decrypted = aes_decrypt(&key, &encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn aes_different_keys_are_unique() {
    let k1 = generate_aes_key();
    let k2 = generate_aes_key();
    assert_ne!(k1, k2);
}

// ─── RSA ────────────────────────────────────────────────────────────────────

#[test]
fn rsa_encrypt_decrypt_roundtrip() {
    let (pub_pem, priv_pem) = generate_rsa_key_pair().unwrap();
    let plaintext = b"hello RSA-OAEP world";

    let ciphertext = rsa_encrypt(&pub_pem, plaintext).unwrap();
    let decrypted = rsa_decrypt(&priv_pem, &ciphertext).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn rsa_decrypt_with_wrong_key_fails() {
    let (pub_pem, _) = generate_rsa_key_pair().unwrap();
    let (_, priv_pem2) = generate_rsa_key_pair().unwrap();

    let ciphertext = rsa_encrypt(&pub_pem, b"secret").unwrap();
    let result = rsa_decrypt(&priv_pem2, &ciphertext);

    assert!(result.is_err());
}

#[test]
fn rsa_key_generation_produces_valid_pem() {
    let (pub_pem, priv_pem) = generate_rsa_key_pair().unwrap();

    assert!(pub_pem.starts_with("-----BEGIN PUBLIC KEY-----"));
    assert!(priv_pem.starts_with("-----BEGIN PRIVATE KEY-----"));
}

#[test]
fn rsa_encrypt_empty_data() {
    let (pub_pem, priv_pem) = generate_rsa_key_pair().unwrap();
    let plaintext = b"";

    let ciphertext = rsa_encrypt(&pub_pem, plaintext).unwrap();
    let decrypted = rsa_decrypt(&priv_pem, &ciphertext).unwrap();

    assert_eq!(decrypted, plaintext.to_vec());
}

#[test]
fn rsa_encrypt_invalid_pem_returns_error() {
    let result = rsa_encrypt("not-a-valid-pem", b"data");
    assert!(matches!(result, Err(EncryptionError::RsaEncryptFailed(_))));
}

#[test]
fn rsa_decrypt_invalid_pem_returns_error() {
    let result = rsa_decrypt("not-a-valid-pem", b"data");
    assert!(matches!(result, Err(EncryptionError::RsaDecryptFailed(_))));
}

#[test]
fn rsa_decrypt_garbage_ciphertext_returns_error() {
    let (_, priv_pem) = generate_rsa_key_pair().unwrap();
    let result = rsa_decrypt(&priv_pem, &[0u8; 256]);
    assert!(result.is_err());
}

#[test]
fn rsa_encrypt_produces_different_ciphertexts() {
    let (pub_pem, _) = generate_rsa_key_pair().unwrap();
    let plaintext = b"same input";

    let c1 = rsa_encrypt(&pub_pem, plaintext).unwrap();
    let c2 = rsa_encrypt(&pub_pem, plaintext).unwrap();

    // OAEP uses random padding, so ciphertexts should differ
    assert_ne!(c1, c2);
}

// ─── Hash (Argon2id) ────────────────────────────────────────────────────────

#[test]
fn hash_password_and_verify_succeeds() {
    let password = "my-secure-password";
    let hashed = hash_password(password).unwrap();
    assert!(verify_password(password, &hashed).unwrap());
}

#[test]
fn hash_verify_wrong_password_fails() {
    let hashed = hash_password("correct-password").unwrap();
    assert!(!verify_password("wrong-password", &hashed).unwrap());
}

#[test]
fn hash_different_passwords_produce_different_hashes() {
    let h1 = hash_password("password-one").unwrap();
    let h2 = hash_password("password-two").unwrap();
    assert_ne!(h1, h2);
}

#[test]
fn hash_same_password_produces_different_hashes_due_to_salt() {
    let password = "same-password";
    let h1 = hash_password(password).unwrap();
    let h2 = hash_password(password).unwrap();
    assert_ne!(h1, h2);
    // But both should verify
    assert!(verify_password(password, &h1).unwrap());
    assert!(verify_password(password, &h2).unwrap());
}

#[test]
fn hash_contains_argon2id_identifier() {
    let hashed = hash_password("test-password").unwrap();
    assert!(hashed.starts_with("$argon2id$"));
}

#[test]
fn hash_empty_password() {
    let hashed = hash_password("").unwrap();
    assert!(verify_password("", &hashed).unwrap());
    assert!(!verify_password("non-empty", &hashed).unwrap());
}

#[test]
fn hash_verify_invalid_hash_string_returns_error() {
    let result = verify_password("password", "not-a-valid-hash");
    assert!(result.is_err());
}

#[test]
fn hash_unicode_password() {
    let password = "p@$$w0rd-\u{1F512}-\u{00E9}\u{00F1}\u{00FC}";
    let hashed = hash_password(password).unwrap();
    assert!(verify_password(password, &hashed).unwrap());
}

// ─── Error Display ──────────────────────────────────────────────────────────

#[test]
fn encryption_error_display() {
    let err = EncryptionError::EncryptFailed("test error".to_string());
    assert_eq!(err.to_string(), "encrypt failed: test error");

    let err = EncryptionError::DecryptFailed("bad data".to_string());
    assert_eq!(err.to_string(), "decrypt failed: bad data");

    let err = EncryptionError::HashFailed("hash broke".to_string());
    assert_eq!(err.to_string(), "hash failed: hash broke");

    let err = EncryptionError::RsaKeyGenerationFailed("keygen err".to_string());
    assert_eq!(err.to_string(), "RSA key generation failed: keygen err");

    let err = EncryptionError::RsaEncryptFailed("enc err".to_string());
    assert_eq!(err.to_string(), "RSA encrypt failed: enc err");

    let err = EncryptionError::RsaDecryptFailed("dec err".to_string());
    assert_eq!(err.to_string(), "RSA decrypt failed: dec err");
}
