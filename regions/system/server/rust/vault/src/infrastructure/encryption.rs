use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};

/// MasterKey は vault の暗号化/復号化に使用するマスター鍵を保持する。
/// AES-256-GCM を使用し、各暗号化操作で一意の 12 バイト nonce を生成する。
#[derive(Debug)]
pub struct MasterKey {
    key: Key<Aes256Gcm>,
}

impl MasterKey {
    /// 環境変数 `VAULT_MASTER_KEY` から hex エンコードされた 32 バイト鍵を読み込む。
    /// 本番環境ではゼロ鍵での起動を拒否する。
    pub fn from_env() -> anyhow::Result<Self> {
        let environment = std::env::var("APP_ENVIRONMENT").unwrap_or_default();
        let key_hex = match std::env::var("VAULT_MASTER_KEY") {
            Ok(key) => key,
            Err(_) => {
                // 本番・ステージング環境ではゼロ鍵を許可しない
                // 大文字小文字を無視して比較（"Production", "PRODUCTION" 等のバイパスを防止）
                let env_lower = environment.to_lowercase();
                if env_lower == "production" || env_lower == "staging" {
                    return Err(anyhow::anyhow!(
                        "VAULT_MASTER_KEY must be set in {} environment",
                        environment
                    ));
                }
                tracing::warn!("VAULT_MASTER_KEY not set, using zero key (development only)");
                "0".repeat(64) // 32 bytes hex = 64 chars, dev default
            }
        };
        let key_bytes = hex::decode(&key_hex)?;
        if key_bytes.len() != 32 {
            return Err(anyhow::anyhow!(
                "VAULT_MASTER_KEY must be 32 bytes (64 hex chars), got {} bytes",
                key_bytes.len()
            ));
        }
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        Ok(Self { key: *key })
    }

    /// 平文データを AES-256-GCM で暗号化し、(暗号文, nonce) を返す。
    pub fn encrypt(&self, plaintext: &[u8]) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
        let cipher = Aes256Gcm::new(&self.key);
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("encryption failed: {}", e))?;
        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    /// 暗号文と nonce から平文データを復号化する。
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> anyhow::Result<Vec<u8>> {
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(nonce);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("decryption failed: {}", e))?;
        Ok(plaintext)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// 環境変数を操作するテストの並行実行を防ぐためのロック。
    /// std::env::set_var/remove_var はプロセスグローバルなので、
    /// 並行テストで競合すると flaky failure になる。
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn make_test_key() -> MasterKey {
        // 全ゼロ鍵で簡易テスト（32 バイト = 64 hex chars）
        let key_bytes = hex::decode("0".repeat(64)).unwrap();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        MasterKey { key: *key }
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let master = make_test_key();
        let plaintext = b"secret-password-123";

        let (ciphertext, nonce) = master.encrypt(plaintext).unwrap();
        let decrypted = master.decrypt(&ciphertext, &nonce).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_unique_nonces() {
        let master = make_test_key();
        let plaintext = b"same data";

        let (_, nonce1) = master.encrypt(plaintext).unwrap();
        let (_, nonce2) = master.encrypt(plaintext).unwrap();

        // 異なる nonce が生成されることを確認
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn test_decrypt_with_wrong_nonce_fails() {
        let master = make_test_key();
        let plaintext = b"secret data";

        let (ciphertext, _) = master.encrypt(plaintext).unwrap();
        let wrong_nonce = vec![0u8; 12];

        let result = master.decrypt(&ciphertext, &wrong_nonce);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_tampered_ciphertext_fails() {
        let master = make_test_key();
        let plaintext = b"tamper test";

        let (mut ciphertext, nonce) = master.encrypt(plaintext).unwrap();
        if let Some(last) = ciphertext.last_mut() {
            *last ^= 0xFF;
        }

        let result = master.decrypt(&ciphertext, &nonce);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_env_default_key() {
        // 環境変数操作の競合を防ぐためロックを取得
        let _guard = ENV_LOCK.lock().unwrap();
        // VAULT_MASTER_KEY が未設定の場合、ゼロ鍵が使われる
        std::env::remove_var("VAULT_MASTER_KEY");
        let result = MasterKey::from_env();
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_env_invalid_hex() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("VAULT_MASTER_KEY", "not-valid-hex");
        let result = MasterKey::from_env();
        assert!(result.is_err());
        std::env::remove_var("VAULT_MASTER_KEY");
    }

    #[test]
    fn test_from_env_wrong_length() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("VAULT_MASTER_KEY", "aabb"); // 2 bytes, not 32
        let result = MasterKey::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("32 bytes"));
        std::env::remove_var("VAULT_MASTER_KEY");
    }
}
