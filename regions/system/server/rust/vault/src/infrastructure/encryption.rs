use aes_gcm::aead::rand_core::RngCore;
// CRIT-003 監査対応: Payload を追加して AAD（Additional Authenticated Data）を暗号化操作に渡す
// ciphertext swap attack を防止し NIST SP 800-38D 準拠を達成する
use aes_gcm::aead::{Aead, KeyInit, OsRng, Payload};
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
                // 本番・ステージング環境では VAULT_MASTER_KEY が必須
                // 大文字小文字を無視して比較（"Production", "PRODUCTION" 等のバイパスを防止）
                let env_lower = environment.to_lowercase();
                if env_lower == "production" || env_lower == "staging" {
                    return Err(anyhow::anyhow!(
                        "VAULT_MASTER_KEY 環境変数は本番・ステージング環境では必須です"
                    ));
                }
                // 開発環境: 起動ごとにランダムな鍵を生成（再起動でデータ復号不可になる点に注意）
                // ゼロ鍵は既知の弱い鍵であるため使用しない
                tracing::warn!(
                    "VAULT_MASTER_KEY が設定されていません。開発環境用にランダム鍵を生成します（再起動で暗号化データが失われます）"
                );
                let mut key_bytes = [0u8; 32];
                // aes_gcm クレートが依存する rand_core の OsRng で安全な乱数を生成する
                OsRng.fill_bytes(&mut key_bytes);
                hex::encode(key_bytes)
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
    /// CRIT-003 監査対応: aad（Additional Authenticated Data）を Payload に含めることで
    /// ciphertext swap attack を防止する。aad にはシークレットのパス等の識別子を渡す。
    pub fn encrypt(&self, plaintext: &[u8], aad: &[u8]) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
        let cipher = Aes256Gcm::new(&self.key);
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        // CRIT-003 監査対応: AAD を Payload に含めて暗号化し、認証タグがコンテキストを保証する
        let ciphertext = cipher
            .encrypt(nonce, Payload { msg: plaintext, aad })
            .map_err(|e| anyhow::anyhow!("encryption failed: {}", e))?;
        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    /// 暗号文と nonce から平文データを復号化する。
    /// CRIT-003 監査対応: aad（Additional Authenticated Data）を Payload に含めることで
    /// 暗号化時と同一の AAD が指定された場合のみ復号成功となる。
    /// AAD が異なる場合は認証タグ検証に失敗しエラーを返す。
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8], aad: &[u8]) -> anyhow::Result<Vec<u8>> {
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(nonce);
        // CRIT-003 監査対応: AAD を Payload に含めて復号することでコンテキスト認証を実施する
        cipher
            .decrypt(nonce, Payload { msg: ciphertext, aad })
            .map_err(|e| anyhow::anyhow!("decryption failed: {}", e))
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
        let aad = b"vault/test-path";

        let (ciphertext, nonce) = master.encrypt(plaintext, aad).unwrap();
        let decrypted = master.decrypt(&ciphertext, &nonce, aad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_unique_nonces() {
        let master = make_test_key();
        let plaintext = b"same data";
        let aad = b"vault/test-path";

        let (_, nonce1) = master.encrypt(plaintext, aad).unwrap();
        let (_, nonce2) = master.encrypt(plaintext, aad).unwrap();

        // 異なる nonce が生成されることを確認
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn test_decrypt_with_wrong_nonce_fails() {
        let master = make_test_key();
        let plaintext = b"secret data";
        let aad = b"vault/test-path";

        let (ciphertext, _) = master.encrypt(plaintext, aad).unwrap();
        let wrong_nonce = vec![0u8; 12];

        let result = master.decrypt(&ciphertext, &wrong_nonce, aad);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_tampered_ciphertext_fails() {
        let master = make_test_key();
        let plaintext = b"tamper test";
        let aad = b"vault/test-path";

        let (mut ciphertext, nonce) = master.encrypt(plaintext, aad).unwrap();
        if let Some(last) = ciphertext.last_mut() {
            *last ^= 0xFF;
        }

        let result = master.decrypt(&ciphertext, &nonce, aad);
        assert!(result.is_err());
    }

    /// CRIT-003 監査対応: AAD 不一致で復号が失敗することを確認する（ciphertext swap attack 防止）。
    /// 異なる AAD で暗号化されたデータは、別の AAD を使用しても復号できないことを検証する。
    #[test]
    fn test_decrypt_with_wrong_aad_fails() {
        let master = make_test_key();
        let plaintext = b"sensitive vault secret";
        let correct_aad = b"vault/my-service/db-password";
        let wrong_aad = b"vault/other-service/api-key";

        // 正しい AAD で暗号化する
        let (ciphertext, nonce) = master.encrypt(plaintext, correct_aad).unwrap();

        // 正しい AAD では復号成功を確認する
        let decrypted = master.decrypt(&ciphertext, &nonce, correct_aad).unwrap();
        assert_eq!(decrypted, plaintext);

        // 異なる AAD では復号が失敗することを確認する（認証タグ検証失敗）
        assert!(master.decrypt(&ciphertext, &nonce, wrong_aad).is_err());
    }

    #[test]
    fn test_from_env_default_key() {
        // 環境変数操作の競合を防ぐためロックを取得
        let _guard = ENV_LOCK.lock().unwrap();
        // VAULT_MASTER_KEY が未設定の場合、開発環境ではランダム鍵が生成されて Ok が返る
        std::env::remove_var("VAULT_MASTER_KEY");
        // APP_ENVIRONMENT が空（開発環境扱い）のため、ランダム鍵が生成される
        std::env::remove_var("APP_ENVIRONMENT");
        let result = MasterKey::from_env();
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_env_random_key_is_unique() {
        // 環境変数操作の競合を防ぐためロックを取得
        let _guard = ENV_LOCK.lock().unwrap();
        // 開発環境では呼び出しごとに異なるランダム鍵が生成されることを確認
        std::env::remove_var("VAULT_MASTER_KEY");
        std::env::remove_var("APP_ENVIRONMENT");
        let key1 = MasterKey::from_env().unwrap();
        let key2 = MasterKey::from_env().unwrap();
        // 鍵の内容が異なることを確認（同一ゼロ鍵が返らないことの検証）
        let pt = b"test";
        let aad = b"vault/test";
        let (ct1, n1) = key1.encrypt(pt, aad).unwrap();
        let (ct2, n2) = key2.encrypt(pt, aad).unwrap();
        // 異なる鍵で暗号化されたため、一方の鍵で他方を復号できない
        assert!(key1.decrypt(&ct2, &n2, aad).is_err() || key2.decrypt(&ct1, &n1, aad).is_err());
    }

    #[test]
    fn test_from_env_production_requires_key() {
        // 環境変数操作の競合を防ぐためロックを取得
        let _guard = ENV_LOCK.lock().unwrap();
        // 本番環境で VAULT_MASTER_KEY が未設定の場合はエラーになる
        std::env::remove_var("VAULT_MASTER_KEY");
        std::env::set_var("APP_ENVIRONMENT", "production");
        let result = MasterKey::from_env();
        assert!(result.is_err());
        std::env::remove_var("APP_ENVIRONMENT");
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
