pub mod category_repo_impl;
pub mod item_repo_impl;
pub mod tenant_extension_repo_impl;
pub mod version_repo_impl;

use sqlx::PgPool;

/// トランザクションマネージャー。
/// ユースケース層で複数リポジトリにまたがる操作をひとつのトランザクションにまとめるために使用する。
/// 各リポジトリの `*_with_executor` メソッドにトランザクション (&mut *tx) を渡すことで、
/// アトミックな複数テーブル操作を実現する。
///
/// # 使用例
/// ```ignore
/// let tx_mgr = TransactionManager::new(pool.clone());
/// let mut tx = tx_mgr.begin().await?;
///
/// // トランザクション内で複数のリポジトリ操作を実行
/// let item = ItemPostgresRepository::create_with_executor(&mut *tx, category_id, &input, "admin").await?;
/// VersionPostgresRepository::create_with_executor(&mut *tx, item.id, 1, None, Some(data), "admin", None).await?;
///
/// tx.commit().await?;
/// ```
pub struct TransactionManager {
    pool: PgPool,
}

impl TransactionManager {
    /// 新しいトランザクションマネージャーを生成する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 新しいトランザクションを開始する。
    pub async fn begin(&self) -> anyhow::Result<sqlx::Transaction<'_, sqlx::Postgres>> {
        Ok(self.pool.begin().await?)
    }
}
