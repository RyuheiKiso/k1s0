use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::entity::api_key::{ApiKey, CreateApiKeyRequest, CreateApiKeyResponse};
use crate::domain::repository::api_key_repository::ApiKeyRepository;

/// CreateApiKeyError は API キー作成に関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum CreateApiKeyError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// CreateApiKeyUseCase は API キー作成ユースケース。
pub struct CreateApiKeyUseCase {
    repo: Arc<dyn ApiKeyRepository>,
}

impl CreateApiKeyUseCase {
    pub fn new(repo: Arc<dyn ApiKeyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        req: CreateApiKeyRequest,
    ) -> Result<CreateApiKeyResponse, CreateApiKeyError> {
        if req.name.is_empty() {
            return Err(CreateApiKeyError::Validation(
                "name is required".to_string(),
            ));
        }
        if req.tenant_id.is_empty() {
            return Err(CreateApiKeyError::Validation(
                "tenant_id is required".to_string(),
            ));
        }

        let id = Uuid::new_v4();
        let raw_key = format!("k1s0_{}", generate_random_key());
        let prefix = raw_key[..13].to_string();
        let key_hash = hash_key(&raw_key);
        let now = Utc::now();

        let api_key = ApiKey {
            id,
            tenant_id: req.tenant_id,
            name: req.name.clone(),
            key_hash,
            prefix: prefix.clone(),
            scopes: req.scopes.clone(),
            expires_at: req.expires_at,
            revoked: false,
            created_at: now,
            updated_at: now,
        };

        self.repo
            .create(&api_key)
            .await
            .map_err(|e| CreateApiKeyError::Internal(e.to_string()))?;

        Ok(CreateApiKeyResponse {
            id,
            name: req.name,
            prefix,
            raw_key,
            scopes: req.scopes,
            expires_at: req.expires_at,
            created_at: now,
        })
    }
}

fn generate_random_key() -> String {
    use std::fmt::Write;
    let bytes: [u8; 24] = {
        let mut buf = [0u8; 24];
        // Use uuid v4 randomness as source
        let u1 = Uuid::new_v4();
        let u2 = Uuid::new_v4();
        buf[..16].copy_from_slice(u1.as_bytes());
        buf[16..].copy_from_slice(&u2.as_bytes()[..8]);
        buf
    };
    let mut s = String::with_capacity(48);
    for b in &bytes {
        // String への write!() は常に成功するため失敗しない
        let _ = write!(s, "{:02x}", b);
    }
    s
}

/// HMAC-SHA256 を使用して API キーをハッシュ化する。
/// サーバー側ペッパーにより、DB 漏洩時でも元キーの復元を困難にする。
fn hash_key(raw_key: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    // サーバー側ペッパーを環境変数から取得（未設定時は開発用デフォルト）
    let pepper = std::env::var("API_KEY_PEPPER")
        .unwrap_or_else(|_| "k1s0-dev-pepper-do-not-use-in-production".to_string());

    let mut mac = HmacSha256::new_from_slice(pepper.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(raw_key.as_bytes());
    let result = mac.finalize();
    let digest = result.into_bytes();

    let mut out = String::with_capacity(digest.len() * 2);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(&mut out, "{:02x}", b);
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::api_key_repository::MockApiKeyRepository;

    #[tokio::test]
    async fn test_create_api_key_success() {
        let mut mock = MockApiKeyRepository::new();
        mock.expect_create().returning(|_| Ok(()));

        let uc = CreateApiKeyUseCase::new(Arc::new(mock));
        let req = CreateApiKeyRequest {
            tenant_id: "tenant-1".to_string(),
            name: "My Key".to_string(),
            scopes: vec!["read".to_string()],
            expires_at: None,
        };

        let result = uc.execute(req).await;
        assert!(result.is_ok());

        let resp = result.expect("create_api_key should succeed");
        assert_eq!(resp.name, "My Key");
        assert!(resp.raw_key.starts_with("k1s0_"));
        assert!(!resp.prefix.is_empty());
    }

    #[tokio::test]
    async fn test_create_api_key_empty_name() {
        let mock = MockApiKeyRepository::new();
        let uc = CreateApiKeyUseCase::new(Arc::new(mock));

        let req = CreateApiKeyRequest {
            tenant_id: "tenant-1".to_string(),
            name: String::new(),
            scopes: vec![],
            expires_at: None,
        };

        let result = uc.execute(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateApiKeyError::Validation(msg) => assert!(msg.contains("name")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_create_api_key_empty_tenant() {
        let mock = MockApiKeyRepository::new();
        let uc = CreateApiKeyUseCase::new(Arc::new(mock));

        let req = CreateApiKeyRequest {
            tenant_id: String::new(),
            name: "Key".to_string(),
            scopes: vec![],
            expires_at: None,
        };

        let result = uc.execute(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateApiKeyError::Validation(msg) => assert!(msg.contains("tenant_id")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    /// 同一入力に対して hash_key が決定的な結果を返すことを確認する。
    #[test]
    fn test_hash_key_deterministic() {
        // 環境変数を固定してテスト間の競合を防ぐ
        std::env::set_var("API_KEY_PEPPER", "test-pepper-deterministic");
        let key = "k1s0_test_deterministic_key";
        let h1 = hash_key(key);
        let h2 = hash_key(key);
        assert_eq!(h1, h2, "同一入力に対するハッシュは一致すべき");
    }

    /// 異なる入力に対して hash_key が異なるハッシュを返すことを確認する。
    #[test]
    fn test_hash_key_different_inputs() {
        let h1 = hash_key("k1s0_key_alpha");
        let h2 = hash_key("k1s0_key_beta");
        assert_ne!(h1, h2, "異なる入力に対するハッシュは異なるべき");
    }

    /// ペッパーが変わるとハッシュ値も変わることを確認する。
    #[test]
    fn test_hash_key_pepper_changes_output() {
        let key = "k1s0_pepper_test_key_12345";

        // デフォルトペッパーでハッシュ生成
        std::env::remove_var("API_KEY_PEPPER");
        let h_default = hash_key(key);

        // カスタムペッパーでハッシュ生成
        std::env::set_var("API_KEY_PEPPER", "custom-test-pepper");
        let h_custom = hash_key(key);

        // テスト後に環境変数をクリーンアップ
        std::env::remove_var("API_KEY_PEPPER");

        assert_ne!(
            h_default, h_custom,
            "ペッパーが異なればハッシュも異なるべき"
        );
    }
}
