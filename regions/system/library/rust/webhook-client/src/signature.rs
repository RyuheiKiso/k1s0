use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn generate_signature(secret: &str, body: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(body);
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

pub fn verify_signature(secret: &str, body: &[u8], signature: &str) -> bool {
    let expected = generate_signature(secret, body);
    constant_time_eq(expected.as_bytes(), signature.as_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_roundtrip() {
        let secret = "my-secret-key";
        let body = b"hello world";
        let sig = generate_signature(secret, body);
        assert!(verify_signature(secret, body, &sig));
    }

    #[test]
    fn test_verify_with_wrong_secret() {
        let body = b"hello world";
        let sig = generate_signature("correct-secret", body);
        assert!(!verify_signature("wrong-secret", body, &sig));
    }

    #[test]
    fn test_verify_with_tampered_body() {
        let secret = "my-secret-key";
        let sig = generate_signature(secret, b"original body");
        assert!(!verify_signature(secret, b"tampered body", &sig));
    }

    #[test]
    fn test_signature_is_hex_encoded() {
        let sig = generate_signature("secret", b"data");
        assert!(sig.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(sig.len(), 64); // SHA-256 = 32 bytes = 64 hex chars
    }
}
