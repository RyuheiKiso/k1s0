use std::sync::Arc;

use chrono::Utc;

use crate::domain::repository::api_key_repository::ApiKeyRepository;

/// ValidateApiKeyError は API キー検証に関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum ValidateApiKeyError {
    #[error("invalid api key")]
    Invalid,

    #[error("api key revoked")]
    Revoked,

    #[error("api key expired")]
    Expired,

    #[error("internal error: {0}")]
    Internal(String),

    /// API_KEY_PEPPER 環境変数が未設定の場合のエラー。
    #[error("pepper not configured")]
    PepperNotConfigured,
}

/// ValidateApiKeyResult は検証成功時の結果。
#[derive(Debug, Clone)]
pub struct ValidateApiKeyResult {
    pub tenant_id: String,
    pub name: String,
    pub scopes: Vec<String>,
}

/// ValidateApiKeyUseCase は API キー検証ユースケース。
pub struct ValidateApiKeyUseCase {
    repo: Arc<dyn ApiKeyRepository>,
}

impl ValidateApiKeyUseCase {
    pub fn new(repo: Arc<dyn ApiKeyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        raw_key: &str,
    ) -> Result<ValidateApiKeyResult, ValidateApiKeyError> {
        if raw_key.len() < 13 {
            return Err(ValidateApiKeyError::Invalid);
        }

        let prefix = &raw_key[..13];
        let api_key = self
            .repo
            .find_by_prefix(prefix)
            .await
            .map_err(|e| ValidateApiKeyError::Internal(e.to_string()))?
            .ok_or(ValidateApiKeyError::Invalid)?;

        // H-008 監査対応: ハッシュ比較を定数時間で行い、タイミング攻撃を防止する
        // 通常の文字列比較（!=）は最初に不一致したバイトで短絡し、タイミングの差でハッシュを推測可能になる
        use subtle::ConstantTimeEq;
        let computed_hash = hash_key(raw_key)?;
        if computed_hash
            .as_bytes()
            .ct_eq(api_key.key_hash.as_bytes())
            .unwrap_u8()
            != 1
        {
            return Err(ValidateApiKeyError::Invalid);
        }

        if api_key.revoked {
            return Err(ValidateApiKeyError::Revoked);
        }

        if let Some(expires_at) = api_key.expires_at {
            if Utc::now() > expires_at {
                return Err(ValidateApiKeyError::Expired);
            }
        }

        Ok(ValidateApiKeyResult {
            tenant_id: api_key.tenant_id,
            name: api_key.name,
            scopes: api_key.scopes,
        })
    }
}

/// HMAC-SHA256 を使用して API キーをハッシュ化する。
/// サーバー側ペッパーにより、DB 漏洩時でも元キーの復元を困難にする。
/// ペッパーが未設定の場合はエラーを返し、デフォルト値へのフォールバックを行わない。
fn hash_key(raw_key: &str) -> Result<String, ValidateApiKeyError> {
    // サーバー側ペッパーを環境変数から取得（未設定時はエラー）
    let pepper =
        std::env::var("API_KEY_PEPPER").map_err(|_| ValidateApiKeyError::PepperNotConfigured)?;
    Ok(compute_hmac_hex(raw_key, &pepper))
}

/// HMAC-SHA256 ハッシュ計算の内部実装。
/// テストから環境変数に依存せず直接呼び出せるよう分離する。
fn compute_hmac_hex(raw_key: &str, pepper: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let mut mac =
        HmacSha256::new_from_slice(pepper.as_bytes()).expect("HMAC accepts any key length");
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
    use crate::domain::entity::api_key::ApiKey;
    use crate::domain::repository::api_key_repository::MockApiKeyRepository;
    use uuid::Uuid;

    /// テスト用ペッパー定数（本番環境では使用しない）。
    const TEST_PEPPER: &str = "test-pepper-for-unit-tests";

    /// env var を変更するテスト間の競合を防ぐためのセマフォ（max=1 で直列化）。
    static PEPPER_SEM: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

    /// テスト用 ApiKey を生成する。
    /// compute_hmac_hex を直接使用することで、環境変数に依存しないテストを実現する。
    fn make_api_key(raw_key: &str, revoked: bool, expired: bool) -> ApiKey {
        let now = Utc::now();
        let expires_at = if expired {
            Some(now - chrono::Duration::hours(1))
        } else {
            None
        };

        ApiKey {
            id: Uuid::new_v4(),
            tenant_id: "tenant-1".to_string(),
            name: "Test Key".to_string(),
            // 環境変数に依存せず、固定ペッパーで直接ハッシュを生成する
            key_hash: compute_hmac_hex(raw_key, TEST_PEPPER),
            prefix: raw_key[..13].to_string(),
            scopes: vec!["read".to_string()],
            expires_at,
            revoked,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn test_validate_api_key_success() {
        // ペッパーセマフォを取得し、env var を安定させる
        let _permit = PEPPER_SEM.acquire().await.unwrap();
        std::env::set_var("API_KEY_PEPPER", TEST_PEPPER);
        let raw_key = "k1s0_abcdef1234567890abcdef";
        let api_key = make_api_key(raw_key, false, false);

        let mut mock = MockApiKeyRepository::new();
        mock.expect_find_by_prefix()
            .withf(|p| p == "k1s0_abcdef12")
            .returning(move |_| Ok(Some(api_key.clone())));

        let uc = ValidateApiKeyUseCase::new(Arc::new(mock));
        let result = uc.execute(raw_key).await;
        assert!(result.is_ok());

        let val = result.unwrap();
        assert_eq!(val.tenant_id, "tenant-1");
        assert_eq!(val.scopes, vec!["read"]);
    }

    #[tokio::test]
    async fn test_validate_api_key_not_found() {
        let _permit = PEPPER_SEM.acquire().await.unwrap();
        std::env::set_var("API_KEY_PEPPER", TEST_PEPPER);
        let mut mock = MockApiKeyRepository::new();
        mock.expect_find_by_prefix().returning(|_| Ok(None));

        let uc = ValidateApiKeyUseCase::new(Arc::new(mock));
        let result = uc.execute("k1s0_nonexistent_key").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ValidateApiKeyError::Invalid => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_validate_api_key_revoked() {
        let _permit = PEPPER_SEM.acquire().await.unwrap();
        std::env::set_var("API_KEY_PEPPER", TEST_PEPPER);
        let raw_key = "k1s0_revoked_1234567890abc";
        let api_key = make_api_key(raw_key, true, false);

        let mut mock = MockApiKeyRepository::new();
        mock.expect_find_by_prefix()
            .returning(move |_| Ok(Some(api_key.clone())));

        let uc = ValidateApiKeyUseCase::new(Arc::new(mock));
        let result = uc.execute(raw_key).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ValidateApiKeyError::Revoked => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_validate_api_key_expired() {
        let _permit = PEPPER_SEM.acquire().await.unwrap();
        std::env::set_var("API_KEY_PEPPER", TEST_PEPPER);
        let raw_key = "k1s0_expired_1234567890abc";
        let api_key = make_api_key(raw_key, false, true);

        let mut mock = MockApiKeyRepository::new();
        mock.expect_find_by_prefix()
            .returning(move |_| Ok(Some(api_key.clone())));

        let uc = ValidateApiKeyUseCase::new(Arc::new(mock));
        let result = uc.execute(raw_key).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ValidateApiKeyError::Expired => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_validate_api_key_too_short() {
        let _permit = PEPPER_SEM.acquire().await.unwrap();
        std::env::set_var("API_KEY_PEPPER", TEST_PEPPER);
        let mock = MockApiKeyRepository::new();
        let uc = ValidateApiKeyUseCase::new(Arc::new(mock));
        let result = uc.execute("short").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ValidateApiKeyError::Invalid => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    /// 同一入力に対して compute_hmac_hex が決定的な結果を返すことを確認する。
    /// 環境変数を使わず compute_hmac_hex を直接呼び出してテスト間の競合を回避する。
    #[test]
    fn test_hash_key_deterministic() {
        let key = "k1s0_test_deterministic_key";
        let h1 = compute_hmac_hex(key, TEST_PEPPER);
        let h2 = compute_hmac_hex(key, TEST_PEPPER);
        assert_eq!(h1, h2, "同一入力に対するハッシュは一致すべき");
    }

    /// 異なる入力に対して compute_hmac_hex が異なるハッシュを返すことを確認する。
    #[test]
    fn test_hash_key_different_inputs() {
        let h1 = compute_hmac_hex("k1s0_key_alpha", TEST_PEPPER);
        let h2 = compute_hmac_hex("k1s0_key_beta", TEST_PEPPER);
        assert_ne!(h1, h2, "異なる入力に対するハッシュは異なるべき");
    }

    /// ペッパーが変わるとハッシュ値も変わることを確認する。
    /// 環境変数を使わず compute_hmac_hex を直接呼び出してテスト間の競合を回避する。
    #[test]
    fn test_hash_key_pepper_changes_output() {
        let key = "k1s0_pepper_test_key_12345";
        let h_first = compute_hmac_hex(key, TEST_PEPPER);
        let h_custom = compute_hmac_hex(key, "custom-test-pepper");
        assert_ne!(h_first, h_custom, "ペッパーが異なればハッシュも異なるべき");
    }

    /// ペッパーが未設定の場合に hash_key がエラーを返すことを確認する。
    /// セマフォを取得して async テストとの env var 競合を防ぐ。
    #[test]
    fn test_hash_key_pepper_not_set_returns_error() {
        // 一時的な tokio ランタイムでセマフォを取得してから環境変数を操作する
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        rt.block_on(async {
            let _permit = PEPPER_SEM.acquire().await.unwrap();
            std::env::remove_var("API_KEY_PEPPER");
            let result = hash_key("k1s0_test_key_12345");
            assert!(matches!(
                result,
                Err(ValidateApiKeyError::PepperNotConfigured)
            ));
            // permit がここでドロップされ、セマフォが解放される
        });
    }
}
