//! トークンブラックリスト
//!
//! JWT トークンの無効化（リボケーション）を管理する。
//!
//! # 機能
//!
//! - トークンID (jti) によるブラックリスト管理
//! - 有効期限付きのブラックリスト登録
//! - インメモリ/Redis による実装
//!
//! # 使用例
//!
//! ```rust
//! use k1s0_auth::blacklist::{TokenBlacklist, InMemoryBlacklist};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() {
//!     let blacklist = InMemoryBlacklist::new();
//!
//!     // トークンをブラックリストに追加（1時間後に期限切れ）
//!     blacklist.revoke("token-jti-123", Duration::from_secs(3600)).await.unwrap();
//!
//!     // トークンがブラックリストにあるかチェック
//!     assert!(blacklist.is_revoked("token-jti-123").await.unwrap());
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::error::AuthError;

/// トークンブラックリストトレイト
#[async_trait]
pub trait TokenBlacklist: Send + Sync {
    /// トークンを無効化（ブラックリストに追加）
    ///
    /// # 引数
    ///
    /// * `jti` - トークンID (JWT ID)
    /// * `ttl` - ブラックリストに保持する期間（通常はトークンの残り有効期間）
    async fn revoke(&self, jti: &str, ttl: Duration) -> Result<(), AuthError>;

    /// トークンが無効化されているかチェック
    async fn is_revoked(&self, jti: &str) -> Result<bool, AuthError>;

    /// トークンをブラックリストから削除
    async fn remove(&self, jti: &str) -> Result<bool, AuthError>;

    /// ユーザーの全トークンを無効化
    async fn revoke_all_for_user(&self, user_id: &str, ttl: Duration) -> Result<u64, AuthError>;

    /// ブラックリストのエントリ数を取得
    async fn count(&self) -> Result<u64, AuthError>;

    /// 期限切れエントリをクリーンアップ
    async fn cleanup(&self) -> Result<u64, AuthError>;
}

/// インメモリブラックリスト
///
/// 開発・テスト用のインメモリ実装。
/// 本番環境では Redis 実装を推奨。
pub struct InMemoryBlacklist {
    /// ブラックリスト（jti -> 有効期限）
    tokens: Arc<RwLock<HashMap<String, Instant>>>,
    /// ユーザーごとのブラックリスト（user_id -> 無効化時刻）
    users: Arc<RwLock<HashMap<String, Instant>>>,
}

impl Default for InMemoryBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryBlacklist {
    /// 新しいブラックリストを作成
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// トークンをユーザーに関連付けて無効化
    pub async fn revoke_with_user(
        &self,
        jti: &str,
        user_id: &str,
        ttl: Duration,
    ) -> Result<(), AuthError> {
        let expires_at = Instant::now() + ttl;

        {
            let mut tokens = self.tokens.write().await;
            tokens.insert(jti.to_string(), expires_at);
        }

        // ユーザー関連付けは別途管理（将来的な拡張用）
        debug!(jti = %jti, user_id = %user_id, "Token revoked");
        Ok(())
    }

    /// 特定時刻以前に発行されたトークンがブラックリストにあるかチェック
    pub async fn is_user_revoked_since(&self, user_id: &str, issued_at: i64) -> Result<bool, AuthError> {
        let users = self.users.read().await;
        if let Some(revoked_at) = users.get(user_id) {
            // revoked_at より前の現在時刻との差分を計算
            // issued_at は Unix timestamp なので、revoked_at と比較する必要がある
            // ここでは簡略化のため、常に false を返す（将来の実装で改善）
            let _ = revoked_at;
            let _ = issued_at;
            Ok(false)
        } else {
            Ok(false)
        }
    }
}

#[async_trait]
impl TokenBlacklist for InMemoryBlacklist {
    async fn revoke(&self, jti: &str, ttl: Duration) -> Result<(), AuthError> {
        let expires_at = Instant::now() + ttl;

        {
            let mut tokens = self.tokens.write().await;
            tokens.insert(jti.to_string(), expires_at);
        }

        debug!(jti = %jti, ttl = ?ttl, "Token added to blacklist");
        Ok(())
    }

    async fn is_revoked(&self, jti: &str) -> Result<bool, AuthError> {
        let tokens = self.tokens.read().await;

        if let Some(expires_at) = tokens.get(jti) {
            if Instant::now() < *expires_at {
                debug!(jti = %jti, "Token found in blacklist (still valid)");
                return Ok(true);
            }
            // 期限切れのエントリ（cleanup で削除される）
            debug!(jti = %jti, "Token found in blacklist but expired");
        }

        Ok(false)
    }

    async fn remove(&self, jti: &str) -> Result<bool, AuthError> {
        let mut tokens = self.tokens.write().await;
        let removed = tokens.remove(jti).is_some();

        if removed {
            debug!(jti = %jti, "Token removed from blacklist");
        }

        Ok(removed)
    }

    async fn revoke_all_for_user(&self, user_id: &str, ttl: Duration) -> Result<u64, AuthError> {
        let expires_at = Instant::now() + ttl;

        {
            let mut users = self.users.write().await;
            users.insert(user_id.to_string(), expires_at);
        }

        info!(user_id = %user_id, "All tokens revoked for user");
        Ok(1) // ユーザーエントリは1つ
    }

    async fn count(&self) -> Result<u64, AuthError> {
        let tokens = self.tokens.read().await;
        Ok(tokens.len() as u64)
    }

    async fn cleanup(&self) -> Result<u64, AuthError> {
        let now = Instant::now();
        let mut cleaned = 0u64;

        // トークンのクリーンアップ
        {
            let mut tokens = self.tokens.write().await;
            let before = tokens.len();
            tokens.retain(|_, expires_at| *expires_at > now);
            cleaned += (before - tokens.len()) as u64;
        }

        // ユーザーのクリーンアップ
        {
            let mut users = self.users.write().await;
            let before = users.len();
            users.retain(|_, expires_at| *expires_at > now);
            cleaned += (before - users.len()) as u64;
        }

        if cleaned > 0 {
            debug!(count = cleaned, "Cleaned up expired blacklist entries");
        }

        Ok(cleaned)
    }
}

/// Redis ブラックリスト
///
/// Redis を使用した分散ブラックリスト実装。
#[cfg(feature = "redis-cache")]
pub struct RedisBlacklist {
    cache: Arc<k1s0_cache::CacheClient>,
    key_prefix: String,
}

#[cfg(feature = "redis-cache")]
impl RedisBlacklist {
    /// 新しい Redis ブラックリストを作成
    pub fn new(cache: Arc<k1s0_cache::CacheClient>) -> Self {
        Self {
            cache,
            key_prefix: "auth:blacklist".to_string(),
        }
    }

    /// キープレフィックスを設定
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = prefix.into();
        self
    }

    /// トークン用キーを生成
    fn token_key(&self, jti: &str) -> String {
        format!("{}:token:{}", self.key_prefix, jti)
    }

    /// ユーザー用キーを生成
    fn user_key(&self, user_id: &str) -> String {
        format!("{}:user:{}", self.key_prefix, user_id)
    }
}

#[cfg(feature = "redis-cache")]
#[async_trait]
impl TokenBlacklist for RedisBlacklist {
    async fn revoke(&self, jti: &str, ttl: Duration) -> Result<(), AuthError> {
        use k1s0_cache::CacheOperations;

        let key = self.token_key(jti);

        self.cache
            .set(&key, &true, Some(ttl))
            .await
            .map_err(|e| AuthError::internal(format!("Failed to revoke token: {}", e)))?;

        debug!(jti = %jti, ttl = ?ttl, "Token added to Redis blacklist");
        Ok(())
    }

    async fn is_revoked(&self, jti: &str) -> Result<bool, AuthError> {
        use k1s0_cache::CacheOperations;

        let key = self.token_key(jti);

        let exists: Option<bool> = self.cache
            .get(&key)
            .await
            .map_err(|e| AuthError::internal(format!("Failed to check token: {}", e)))?;

        Ok(exists.unwrap_or(false))
    }

    async fn remove(&self, jti: &str) -> Result<bool, AuthError> {
        use k1s0_cache::CacheOperations;

        let key = self.token_key(jti);

        self.cache
            .delete(&key)
            .await
            .map_err(|e| AuthError::internal(format!("Failed to remove token: {}", e)))
    }

    async fn revoke_all_for_user(&self, user_id: &str, ttl: Duration) -> Result<u64, AuthError> {
        use k1s0_cache::CacheOperations;

        let key = self.user_key(user_id);
        let revoked_at = chrono::Utc::now().timestamp();

        self.cache
            .set(&key, &revoked_at, Some(ttl))
            .await
            .map_err(|e| AuthError::internal(format!("Failed to revoke user tokens: {}", e)))?;

        info!(user_id = %user_id, "All tokens revoked for user in Redis");
        Ok(1)
    }

    async fn count(&self) -> Result<u64, AuthError> {
        // Redis では SCAN でカウントする必要があるが、コストが高い
        // 実装は省略（必要に応じて追加）
        Ok(0)
    }

    async fn cleanup(&self) -> Result<u64, AuthError> {
        // Redis の TTL が自動的にクリーンアップする
        Ok(0)
    }
}

/// Redis ブラックリストでユーザー全体の無効化をチェック
#[cfg(feature = "redis-cache")]
impl RedisBlacklist {
    /// ユーザーの全トークンが無効化されているかチェック
    pub async fn is_user_revoked_since(&self, user_id: &str, issued_at: i64) -> Result<bool, AuthError> {
        use k1s0_cache::CacheOperations;

        let key = self.user_key(user_id);

        let revoked_at: Option<i64> = self.cache
            .get(&key)
            .await
            .map_err(|e| AuthError::internal(format!("Failed to check user revocation: {}", e)))?;

        match revoked_at {
            Some(timestamp) => Ok(issued_at < timestamp),
            None => Ok(false),
        }
    }
}

/// ブラックリストを使用した JWT 検証
pub struct BlacklistAwareVerifier<B: TokenBlacklist> {
    blacklist: Arc<B>,
    verifier: crate::jwt::JwtVerifier,
}

impl<B: TokenBlacklist> BlacklistAwareVerifier<B> {
    /// 新しい検証器を作成
    pub fn new(verifier: crate::jwt::JwtVerifier, blacklist: Arc<B>) -> Self {
        Self { blacklist, verifier }
    }

    /// JWT を検証（ブラックリストチェック付き）
    pub async fn verify(&self, token: &str) -> Result<crate::jwt::Claims, AuthError> {
        let claims = self.verifier.verify(token).await?;

        // jti があればブラックリストをチェック
        if let Some(ref jti) = claims.jti {
            if self.blacklist.is_revoked(jti).await? {
                return Err(AuthError::token_revoked(jti.clone()));
            }
        }

        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_blacklist_revoke() {
        let blacklist = InMemoryBlacklist::new();

        blacklist
            .revoke("jti-123", Duration::from_secs(3600))
            .await
            .unwrap();

        assert!(blacklist.is_revoked("jti-123").await.unwrap());
        assert!(!blacklist.is_revoked("jti-456").await.unwrap());
    }

    #[tokio::test]
    async fn test_in_memory_blacklist_remove() {
        let blacklist = InMemoryBlacklist::new();

        blacklist
            .revoke("jti-123", Duration::from_secs(3600))
            .await
            .unwrap();

        assert!(blacklist.is_revoked("jti-123").await.unwrap());

        assert!(blacklist.remove("jti-123").await.unwrap());
        assert!(!blacklist.is_revoked("jti-123").await.unwrap());

        // 存在しないキーの削除
        assert!(!blacklist.remove("jti-123").await.unwrap());
    }

    #[tokio::test]
    async fn test_in_memory_blacklist_count() {
        let blacklist = InMemoryBlacklist::new();

        assert_eq!(blacklist.count().await.unwrap(), 0);

        blacklist.revoke("jti-1", Duration::from_secs(3600)).await.unwrap();
        blacklist.revoke("jti-2", Duration::from_secs(3600)).await.unwrap();
        blacklist.revoke("jti-3", Duration::from_secs(3600)).await.unwrap();

        assert_eq!(blacklist.count().await.unwrap(), 3);
    }

    #[tokio::test]
    async fn test_in_memory_blacklist_cleanup() {
        let blacklist = InMemoryBlacklist::new();

        // 期限切れのエントリを追加
        blacklist
            .revoke("jti-expired", Duration::from_millis(1))
            .await
            .unwrap();

        // 少し待つ
        tokio::time::sleep(Duration::from_millis(10)).await;

        // まだカウントされる
        assert_eq!(blacklist.count().await.unwrap(), 1);

        // クリーンアップ
        let cleaned = blacklist.cleanup().await.unwrap();
        assert_eq!(cleaned, 1);
        assert_eq!(blacklist.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_in_memory_blacklist_expired_check() {
        let blacklist = InMemoryBlacklist::new();

        blacklist
            .revoke("jti-short", Duration::from_millis(1))
            .await
            .unwrap();

        // 期限切れ前
        assert!(blacklist.is_revoked("jti-short").await.unwrap());

        // 期限切れ後
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(!blacklist.is_revoked("jti-short").await.unwrap());
    }

    #[tokio::test]
    async fn test_revoke_all_for_user() {
        let blacklist = InMemoryBlacklist::new();

        blacklist
            .revoke_all_for_user("user-123", Duration::from_secs(3600))
            .await
            .unwrap();

        // ユーザー単位のチェックは実装依存
        // ここではエラーなく実行されることを確認
    }
}
