use aes_gcm::{
    // C-001 監査対応: Payload を追加して AAD（Additional Authenticated Data）を暗号化操作に渡す
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
// L-10 監査対応: OsRng は OS の乱数生成器を直接使用する暗号学的安全乱数生成器。
// thread_rng() はスレッドローカルな PRNG であり初期化コストが低い一方、
// OsRng はエントロピー源として OS（getrandom syscall）を直接使用するため、
// 暗号鍵やノンスの生成には OsRng が適切である。
use rand::rngs::OsRng;
use rand::RngCore;

use crate::error::EncryptionError;

// HIGH-001 監査対応: 戻り値を必ず使用するよう must_use を付与する
#[must_use]
pub fn generate_aes_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    // OS の乱数生成器を使用して暗号学的に安全な AES-256 鍵を生成する
    OsRng.fill_bytes(&mut key);
    key
}

/// AES-256-GCM で平文を暗号化する。
/// C-001 監査対応: aad（Additional Authenticated Data）を Payload に含めることで、
/// ciphertext swap attack を防止し NIST SP 800-38D 準拠を達成する。
/// aad には暗号化コンテキスト（namespace やチャンネル ID 等の識別子）を渡す。
pub fn aes_encrypt(key: &[u8; 32], plaintext: &[u8], aad: &[u8]) -> Result<String, EncryptionError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| EncryptionError::EncryptFailed(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    // OS の乱数生成器を使用して暗号学的に安全なノンスを生成する
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // C-001 監査対応: AAD を Payload に含めて暗号化することで認証タグがコンテキストを保証する
    let ciphertext = cipher
        .encrypt(nonce, Payload { msg: plaintext, aad })
        .map_err(|e| EncryptionError::EncryptFailed(e.to_string()))?;

    // ノンス（12バイト）を暗号文の先頭に結合し Base64 エンコードして返す
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(&combined))
}

/// AES-256-GCM で暗号文を復号する。
/// C-001 監査対応: aad（Additional Authenticated Data）を Payload に含めることで、
/// 暗号化時と同一の AAD が指定された場合のみ復号成功となる。
/// AAD が異なる場合は認証タグ検証に失敗し `DecryptFailed` エラーを返す。
pub fn aes_decrypt(key: &[u8; 32], ciphertext: &str, aad: &[u8]) -> Result<Vec<u8>, EncryptionError> {
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

    // C-001 監査対応: AAD を Payload に含めて復号することでコンテキスト認証を実施する
    cipher
        .decrypt(nonce, Payload { msg: encrypted, aad })
        .map_err(|e| EncryptionError::DecryptFailed(e.to_string()))
}


#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // AES-GCM で暗号化・復号のラウンドトリップが正常に動作することを確認する。
    // C-001 監査対応: aad 引数を追加（空バイト列でも動作を確認）
    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = generate_aes_key();
        let plaintext = b"Hello, World!";

        let encrypted = aes_encrypt(&key, plaintext, b"").unwrap();
        let decrypted = aes_decrypt(&key, &encrypted, b"").unwrap();

        assert_eq!(decrypted, plaintext);
    }

    // 異なるキーでの復号が失敗することを確認する。
    #[test]
    fn test_decrypt_with_wrong_key() {
        let key1 = generate_aes_key();
        let key2 = generate_aes_key();
        let plaintext = b"secret data";

        let encrypted = aes_encrypt(&key1, plaintext, b"").unwrap();
        assert!(aes_decrypt(&key2, &encrypted, b"").is_err());
    }

    // 改ざんされた暗号文の復号が失敗することを確認する。
    #[test]
    fn test_decrypt_tampered_ciphertext() {
        let key = generate_aes_key();
        let plaintext = b"tamper test";

        let encrypted = aes_encrypt(&key, plaintext, b"").unwrap();
        let mut bytes = STANDARD.decode(&encrypted).unwrap();
        if let Some(last) = bytes.last_mut() {
            *last ^= 0xFF;
        }
        let tampered = STANDARD.encode(&bytes);
        assert!(aes_decrypt(&key, &tampered, b"").is_err());
    }

    // 無効な Base64 文字列の復号が失敗することを確認する。
    #[test]
    fn test_decrypt_invalid_base64() {
        let key = generate_aes_key();
        assert!(aes_decrypt(&key, "!!!invalid!!!", b"").is_err());
    }

    // C-001 監査対応: AAD 不一致時に復号が失敗することを確認する（ciphertext swap attack 防止）。
    // 異なる AAD で暗号化されたデータは、正しい AAD を使用しても復号できないことを検証する。
    #[test]
    fn test_decrypt_with_wrong_aad_fails() {
        let key = generate_aes_key();
        let plaintext = b"sensitive config value";
        let correct_aad = b"system.auth";
        let wrong_aad = b"system.other";

        // 正しい AAD で暗号化する
        let encrypted = aes_encrypt(&key, plaintext, correct_aad).unwrap();

        // 正しい AAD では復号成功を確認する
        let decrypted = aes_decrypt(&key, &encrypted, correct_aad).unwrap();
        assert_eq!(decrypted, plaintext);

        // 異なる AAD では復号が失敗することを確認する（認証タグ検証失敗）
        assert!(aes_decrypt(&key, &encrypted, wrong_aad).is_err());
    }
}
