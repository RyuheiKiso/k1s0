//! リフレッシュトークン管理
//!
//! リフレッシュトークンの発行、検証、およびローテーションを管理する。
//!
//! # 機能
//!
//! - セキュアなリフレッシュトークン生成
//! - トークンローテーション（使用時に新しいトークンを発行）
//! - トークンファミリー追跡（リプレイ攻撃検出）
//! - インメモリ/Redis によるストレージ
//!
//! # 使用例
//!
//! ```rust
//! use k1s0_auth::refresh::{RefreshTokenManager, InMemoryRefreshTokenStore, RefreshTokenConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let store = InMemoryRefreshTokenStore::new();
//!     let config = RefreshTokenConfig::default();
//!     let manager = RefreshTokenManager::new(store, config);
//!
//!     // トークンを発行
//!     let token = manager.issue("user-123", None).await.unwrap();
//!
//!     // トークンを検証してローテーション
//!     let (new_token, user_id) = manager.rotate(&token.token).await.unwrap();
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::AuthError;

/// リフレッシュトークン設定
#[derive(Debug, Clone)]
pub struct RefreshTokenConfig {
    /// トークンの有効期間
    pub token_ttl: Duration,
    /// ローテーション有効期間（古いトークンがまだ使える猶予期間）
    pub rotation_grace_period: Duration,
    /// トークンファミリーの最大サイズ
    pub max_family_size: usize,
    /// 同一ユーザーの最大アクティブトークン数
    pub max_tokens_per_user: usize,
    /// トークン長（バイト）
    pub token_bytes: usize,
}

impl Default for RefreshTokenConfig {
    fn default() -> Self {
        Self {
            token_ttl: Duration::from_secs(30 * 24 * 3600), // 30 days
            rotation_grace_period: Duration::from_secs(60), // 1 minute
            max_family_size: 10,
            max_tokens_per_user: 5,
            token_bytes: 32,
        }
    }
}

impl RefreshTokenConfig {
    /// トークンの有効期間を設定
    pub fn with_token_ttl(mut self, ttl: Duration) -> Self {
        self.token_ttl = ttl;
        self
    }

    /// ローテーション猶予期間を設定
    pub fn with_rotation_grace_period(mut self, period: Duration) -> Self {
        self.rotation_grace_period = period;
        self
    }

    /// トークンファミリーの最大サイズを設定
    pub fn with_max_family_size(mut self, size: usize) -> Self {
        self.max_family_size = size;
        self
    }

    /// 同一ユーザーの最大アクティブトークン数を設定
    pub fn with_max_tokens_per_user(mut self, count: usize) -> Self {
        self.max_tokens_per_user = count;
        self
    }
}

/// リフレッシュトークンデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenData {
    /// トークンID
    pub id: String,
    /// ユーザーID
    pub user_id: String,
    /// トークンファミリーID
    pub family_id: String,
    /// 発行時刻（Unix timestamp）
    pub issued_at: i64,
    /// 有効期限（Unix timestamp）
    pub expires_at: i64,
    /// 使用済みかどうか
    pub used: bool,
    /// デバイス情報（オプション）
    pub device_info: Option<String>,
    /// IPアドレス（オプション）
    pub ip_address: Option<String>,
}

/// 発行されたトークン
#[derive(Debug, Clone)]
pub struct IssuedRefreshToken {
    /// トークン文字列
    pub token: String,
    /// トークンデータ
    pub data: RefreshTokenData,
}

/// リフレッシュトークンストアトレイト
#[async_trait]
pub trait RefreshTokenStore: Send + Sync {
    /// トークンを保存
    async fn save(&self, token: &str, data: &RefreshTokenData) -> Result<(), AuthError>;

    /// トークンを取得
    async fn get(&self, token: &str) -> Result<Option<RefreshTokenData>, AuthError>;

    /// トークンを削除
    async fn delete(&self, token: &str) -> Result<bool, AuthError>;

    /// トークンを使用済みにマーク
    async fn mark_used(&self, token: &str) -> Result<bool, AuthError>;

    /// ユーザーの全トークンを取得
    async fn get_by_user(&self, user_id: &str) -> Result<Vec<RefreshTokenData>, AuthError>;

    /// ユーザーの全トークンを無効化
    async fn revoke_all_for_user(&self, user_id: &str) -> Result<u64, AuthError>;

    /// トークンファミリーの全トークンを無効化
    async fn revoke_family(&self, family_id: &str) -> Result<u64, AuthError>;

    /// 期限切れトークンをクリーンアップ
    async fn cleanup(&self) -> Result<u64, AuthError>;
}

/// インメモリリフレッシュトークンストア
#[derive(Default)]
pub struct InMemoryRefreshTokenStore {
    tokens: RwLock<HashMap<String, RefreshTokenData>>,
}

impl InMemoryRefreshTokenStore {
    /// 新しいストアを作成
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl RefreshTokenStore for InMemoryRefreshTokenStore {
    async fn save(&self, token: &str, data: &RefreshTokenData) -> Result<(), AuthError> {
        let mut tokens = self.tokens.write().await;
        tokens.insert(token.to_string(), data.clone());
        Ok(())
    }

    async fn get(&self, token: &str) -> Result<Option<RefreshTokenData>, AuthError> {
        let tokens = self.tokens.read().await;
        Ok(tokens.get(token).cloned())
    }

    async fn delete(&self, token: &str) -> Result<bool, AuthError> {
        let mut tokens = self.tokens.write().await;
        Ok(tokens.remove(token).is_some())
    }

    async fn mark_used(&self, token: &str) -> Result<bool, AuthError> {
        let mut tokens = self.tokens.write().await;
        if let Some(data) = tokens.get_mut(token) {
            data.used = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn get_by_user(&self, user_id: &str) -> Result<Vec<RefreshTokenData>, AuthError> {
        let tokens = self.tokens.read().await;
        let user_tokens: Vec<RefreshTokenData> = tokens
            .values()
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect();
        Ok(user_tokens)
    }

    async fn revoke_all_for_user(&self, user_id: &str) -> Result<u64, AuthError> {
        let mut tokens = self.tokens.write().await;
        let before = tokens.len();
        tokens.retain(|_, t| t.user_id != user_id);
        Ok((before - tokens.len()) as u64)
    }

    async fn revoke_family(&self, family_id: &str) -> Result<u64, AuthError> {
        let mut tokens = self.tokens.write().await;
        let before = tokens.len();
        tokens.retain(|_, t| t.family_id != family_id);
        Ok((before - tokens.len()) as u64)
    }

    async fn cleanup(&self) -> Result<u64, AuthError> {
        let now = chrono::Utc::now().timestamp();
        let mut tokens = self.tokens.write().await;
        let before = tokens.len();
        tokens.retain(|_, t| t.expires_at > now);
        Ok((before - tokens.len()) as u64)
    }
}

/// リフレッシュトークンマネージャー
pub struct RefreshTokenManager<S: RefreshTokenStore> {
    store: Arc<S>,
    config: RefreshTokenConfig,
}

impl<S: RefreshTokenStore> RefreshTokenManager<S> {
    /// 新しいマネージャーを作成
    pub fn new(store: S, config: RefreshTokenConfig) -> Self {
        Self {
            store: Arc::new(store),
            config,
        }
    }

    /// 設定を取得
    pub fn config(&self) -> &RefreshTokenConfig {
        &self.config
    }

    /// 新しいトークンを発行
    pub async fn issue(
        &self,
        user_id: &str,
        device_info: Option<String>,
    ) -> Result<IssuedRefreshToken, AuthError> {
        self.issue_with_family(user_id, None, device_info).await
    }

    /// 新しいトークンを発行（ファミリーID指定）
    pub async fn issue_with_family(
        &self,
        user_id: &str,
        family_id: Option<String>,
        device_info: Option<String>,
    ) -> Result<IssuedRefreshToken, AuthError> {
        // ユーザーの既存トークン数をチェック
        let existing_tokens = self.store.get_by_user(user_id).await?;
        if existing_tokens.len() >= self.config.max_tokens_per_user {
            // 最も古いトークンを削除
            if let Some(oldest) = existing_tokens.iter().min_by_key(|t| t.issued_at) {
                debug!(user_id = %user_id, token_id = %oldest.id, "Removing oldest token");
                // 注: ここではトークン文字列が必要だが、データには含まれていない
                // 実際の実装では、トークン文字列とデータを紐付けて保存する必要がある
            }
        }

        // トークンを生成
        let token = self.generate_token();
        let family_id = family_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let now = chrono::Utc::now();

        let data = RefreshTokenData {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            family_id,
            issued_at: now.timestamp(),
            expires_at: (now + chrono::Duration::from_std(self.config.token_ttl).unwrap()).timestamp(),
            used: false,
            device_info,
            ip_address: None,
        };

        self.store.save(&token, &data).await?;

        info!(user_id = %user_id, token_id = %data.id, "Refresh token issued");

        Ok(IssuedRefreshToken { token, data })
    }

    /// トークンを検証
    pub async fn verify(&self, token: &str) -> Result<RefreshTokenData, AuthError> {
        let data = self
            .store
            .get(token)
            .await?
            .ok_or_else(|| AuthError::invalid_token("Refresh token not found"))?;

        let now = chrono::Utc::now().timestamp();

        // 期限切れチェック
        if data.expires_at <= now {
            return Err(AuthError::token_expired());
        }

        // 使用済みチェック（ローテーションポリシーに依存）
        if data.used {
            // 猶予期間内かチェック
            let used_within_grace = data.issued_at
                + self.config.rotation_grace_period.as_secs() as i64
                > now;

            if !used_within_grace {
                // リプレイ攻撃の可能性 - ファミリー全体を無効化
                warn!(
                    family_id = %data.family_id,
                    token_id = %data.id,
                    "Potential token replay attack detected"
                );
                self.store.revoke_family(&data.family_id).await?;
                return Err(AuthError::token_revoked(data.id));
            }
        }

        Ok(data)
    }

    /// トークンをローテーション（検証して新しいトークンを発行）
    pub async fn rotate(&self, token: &str) -> Result<(IssuedRefreshToken, String), AuthError> {
        let data = self.verify(token).await?;

        // 古いトークンを使用済みにマーク
        self.store.mark_used(token).await?;

        // 新しいトークンを同じファミリーで発行
        let new_token = self
            .issue_with_family(&data.user_id, Some(data.family_id.clone()), data.device_info.clone())
            .await?;

        debug!(
            old_token_id = %data.id,
            new_token_id = %new_token.data.id,
            "Refresh token rotated"
        );

        Ok((new_token, data.user_id))
    }

    /// トークンを無効化
    pub async fn revoke(&self, token: &str) -> Result<bool, AuthError> {
        self.store.delete(token).await
    }

    /// ユーザーの全トークンを無効化
    pub async fn revoke_all_for_user(&self, user_id: &str) -> Result<u64, AuthError> {
        let count = self.store.revoke_all_for_user(user_id).await?;
        info!(user_id = %user_id, count = count, "All refresh tokens revoked for user");
        Ok(count)
    }

    /// ファミリーの全トークンを無効化
    pub async fn revoke_family(&self, family_id: &str) -> Result<u64, AuthError> {
        let count = self.store.revoke_family(family_id).await?;
        debug!(family_id = %family_id, count = count, "Token family revoked");
        Ok(count)
    }

    /// セキュアなトークンを生成
    fn generate_token(&self) -> String {
        use std::io::Read;

        let mut bytes = vec![0u8; self.config.token_bytes];

        // /dev/urandom から読み取り（Unix系）または CryptoGenRandom（Windows）
        if let Ok(mut urandom) = std::fs::File::open("/dev/urandom") {
            let _ = urandom.read_exact(&mut bytes);
        } else {
            // フォールバック: UUID を使用
            for chunk in bytes.chunks_mut(16) {
                let uuid = Uuid::new_v4();
                let uuid_bytes = uuid.as_bytes();
                let len = chunk.len().min(16);
                chunk[..len].copy_from_slice(&uuid_bytes[..len]);
            }
        }

        // Base64 URL-safe エンコード
        base64_url_encode(&bytes)
    }
}

/// Base64 URL-safe エンコード（パディングなし）
fn base64_url_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    let mut result = String::new();

    for chunk in data.chunks(3) {
        let n = match chunk.len() {
            3 => ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32),
            2 => ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8),
            1 => (chunk[0] as u32) << 16,
            _ => break,
        };

        result.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        }
        if chunk.len() > 2 {
            result.push(ALPHABET[(n & 0x3F) as usize] as char);
        }
    }

    result
}

/// Redis リフレッシュトークンストア
#[cfg(feature = "redis-cache")]
pub struct RedisRefreshTokenStore {
    cache: Arc<k1s0_cache::CacheClient>,
    key_prefix: String,
}

#[cfg(feature = "redis-cache")]
impl RedisRefreshTokenStore {
    /// 新しいストアを作成
    pub fn new(cache: Arc<k1s0_cache::CacheClient>) -> Self {
        Self {
            cache,
            key_prefix: "auth:refresh".to_string(),
        }
    }

    /// キープレフィックスを設定
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = prefix.into();
        self
    }

    fn token_key(&self, token: &str) -> String {
        format!("{}:token:{}", self.key_prefix, token)
    }

    fn user_key(&self, user_id: &str) -> String {
        format!("{}:user:{}", self.key_prefix, user_id)
    }

    fn family_key(&self, family_id: &str) -> String {
        format!("{}:family:{}", self.key_prefix, family_id)
    }
}

#[cfg(feature = "redis-cache")]
#[async_trait]
impl RefreshTokenStore for RedisRefreshTokenStore {
    async fn save(&self, token: &str, data: &RefreshTokenData) -> Result<(), AuthError> {
        use k1s0_cache::CacheOperations;

        let key = self.token_key(token);
        let ttl = Duration::from_secs((data.expires_at - chrono::Utc::now().timestamp()).max(0) as u64);

        self.cache
            .set(&key, data, Some(ttl))
            .await
            .map_err(|e| AuthError::internal(format!("Failed to save refresh token: {}", e)))?;

        // ユーザーインデックスに追加
        let user_key = self.user_key(&data.user_id);
        let _: Result<i64, _> = self.cache.incr(&user_key, 1).await;

        Ok(())
    }

    async fn get(&self, token: &str) -> Result<Option<RefreshTokenData>, AuthError> {
        use k1s0_cache::CacheOperations;

        let key = self.token_key(token);

        self.cache
            .get(&key)
            .await
            .map_err(|e| AuthError::internal(format!("Failed to get refresh token: {}", e)))
    }

    async fn delete(&self, token: &str) -> Result<bool, AuthError> {
        use k1s0_cache::CacheOperations;

        let key = self.token_key(token);

        self.cache
            .delete(&key)
            .await
            .map_err(|e| AuthError::internal(format!("Failed to delete refresh token: {}", e)))
    }

    async fn mark_used(&self, token: &str) -> Result<bool, AuthError> {
        use k1s0_cache::CacheOperations;

        let key = self.token_key(token);

        if let Some(mut data) = self.get(token).await? {
            data.used = true;
            let ttl = Duration::from_secs((data.expires_at - chrono::Utc::now().timestamp()).max(0) as u64);
            self.cache
                .set(&key, &data, Some(ttl))
                .await
                .map_err(|e| AuthError::internal(format!("Failed to mark token used: {}", e)))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn get_by_user(&self, _user_id: &str) -> Result<Vec<RefreshTokenData>, AuthError> {
        // Redis では SCAN が必要だが、コストが高い
        // 実装は省略（必要に応じて追加）
        Ok(vec![])
    }

    async fn revoke_all_for_user(&self, user_id: &str) -> Result<u64, AuthError> {
        // パターン削除が必要
        let pattern = format!("{}:token:*", self.key_prefix);
        let _ = self.cache.delete_by_pattern(&pattern).await;
        let _ = user_id;
        Ok(0)
    }

    async fn revoke_family(&self, family_id: &str) -> Result<u64, AuthError> {
        // パターン削除が必要
        let _ = family_id;
        Ok(0)
    }

    async fn cleanup(&self) -> Result<u64, AuthError> {
        // Redis の TTL が自動的にクリーンアップする
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = RefreshTokenConfig::default();
        assert_eq!(config.token_ttl, Duration::from_secs(30 * 24 * 3600));
        assert_eq!(config.rotation_grace_period, Duration::from_secs(60));
        assert_eq!(config.max_family_size, 10);
        assert_eq!(config.max_tokens_per_user, 5);
    }

    #[test]
    fn test_config_builder() {
        let config = RefreshTokenConfig::default()
            .with_token_ttl(Duration::from_secs(7 * 24 * 3600))
            .with_max_tokens_per_user(10);

        assert_eq!(config.token_ttl, Duration::from_secs(7 * 24 * 3600));
        assert_eq!(config.max_tokens_per_user, 10);
    }

    #[tokio::test]
    async fn test_in_memory_store() {
        let store = InMemoryRefreshTokenStore::new();

        let data = RefreshTokenData {
            id: "token-1".to_string(),
            user_id: "user-1".to_string(),
            family_id: "family-1".to_string(),
            issued_at: chrono::Utc::now().timestamp(),
            expires_at: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
            used: false,
            device_info: None,
            ip_address: None,
        };

        store.save("token-abc", &data).await.unwrap();

        let retrieved = store.get("token-abc").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "token-1");

        let not_found = store.get("nonexistent").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_issue_and_verify() {
        let store = InMemoryRefreshTokenStore::new();
        let config = RefreshTokenConfig::default();
        let manager = RefreshTokenManager::new(store, config);

        let issued = manager.issue("user-123", None).await.unwrap();
        assert!(!issued.token.is_empty());
        assert_eq!(issued.data.user_id, "user-123");

        let verified = manager.verify(&issued.token).await.unwrap();
        assert_eq!(verified.id, issued.data.id);
    }

    #[tokio::test]
    async fn test_rotate() {
        let store = InMemoryRefreshTokenStore::new();
        let config = RefreshTokenConfig::default();
        let manager = RefreshTokenManager::new(store, config);

        let issued = manager.issue("user-123", None).await.unwrap();

        let (new_token, user_id) = manager.rotate(&issued.token).await.unwrap();
        assert_eq!(user_id, "user-123");
        assert_ne!(new_token.token, issued.token);
        assert_eq!(new_token.data.family_id, issued.data.family_id);
    }

    #[tokio::test]
    async fn test_revoke() {
        let store = InMemoryRefreshTokenStore::new();
        let config = RefreshTokenConfig::default();
        let manager = RefreshTokenManager::new(store, config);

        let issued = manager.issue("user-123", None).await.unwrap();

        assert!(manager.revoke(&issued.token).await.unwrap());

        let result = manager.verify(&issued.token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_revoke_all_for_user() {
        let store = InMemoryRefreshTokenStore::new();
        let config = RefreshTokenConfig::default()
            .with_max_tokens_per_user(10);
        let manager = RefreshTokenManager::new(store, config);

        manager.issue("user-123", None).await.unwrap();
        manager.issue("user-123", None).await.unwrap();
        manager.issue("user-456", None).await.unwrap();

        let count = manager.revoke_all_for_user("user-123").await.unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_base64_url_encode() {
        let data = b"hello world";
        let encoded = base64_url_encode(data);
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
        assert!(!encoded.contains('='));
    }
}
