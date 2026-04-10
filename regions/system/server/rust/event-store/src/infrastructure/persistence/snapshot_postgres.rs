use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::event::Snapshot;
use crate::domain::repository::SnapshotRepository;

/// `SnapshotPostgresRepository` は `PostgreSQL` 実装のスナップショットリポジトリ。
/// テナント分離のため、全クエリの前に `set_config` でテナント ID を設定し RLS を有効化する（ADR-0106）。
pub struct SnapshotPostgresRepository {
    pool: PgPool,
}

impl SnapshotPostgresRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// `PostgreSQL` の snapshots テーブル行をマッピングする内部構造体。
/// `tenant_id` カラムを含む（migration 006 で追加）。
#[derive(sqlx::FromRow)]
struct SnapshotRow {
    id: String,
    stream_id: String,
    tenant_id: String,
    snapshot_version: i64,
    aggregate_type: String,
    state: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<SnapshotRow> for Snapshot {
    fn from(row: SnapshotRow) -> Self {
        Snapshot {
            id: row.id,
            stream_id: row.stream_id,
            tenant_id: row.tenant_id,
            snapshot_version: row.snapshot_version,
            aggregate_type: row.aggregate_type,
            state: row.state,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
impl SnapshotRepository for SnapshotPostgresRepository {
    /// テナント ID を含むスナップショットを INSERT する。
    /// テナント分離のため、INSERT 前にトランザクション内で `set_config` を実行する（ADR-0106）。
    async fn create(&self, snapshot: &Snapshot) -> anyhow::Result<()> {
        // テナントIDをセッション変数に設定して RLS を有効化する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&snapshot.tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r"
            INSERT INTO eventstore.snapshots
                (id, stream_id, tenant_id, snapshot_version, aggregate_type, state, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ",
        )
        .bind(&snapshot.id)
        .bind(&snapshot.stream_id)
        .bind(&snapshot.tenant_id)
        .bind(snapshot.snapshot_version)
        .bind(&snapshot.aggregate_type)
        .bind(&snapshot.state)
        .bind(snapshot.created_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// テナント分離のため、クエリ実行前に `set_config` でテナント ID を設定する。
    async fn find_latest(
        &self,
        tenant_id: &str,
        stream_id: &str,
    ) -> anyhow::Result<Option<Snapshot>> {
        // テナントIDをセッション変数に設定して RLS を有効化する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let row = sqlx::query_as::<_, SnapshotRow>(
            r"
            SELECT id, stream_id, tenant_id, snapshot_version, aggregate_type, state, created_at
            FROM eventstore.snapshots
            WHERE stream_id = $1
            ORDER BY snapshot_version DESC
            LIMIT 1
            ",
        )
        .bind(stream_id)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(row.map(Into::into))
    }

    /// テナント分離のため、クエリ実行前に `set_config` でテナント ID を設定する。
    async fn delete_by_stream(&self, tenant_id: &str, stream_id: &str) -> anyhow::Result<u64> {
        // テナントIDをセッション変数に設定して RLS を有効化する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let result = sqlx::query("DELETE FROM eventstore.snapshots WHERE stream_id = $1")
            .bind(stream_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(result.rows_affected())
    }
}
