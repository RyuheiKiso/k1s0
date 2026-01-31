//! 分散ロックモジュール。
//!
//! DB / Redis バックエンドによる分散ロック、フェンシングトークンによる安全性保証を提供する。

#[cfg(feature = "postgres")]
pub mod db_lock;
pub mod fencing;
pub mod metrics;
#[cfg(feature = "redis-backend")]
pub mod redis_lock;

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::ConsensusResult;

/// 分散ロックの抽象 trait。
#[async_trait]
pub trait DistributedLock: Send + Sync {
    /// ロックの取得を試みる（非ブロッキング）。
    ///
    /// 取得できた場合は `Some(LockGuard)` を返し、
    /// 既に他のホルダーがロックを保持している場合は `None` を返す。
    async fn try_lock(&self, resource: &str, ttl_secs: u64) -> ConsensusResult<Option<LockGuard>>;

    /// ロックを取得する（タイムアウトまでブロッキング）。
    ///
    /// `timeout_ms` ミリ秒以内にロックを取得できない場合は `LockTimeout` エラーを返す。
    async fn lock(
        &self,
        resource: &str,
        ttl_secs: u64,
        timeout_ms: u64,
    ) -> ConsensusResult<LockGuard>;

    /// ロックの TTL を延長する。
    async fn extend(&self, guard: &LockGuard, ttl_secs: u64) -> ConsensusResult<bool>;

    /// ロックを解放する。
    async fn unlock(&self, guard: &LockGuard) -> ConsensusResult<()>;
}

/// ロックの保持情報。
///
/// ロック解放の責務を持つ。`release_fn` が設定されている場合、
/// `release()` メソッドで自動解放が可能。
#[derive(Clone, Serialize, Deserialize)]
pub struct LockGuard {
    /// ロック対象のリソース名。
    pub resource: String,
    /// ロック保持者 ID。
    pub holder_id: String,
    /// フェンシングトークン。
    pub fence_token: u64,
    /// ロックの有効期限。
    pub expires_at: DateTime<Utc>,
    /// 解放用のロック実装への参照（シリアライズ対象外）。
    #[serde(skip)]
    release_fn: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl std::fmt::Debug for LockGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LockGuard")
            .field("resource", &self.resource)
            .field("holder_id", &self.holder_id)
            .field("fence_token", &self.fence_token)
            .field("expires_at", &self.expires_at)
            .field("release_fn", &self.release_fn.is_some())
            .finish()
    }
}

impl LockGuard {
    /// 新しい `LockGuard` を作成する。
    #[must_use]
    pub fn new(
        resource: String,
        holder_id: String,
        fence_token: u64,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            resource,
            holder_id,
            fence_token,
            expires_at,
            release_fn: None,
        }
    }

    /// 自動解放用のコールバックを設定する。
    #[must_use]
    pub fn with_release_fn(mut self, f: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.release_fn = Some(f);
        self
    }

    /// ロックが有効かどうか判定する。
    #[must_use]
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }

    /// ロックを手動で解放する。
    pub fn release(self) {
        if let Some(f) = &self.release_fn {
            f();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_guard_validity() {
        let guard = LockGuard::new(
            "test-resource".into(),
            "node-1".into(),
            1,
            Utc::now() + chrono::Duration::seconds(30),
        );
        assert!(guard.is_valid());
    }

    #[test]
    fn test_lock_guard_expired() {
        let guard = LockGuard::new(
            "test-resource".into(),
            "node-1".into(),
            1,
            Utc::now() - chrono::Duration::seconds(10),
        );
        assert!(!guard.is_valid());
    }

    #[test]
    fn test_lock_guard_release_fn() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = Arc::clone(&released);

        let guard = LockGuard::new(
            "res".into(),
            "holder".into(),
            1,
            Utc::now() + chrono::Duration::seconds(30),
        )
        .with_release_fn(Arc::new(move || {
            released_clone.store(true, Ordering::SeqCst);
        }));

        guard.release();
        assert!(released.load(Ordering::SeqCst));
    }
}
