//! PostgreSQL Unit of Work
//!
//! Unit of Work パターンの PostgreSQL 実装を提供する。
//!
//! # Unit of Work パターン
//!
//! Unit of Work は、ビジネストランザクション中のすべての変更を追跡し、
//! 最後にまとめてデータベースにコミットするパターン。
//!
//! # 機能
//!
//! - 複数テーブルにまたがるトランザクション管理
//! - 自動ロールバック（Drop 時にアクティブなトランザクションがあれば）
//! - トランザクション分離レベルの設定
//! - ネストされたセーブポイントのサポート
//! - リトライ付きトランザクション実行
//!
//! # 使用例
//!
//! ## 基本的な使い方
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
//!
//! ## 分離レベルの指定
//!
//! ```ignore
//! use k1s0_db::uow::PostgresUnitOfWork;
//! use k1s0_db::tx::{TransactionOptions, IsolationLevel};
//!
//! let options = TransactionOptions::new()
//!     .with_isolation_level(IsolationLevel::Serializable);
//!
//! let uow = PostgresUnitOfWork::with_options(&pool, options).await?;
//! // ...
//! uow.commit().await?;
//! ```
//!
//! ## セーブポイントの使用
//!
//! ```ignore
//! use k1s0_db::uow::PostgresUnitOfWork;
//!
//! let mut uow = PostgresUnitOfWork::new(&pool).await?;
//!
//! // セーブポイントを作成
//! uow.savepoint("before_risky_operation").await?;
//!
//! match risky_operation(&mut uow).await {
//!     Ok(_) => uow.release_savepoint("before_risky_operation").await?,
//!     Err(_) => uow.rollback_to_savepoint("before_risky_operation").await?,
//! }
//!
//! uow.commit().await?;
//! ```

use crate::error::{DbError, DbResult};
use crate::tx::{IsolationLevel, TransactionMode, TransactionOptions, TransactionState};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

/// PostgreSQL Unit of Work
///
/// PostgreSQL トランザクションを使用した Unit of Work 実装。
/// 複数テーブルにまたがる操作を単一のトランザクションで管理する。
#[cfg(feature = "postgres")]
pub struct PostgresUnitOfWork<'a> {
    /// トランザクション
    tx: Option<sqlx::Transaction<'a, sqlx::Postgres>>,
    /// Unit of Work の一意識別子
    id: Uuid,
    /// トランザクション状態
    state: TransactionState,
    /// トランザクションオプション
    options: TransactionOptions,
    /// アクティブなセーブポイント
    savepoints: HashSet<String>,
    /// 操作カウント（デバッグ/メトリクス用）
    operation_count: u64,
}

#[cfg(feature = "postgres")]
impl<'a> PostgresUnitOfWork<'a> {
    /// 新しい Unit of Work を作成
    pub async fn new(pool: &'a crate::postgres::PgPool) -> DbResult<Self> {
        Self::with_options(pool, TransactionOptions::default()).await
    }

    /// オプション付きで Unit of Work を作成
    ///
    /// # Arguments
    ///
    /// * `pool` - データベースコネクションプール
    /// * `options` - トランザクションオプション（分離レベル、読み取り専用モードなど）
    pub async fn with_options(
        pool: &'a crate::postgres::PgPool,
        options: TransactionOptions,
    ) -> DbResult<Self> {
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| DbError::transaction(format!("failed to begin transaction: {}", e)))?;

        // 分離レベルとトランザクションモードを設定
        Self::set_transaction_characteristics(&mut tx, &options).await?;

        let id = Uuid::new_v4();
        tracing::debug!(
            uow_id = %id,
            isolation_level = ?options.isolation_level,
            mode = ?options.mode,
            "unit of work created"
        );

        Ok(Self {
            tx: Some(tx),
            id,
            state: TransactionState::Active,
            options,
            savepoints: HashSet::new(),
            operation_count: 0,
        })
    }

    /// トランザクション特性を設定
    async fn set_transaction_characteristics(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        options: &TransactionOptions,
    ) -> DbResult<()> {
        use sqlx::Executor;

        // 分離レベルがデフォルト以外、またはモードが ReadOnly の場合のみ SET を実行
        if options.isolation_level != IsolationLevel::ReadCommitted
            || options.mode != TransactionMode::ReadWrite
        {
            let sql = format!(
                "SET TRANSACTION ISOLATION LEVEL {} {}",
                options.isolation_level.as_sql(),
                options.mode.as_sql()
            );

            tx.execute(sql.as_str())
                .await
                .map_err(|e| DbError::transaction(format!("failed to set transaction options: {}", e)))?;
        }

        Ok(())
    }

    /// トランザクションへの参照を取得
    ///
    /// 直接クエリを実行する場合に使用する。
    pub fn transaction(&mut self) -> DbResult<&mut sqlx::Transaction<'a, sqlx::Postgres>> {
        self.ensure_active()?;
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

    /// 操作カウントを取得
    pub fn operation_count(&self) -> u64 {
        self.operation_count
    }

    /// 操作カウントを増加
    pub fn increment_operation_count(&mut self) {
        self.operation_count += 1;
    }

    /// トランザクションオプションを取得
    pub fn options(&self) -> &TransactionOptions {
        &self.options
    }

    /// アクティブ状態を確認
    fn ensure_active(&self) -> DbResult<()> {
        if self.state != TransactionState::Active {
            return Err(DbError::transaction(format!(
                "transaction is not active: {:?}",
                self.state
            )));
        }
        Ok(())
    }

    /// セーブポイントを作成
    ///
    /// トランザクション内で部分的なロールバックポイントを作成する。
    ///
    /// # Arguments
    ///
    /// * `name` - セーブポイント名
    pub async fn savepoint(&mut self, name: &str) -> DbResult<()> {
        use sqlx::Executor;

        self.ensure_active()?;

        if self.savepoints.contains(name) {
            return Err(DbError::transaction(format!(
                "savepoint '{}' already exists",
                name
            )));
        }

        let tx = self.tx.as_mut()
            .ok_or_else(|| DbError::transaction("transaction already consumed"))?;

        let sql = format!("SAVEPOINT {}", Self::sanitize_identifier(name));
        tx.execute(sql.as_str())
            .await
            .map_err(|e| DbError::transaction(format!("failed to create savepoint: {}", e)))?;

        self.savepoints.insert(name.to_string());
        tracing::debug!(uow_id = %self.id, savepoint = %name, "savepoint created");

        Ok(())
    }

    /// セーブポイントにロールバック
    ///
    /// 指定したセーブポイント以降の変更を取り消す。
    /// セーブポイント自体は保持される。
    ///
    /// # Arguments
    ///
    /// * `name` - セーブポイント名
    pub async fn rollback_to_savepoint(&mut self, name: &str) -> DbResult<()> {
        use sqlx::Executor;

        self.ensure_active()?;

        if !self.savepoints.contains(name) {
            return Err(DbError::transaction(format!(
                "savepoint '{}' does not exist",
                name
            )));
        }

        let tx = self.tx.as_mut()
            .ok_or_else(|| DbError::transaction("transaction already consumed"))?;

        let sql = format!("ROLLBACK TO SAVEPOINT {}", Self::sanitize_identifier(name));
        tx.execute(sql.as_str())
            .await
            .map_err(|e| DbError::transaction(format!("failed to rollback to savepoint: {}", e)))?;

        tracing::debug!(uow_id = %self.id, savepoint = %name, "rolled back to savepoint");

        Ok(())
    }

    /// セーブポイントを解放
    ///
    /// セーブポイントを削除し、それ以降のロールバックを不可にする。
    ///
    /// # Arguments
    ///
    /// * `name` - セーブポイント名
    pub async fn release_savepoint(&mut self, name: &str) -> DbResult<()> {
        use sqlx::Executor;

        self.ensure_active()?;

        if !self.savepoints.remove(name) {
            return Err(DbError::transaction(format!(
                "savepoint '{}' does not exist",
                name
            )));
        }

        let tx = self.tx.as_mut()
            .ok_or_else(|| DbError::transaction("transaction already consumed"))?;

        let sql = format!("RELEASE SAVEPOINT {}", Self::sanitize_identifier(name));
        tx.execute(sql.as_str())
            .await
            .map_err(|e| DbError::transaction(format!("failed to release savepoint: {}", e)))?;

        tracing::debug!(uow_id = %self.id, savepoint = %name, "savepoint released");

        Ok(())
    }

    /// 識別子をサニタイズ
    fn sanitize_identifier(name: &str) -> String {
        // 英数字とアンダースコアのみ許可
        name.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    }

    /// コミット
    ///
    /// トランザクション内のすべての変更をデータベースに確定する。
    pub async fn commit(mut self) -> DbResult<()> {
        self.ensure_active()?;

        if let Some(tx) = self.tx.take() {
            tx.commit()
                .await
                .map_err(|e| DbError::transaction(format!("commit failed: {}", e)))?;
            self.state = TransactionState::Committed;
            tracing::debug!(
                uow_id = %self.id,
                operation_count = self.operation_count,
                "unit of work committed"
            );
        }

        Ok(())
    }

    /// ロールバック
    ///
    /// トランザクション内のすべての変更を取り消す。
    pub async fn rollback(mut self) -> DbResult<()> {
        self.ensure_active()?;

        if let Some(tx) = self.tx.take() {
            tx.rollback()
                .await
                .map_err(|e| DbError::transaction(format!("rollback failed: {}", e)))?;
            self.state = TransactionState::RolledBack;
            tracing::debug!(
                uow_id = %self.id,
                operation_count = self.operation_count,
                "unit of work rolled back"
            );
        }

        Ok(())
    }

    /// トランザクション内でクロージャを実行
    ///
    /// クロージャが成功すればコミット、失敗すればロールバックする。
    ///
    /// # Arguments
    ///
    /// * `f` - 実行するクロージャ
    pub async fn execute<F, Fut, T>(mut self, f: F) -> DbResult<T>
    where
        F: FnOnce(&mut sqlx::Transaction<'a, sqlx::Postgres>) -> Fut,
        Fut: std::future::Future<Output = DbResult<T>>,
    {
        let tx = self.tx.as_mut()
            .ok_or_else(|| DbError::transaction("transaction already consumed"))?;

        match f(tx).await {
            Ok(result) => {
                // 成功時はコミット
                if let Some(tx) = self.tx.take() {
                    tx.commit()
                        .await
                        .map_err(|e| DbError::transaction(format!("commit failed: {}", e)))?;
                    self.state = TransactionState::Committed;
                }
                Ok(result)
            }
            Err(e) => {
                // 失敗時はロールバック
                if let Some(tx) = self.tx.take() {
                    let _ = tx.rollback().await;
                    self.state = TransactionState::RolledBack;
                }
                Err(e)
            }
        }
    }
}

#[cfg(feature = "postgres")]
impl Drop for PostgresUnitOfWork<'_> {
    fn drop(&mut self) {
        if self.tx.is_some() && self.state == TransactionState::Active {
            tracing::warn!(
                uow_id = %self.id,
                operation_count = self.operation_count,
                savepoints = ?self.savepoints,
                "unit of work dropped without commit/rollback, will be rolled back"
            );
            // 注意: Drop は同期なので、実際のロールバックは sqlx::Transaction の Drop で行われる
        }
    }
}

/// スコープ付き Unit of Work
///
/// スコープを抜けると自動的にロールバックする Unit of Work。
/// 明示的に `commit()` を呼ばない限りロールバックされる。
#[cfg(feature = "postgres")]
pub struct ScopedUnitOfWork<'a> {
    inner: Option<PostgresUnitOfWork<'a>>,
}

#[cfg(feature = "postgres")]
impl<'a> ScopedUnitOfWork<'a> {
    /// 新しいスコープ付き Unit of Work を作成
    pub async fn new(pool: &'a crate::postgres::PgPool) -> DbResult<Self> {
        Ok(Self {
            inner: Some(PostgresUnitOfWork::new(pool).await?),
        })
    }

    /// オプション付きで作成
    pub async fn with_options(
        pool: &'a crate::postgres::PgPool,
        options: TransactionOptions,
    ) -> DbResult<Self> {
        Ok(Self {
            inner: Some(PostgresUnitOfWork::with_options(pool, options).await?),
        })
    }

    /// 内部の Unit of Work への参照を取得
    pub fn as_mut(&mut self) -> DbResult<&mut PostgresUnitOfWork<'a>> {
        self.inner
            .as_mut()
            .ok_or_else(|| DbError::transaction("unit of work already consumed"))
    }

    /// コミットして Unit of Work を消費
    pub async fn commit(mut self) -> DbResult<()> {
        if let Some(uow) = self.inner.take() {
            uow.commit().await
        } else {
            Err(DbError::transaction("unit of work already consumed"))
        }
    }

    /// 明示的にロールバック
    pub async fn rollback(mut self) -> DbResult<()> {
        if let Some(uow) = self.inner.take() {
            uow.rollback().await
        } else {
            Err(DbError::transaction("unit of work already consumed"))
        }
    }
}

#[cfg(feature = "postgres")]
impl Drop for ScopedUnitOfWork<'_> {
    fn drop(&mut self) {
        if self.inner.is_some() {
            tracing::debug!("ScopedUnitOfWork dropped without commit, will be rolled back");
            // inner の Drop が自動ロールバックを処理
        }
    }
}

/// Unit of Work ファクトリー
///
/// Unit of Work インスタンスを作成するファクトリー。
/// アプリケーション層での DI に適している。
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

    /// プールへの参照を取得
    pub fn pool(&self) -> &crate::postgres::PgPool {
        &self.pool
    }

    /// デフォルトオプションを取得
    pub fn default_options(&self) -> &TransactionOptions {
        &self.default_options
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

    /// スコープ付き Unit of Work を作成
    pub async fn create_scoped(&self) -> DbResult<ScopedUnitOfWork<'_>> {
        ScopedUnitOfWork::with_options(&self.pool, self.default_options.clone()).await
    }

    /// 読み取り専用 Unit of Work を作成
    pub async fn create_read_only(&self) -> DbResult<PostgresUnitOfWork<'_>> {
        let options = TransactionOptions::read_only();
        PostgresUnitOfWork::with_options(&self.pool, options).await
    }

    /// Serializable 分離レベルで Unit of Work を作成
    pub async fn create_serializable(&self) -> DbResult<PostgresUnitOfWork<'_>> {
        let options = TransactionOptions::serializable();
        PostgresUnitOfWork::with_options(&self.pool, options).await
    }

    /// クロージャをトランザクション内で実行
    ///
    /// 成功すればコミット、失敗すればロールバックする。
    pub async fn run<F, Fut, T>(&self, f: F) -> DbResult<T>
    where
        F: FnOnce(&mut sqlx::Transaction<'_, sqlx::Postgres>) -> Fut,
        Fut: std::future::Future<Output = DbResult<T>>,
    {
        let uow = self.create().await?;
        uow.execute(f).await
    }

    /// リトライ付きでクロージャを実行
    pub async fn run_with_retry<F, Fut, T>(&self, max_retries: u32, f: F) -> DbResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = DbResult<T>>,
    {
        execute_with_retry(&self.pool, max_retries, f).await
    }
}

/// リポジトリコンテキスト
///
/// リポジトリが Unit of Work 内で動作するためのコンテキスト。
/// 複数のリポジトリが同じトランザクションを共有する場合に使用する。
#[cfg(feature = "postgres")]
pub struct RepositoryContext<'a> {
    tx: &'a mut sqlx::Transaction<'a, sqlx::Postgres>,
    uow_id: Uuid,
    /// 読み取り専用モードかどうか
    read_only: bool,
}

#[cfg(feature = "postgres")]
impl<'a> RepositoryContext<'a> {
    /// 新しいコンテキストを作成
    pub fn new(tx: &'a mut sqlx::Transaction<'a, sqlx::Postgres>, uow_id: Uuid) -> Self {
        Self {
            tx,
            uow_id,
            read_only: false,
        }
    }

    /// 読み取り専用コンテキストを作成
    pub fn read_only(tx: &'a mut sqlx::Transaction<'a, sqlx::Postgres>, uow_id: Uuid) -> Self {
        Self {
            tx,
            uow_id,
            read_only: true,
        }
    }

    /// トランザクションへの参照を取得
    pub fn transaction(&mut self) -> &mut sqlx::Transaction<'a, sqlx::Postgres> {
        self.tx
    }

    /// Unit of Work ID を取得
    pub fn uow_id(&self) -> Uuid {
        self.uow_id
    }

    /// 読み取り専用モードかどうか
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// 書き込み操作が許可されているか確認
    pub fn ensure_writable(&self) -> DbResult<()> {
        if self.read_only {
            return Err(DbError::transaction(
                "cannot perform write operation in read-only transaction",
            ));
        }
        Ok(())
    }
}

/// マルチテーブル Unit of Work ビルダー
///
/// 複数のリポジトリを登録し、一括でトランザクション内で操作を行う。
#[cfg(feature = "postgres")]
pub struct MultiTableUnitOfWork<'a> {
    uow: PostgresUnitOfWork<'a>,
}

#[cfg(feature = "postgres")]
impl<'a> MultiTableUnitOfWork<'a> {
    /// 新しい MultiTableUnitOfWork を作成
    pub async fn new(pool: &'a crate::postgres::PgPool) -> DbResult<Self> {
        Ok(Self {
            uow: PostgresUnitOfWork::new(pool).await?,
        })
    }

    /// オプション付きで作成
    pub async fn with_options(
        pool: &'a crate::postgres::PgPool,
        options: TransactionOptions,
    ) -> DbResult<Self> {
        Ok(Self {
            uow: PostgresUnitOfWork::with_options(pool, options).await?,
        })
    }

    /// Unit of Work への参照を取得
    pub fn uow(&mut self) -> &mut PostgresUnitOfWork<'a> {
        &mut self.uow
    }

    /// トランザクションへの参照を取得
    pub fn transaction(&mut self) -> DbResult<&mut sqlx::Transaction<'a, sqlx::Postgres>> {
        self.uow.transaction()
    }

    /// セーブポイントを作成
    pub async fn savepoint(&mut self, name: &str) -> DbResult<()> {
        self.uow.savepoint(name).await
    }

    /// セーブポイントにロールバック
    pub async fn rollback_to_savepoint(&mut self, name: &str) -> DbResult<()> {
        self.uow.rollback_to_savepoint(name).await
    }

    /// セーブポイントを解放
    pub async fn release_savepoint(&mut self, name: &str) -> DbResult<()> {
        self.uow.release_savepoint(name).await
    }

    /// 操作カウントを増加
    pub fn record_operation(&mut self) {
        self.uow.increment_operation_count();
    }

    /// コミット
    pub async fn commit(self) -> DbResult<()> {
        self.uow.commit().await
    }

    /// ロールバック
    pub async fn rollback(self) -> DbResult<()> {
        self.uow.rollback().await
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

    #[test]
    fn test_transaction_options_serializable() {
        let options = TransactionOptions::serializable();
        assert_eq!(options.isolation_level, IsolationLevel::Serializable);
    }

    #[test]
    fn test_transaction_options_read_only() {
        let options = TransactionOptions::read_only();
        assert_eq!(options.mode, crate::tx::TransactionMode::ReadOnly);
    }

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(
            PostgresUnitOfWork::sanitize_identifier("valid_name123"),
            "valid_name123"
        );
        assert_eq!(
            PostgresUnitOfWork::sanitize_identifier("name; DROP TABLE users"),
            "nameDROPTABLEusers"
        );
        assert_eq!(
            PostgresUnitOfWork::sanitize_identifier("name'--"),
            "name"
        );
    }

    #[test]
    fn test_retryable_error_codes() {
        // PostgreSQL 40001 = serialization_failure
        let err = DbError::transaction("ERROR: 40001 serialization failure");
        assert!(is_retryable_error(&err));

        // PostgreSQL 40P01 = deadlock_detected
        let err = DbError::transaction("ERROR: 40P01 deadlock detected");
        assert!(is_retryable_error(&err));
    }
}
