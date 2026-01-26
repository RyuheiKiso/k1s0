//! PostgreSQL Unit of Work
//!
//! Unit of Work パターンの PostgreSQL 実装を提供する。
//!
//! # Unit of Work パターン
//!
//! Unit of Work は、ビジネストランザクション中のすべての変更を追跡し、
//! 最後にまとめてデータベースにコミットするパターン。
//!
//! # 使用例
//!
//! ```ignore
//! use k1s0_db::uow::PostgresUnitOfWork;
//!
//! let uow = PostgresUnitOfWork::new(&pool).await?;
//!
//! // 複数のリポジトリ操作
//! let user = uow.users().create(new_user).await?;
//! uow.posts().create(new_post).await?;
//!
//! // すべての変更をコミット
//! uow.commit().await?;
//! ```

use crate::error::{DbError, DbResult};
use crate::tx::{IsolationLevel, TransactionOptions, TransactionState, UnitOfWork};
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use uuid::Uuid;

/// PostgreSQL Unit of Work
///
/// PostgreSQL トランザクションを使用した Unit of Work 実装。
#[cfg(feature = "postgres")]
pub struct PostgresUnitOfWork<'a> {
    tx: Option<sqlx::Transaction<'a, sqlx::Postgres>>,
    id: Uuid,
    state: TransactionState,
    options: TransactionOptions,
}

#[cfg(feature = "postgres")]
impl<'a> PostgresUnitOfWork<'a> {
    /// 新しい Unit of Work を作成
    pub async fn new(pool: &'a crate::postgres::PgPool) -> DbResult<Self> {
        Self::with_options(pool, TransactionOptions::default()).await
    }

    /// オプション付きで Unit of Work を作成
    pub async fn with_options(
        pool: &'a crate::postgres::PgPool,
        options: TransactionOptions,
    ) -> DbResult<Self> {
        let tx = pool
            .begin()
            .await
            .map_err(|e| DbError::transaction(format!("failed to begin transaction: {}", e)))?;

        Ok(Self {
            tx: Some(tx),
            id: Uuid::new_v4(),
            state: TransactionState::Active,
            options,
        })
    }

    /// トランザクションへの参照を取得
    pub fn transaction(&mut self) -> DbResult<&mut sqlx::Transaction<'a, sqlx::Postgres>> {
        self.tx
            .as_mut()
            .ok_or_else(|| DbError::transaction("transaction already consumed"))
    }

    /// Unit of Work の ID を取得
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// トランザクションの状態を取得
    pub fn state(&self) -> TransactionState {
        self.state
    }

    /// トランザクションがアクティブかどうか
    pub fn is_active(&self) -> bool {
        self.state == TransactionState::Active
    }

    /// コミット
    pub async fn commit(mut self) -> DbResult<()> {
        if self.state != TransactionState::Active {
            return Err(DbError::transaction(format!(
                "cannot commit: transaction is {:?}",
                self.state
            )));
        }

        if let Some(tx) = self.tx.take() {
            tx.commit()
                .await
                .map_err(|e| DbError::transaction(format!("commit failed: {}", e)))?;
            self.state = TransactionState::Committed;
            tracing::debug!(uow_id = %self.id, "unit of work committed");
        }

        Ok(())
    }

    /// ロールバック
    pub async fn rollback(mut self) -> DbResult<()> {
        if self.state != TransactionState::Active {
            return Err(DbError::transaction(format!(
                "cannot rollback: transaction is {:?}",
                self.state
            )));
        }

        if let Some(tx) = self.tx.take() {
            tx.rollback()
                .await
                .map_err(|e| DbError::transaction(format!("rollback failed: {}", e)))?;
            self.state = TransactionState::RolledBack;
            tracing::debug!(uow_id = %self.id, "unit of work rolled back");
        }

        Ok(())
    }
}

#[cfg(feature = "postgres")]
impl Drop for PostgresUnitOfWork<'_> {
    fn drop(&mut self) {
        if self.tx.is_some() && self.state == TransactionState::Active {
            tracing::warn!(
                uow_id = %self.id,
                "unit of work dropped without commit/rollback, will be rolled back"
            );
        }
    }
}

/// Unit of Work ファクトリー
///
/// Unit of Work インスタンスを作成するファクトリー。
#[cfg(feature = "postgres")]
#[derive(Clone)]
pub struct UnitOfWorkFactory {
    pool: Arc<crate::postgres::PgPool>,
    default_options: TransactionOptions,
}

#[cfg(feature = "postgres")]
impl UnitOfWorkFactory {
    /// 新しいファクトリーを作成
    pub fn new(pool: crate::postgres::PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
            default_options: TransactionOptions::default(),
        }
    }

    /// デフォルトオプション付きでファクトリーを作成
    pub fn with_options(pool: crate::postgres::PgPool, options: TransactionOptions) -> Self {
        Self {
            pool: Arc::new(pool),
            default_options: options,
        }
    }

    /// Unit of Work を作成
    pub async fn create(&self) -> DbResult<PostgresUnitOfWork<'_>> {
        PostgresUnitOfWork::with_options(&self.pool, self.default_options.clone()).await
    }

    /// カスタムオプションで Unit of Work を作成
    pub async fn create_with_options(
        &self,
        options: TransactionOptions,
    ) -> DbResult<PostgresUnitOfWork<'_>> {
        PostgresUnitOfWork::with_options(&self.pool, options).await
    }
}

/// リポジトリコンテキスト
///
/// リポジトリが Unit of Work 内で動作するためのコンテキスト。
#[cfg(feature = "postgres")]
pub struct RepositoryContext<'a> {
    tx: &'a mut sqlx::Transaction<'a, sqlx::Postgres>,
    uow_id: Uuid,
}

#[cfg(feature = "postgres")]
impl<'a> RepositoryContext<'a> {
    /// 新しいコンテキストを作成
    pub fn new(tx: &'a mut sqlx::Transaction<'a, sqlx::Postgres>, uow_id: Uuid) -> Self {
        Self { tx, uow_id }
    }

    /// トランザクションへの参照を取得
    pub fn transaction(&mut self) -> &mut sqlx::Transaction<'a, sqlx::Postgres> {
        self.tx
    }

    /// Unit of Work ID を取得
    pub fn uow_id(&self) -> Uuid {
        self.uow_id
    }
}

/// 非同期クロージャでトランザクションを実行
///
/// # 例
///
/// ```ignore
/// use k1s0_db::uow::execute_in_transaction;
///
/// let result = execute_in_transaction(&pool, |tx| async move {
///     // トランザクション内の処理
///     sqlx::query("INSERT INTO users (name) VALUES ($1)")
///         .bind("Alice")
///         .execute(&mut *tx)
///         .await?;
///     Ok(42)
/// }).await?;
/// ```
#[cfg(feature = "postgres")]
pub async fn execute_in_transaction<F, Fut, T>(
    pool: &crate::postgres::PgPool,
    f: F,
) -> DbResult<T>
where
    F: FnOnce(sqlx::Transaction<'_, sqlx::Postgres>) -> Fut,
    Fut: std::future::Future<Output = Result<T, DbError>>,
{
    let tx = pool
        .begin()
        .await
        .map_err(|e| DbError::transaction(format!("failed to begin transaction: {}", e)))?;

    match f(tx).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e),
    }
}

/// リトライ付きトランザクション実行
///
/// シリアライゼーションエラーが発生した場合にリトライする。
#[cfg(feature = "postgres")]
pub async fn execute_with_retry<F, Fut, T>(
    pool: &crate::postgres::PgPool,
    max_retries: u32,
    f: F,
) -> DbResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = DbResult<T>>,
{
    let mut attempts = 0;

    loop {
        attempts += 1;

        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if is_retryable_error(&e) && attempts < max_retries => {
                tracing::warn!(
                    attempt = attempts,
                    max_retries = max_retries,
                    error = %e,
                    "retryable error, retrying transaction"
                );

                // エクスポネンシャルバックオフ
                let delay = std::time::Duration::from_millis(100 * 2u64.pow(attempts - 1));
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}

/// リトライ可能なエラーかどうかを判定
fn is_retryable_error(error: &DbError) -> bool {
    match error {
        DbError::Transaction { message, .. } => {
            // PostgreSQL のシリアライゼーション失敗
            message.contains("could not serialize")
                || message.contains("deadlock detected")
                || message.contains("40001") // serialization_failure
                || message.contains("40P01") // deadlock_detected
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::TransactionOptions;

    #[test]
    fn test_is_retryable_error() {
        let serialization_error = DbError::transaction("could not serialize access");
        assert!(is_retryable_error(&serialization_error));

        let deadlock_error = DbError::transaction("deadlock detected");
        assert!(is_retryable_error(&deadlock_error));

        let connection_error = DbError::connection("connection refused");
        assert!(!is_retryable_error(&connection_error));
    }

    #[test]
    fn test_transaction_options_default() {
        let options = TransactionOptions::default();
        assert_eq!(options.isolation_level, IsolationLevel::ReadCommitted);
    }
}
