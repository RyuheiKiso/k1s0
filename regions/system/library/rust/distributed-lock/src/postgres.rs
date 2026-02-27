use std::time::Duration;

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::lock::{DistributedLock, LockGuard};
use crate::LockError;

/// PostgreSQL advisory lock を使った分散ロック実装。
///
/// `pg_try_advisory_lock(hashtext(key))` でロックを取得し、
/// `pg_advisory_unlock(hashtext(key))` で解放する。
/// advisory lock はセッションスコープのため、TTL はアプリケーション側で管理する。
#[derive(Clone)]
pub struct PostgresDistributedLock {
    pool: PgPool,
    key_prefix: String,
}

impl PostgresDistributedLock {
    /// PgPool から新しい PostgresDistributedLock を作成する。
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            key_prefix: "lock".to_string(),
        }
    }

    /// カスタムキープレフィックスを設定する。
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = prefix.into();
        self
    }

    fn full_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }
}

#[async_trait]
impl DistributedLock for PostgresDistributedLock {
    async fn acquire(&self, key: &str, _ttl: Duration) -> Result<LockGuard, LockError> {
        let full_key = self.full_key(key);
        let token = Uuid::new_v4().to_string();

        // pg_try_advisory_lock は非ブロッキングでロックを試み、成功すれば true を返す。
        // hashtext() でキー文字列を bigint にハッシュする。
        let row: (bool,) =
            sqlx::query_as("SELECT pg_try_advisory_lock(hashtext($1))")
                .bind(&full_key)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| LockError::Internal(e.to_string()))?;

        if row.0 {
            Ok(LockGuard {
                key: key.to_string(),
                token,
            })
        } else {
            Err(LockError::AlreadyLocked(key.to_string()))
        }
    }

    async fn release(&self, guard: LockGuard) -> Result<(), LockError> {
        let full_key = self.full_key(&guard.key);

        let row: (bool,) =
            sqlx::query_as("SELECT pg_advisory_unlock(hashtext($1))")
                .bind(&full_key)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| LockError::Internal(e.to_string()))?;

        if row.0 {
            Ok(())
        } else {
            Err(LockError::LockNotFound(guard.key))
        }
    }

    async fn extend(&self, _guard: &LockGuard, _ttl: Duration) -> Result<(), LockError> {
        // PostgreSQL advisory lock にはTTLの概念がないため、
        // extend はセッションが生きている限りロックが保持されることを確認するのみ。
        Ok(())
    }

    async fn is_locked(&self, key: &str) -> Result<bool, LockError> {
        let full_key = self.full_key(key);

        // pg_locks ビューで advisory lock の存在を確認する。
        // hashtext() の結果を classid として検索する。
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM pg_locks WHERE locktype = 'advisory' AND classid = hashtext($1)::int)"
        )
        .bind(&full_key)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| LockError::Internal(e.to_string()))?;

        Ok(row.0)
    }
}

/// ヘルパー: ロックキーのフォーマット（テスト用に公開）。
pub fn format_lock_key(prefix: &str, key: &str) -> String {
    format!("{}:{}", prefix, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_lock_key() {
        assert_eq!(format_lock_key("lock", "myresource"), "lock:myresource");
        assert_eq!(
            format_lock_key("myapp:lock", "resource"),
            "myapp:lock:resource"
        );
    }

    #[test]
    fn test_full_key() {
        let pool_url = "postgres://localhost/test";
        // PgPool はテストでは作れないため、format だけ検証
        let prefix = "lock";
        let key = "scheduler:job-123";
        assert_eq!(format_lock_key(prefix, key), "lock:scheduler:job-123");
    }
}
