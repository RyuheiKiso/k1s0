use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::event::EventStream;
use crate::domain::repository::EventStreamRepository;

/// StreamPostgresRepository は PostgreSQL 実装のイベントストリームリポジトリ。
/// テナント分離のため、全クエリの前に set_config でテナント ID を設定し RLS を有効化する（ADR-0106）。
pub struct StreamPostgresRepository {
    pool: PgPool,
}

impl StreamPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// トランザクション内でイベントストリームを作成する内部ヘルパー。
    /// 呼び出し元のトランザクション（tx）を受け取り、INSERTを実行する。
    /// tenant_id カラムを明示指定してテナント分離を保証する。
    pub async fn create_in_tx<'a>(
        stream: &EventStream,
        tx: &mut sqlx::Transaction<'a, sqlx::Postgres>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO eventstore.event_streams
                (id, tenant_id, aggregate_type, current_version, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&stream.id)
        .bind(&stream.tenant_id)
        .bind(&stream.aggregate_type)
        .bind(stream.current_version)
        .bind(stream.created_at)
        .bind(stream.updated_at)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// トランザクション内でイベントストリームのバージョンを更新する内部ヘルパー。
    /// 呼び出し元のトランザクション（tx）を受け取り、UPDATEを実行する。
    pub async fn update_version_in_tx<'a>(
        id: &str,
        new_version: i64,
        tx: &mut sqlx::Transaction<'a, sqlx::Postgres>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE eventstore.event_streams
            SET current_version = $2, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(new_version)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

/// PostgreSQL の event_streams テーブル行をマッピングする内部構造体。
/// tenant_id カラムを含む（migration 006 で追加）。
#[derive(sqlx::FromRow)]
struct EventStreamRow {
    id: String,
    tenant_id: String,
    aggregate_type: String,
    current_version: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<EventStreamRow> for EventStream {
    fn from(row: EventStreamRow) -> Self {
        EventStream {
            id: row.id,
            tenant_id: row.tenant_id,
            aggregate_type: row.aggregate_type,
            current_version: row.current_version,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl EventStreamRepository for StreamPostgresRepository {
    /// テナント分離のため、クエリ実行前に set_config でテナント ID を設定する。
    async fn find_by_id(&self, tenant_id: &str, id: &str) -> anyhow::Result<Option<EventStream>> {
        // テナントIDをセッション変数に設定して RLS を有効化する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let row = sqlx::query_as::<_, EventStreamRow>(
            r#"
            SELECT id, tenant_id, aggregate_type, current_version, created_at, updated_at
            FROM eventstore.event_streams
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(row.map(Into::into))
    }

    /// テナント分離のため、クエリ実行前に set_config でテナント ID を設定する。
    async fn list_all(
        &self,
        tenant_id: &str,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<EventStream>, u64)> {
        // テナントIDをセッション変数に設定して RLS を有効化する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM eventstore.event_streams")
                .fetch_one(&mut *tx)
                .await?;

        let page = page.max(1);
        let page_size = page_size.clamp(1, 200);
        let offset = ((page - 1) * page_size) as i64;

        let rows = sqlx::query_as::<_, EventStreamRow>(
            r#"
            SELECT id, tenant_id, aggregate_type, current_version, created_at, updated_at
            FROM eventstore.event_streams
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(page_size as i64)
        .bind(offset)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        let streams: Vec<EventStream> = rows.into_iter().map(Into::into).collect();
        Ok((streams, total as u64))
    }

    /// テナント ID を含む EventStream を INSERT する。
    /// tenant_id カラムはエンティティから取得して挿入する（テナント分離保証）。
    async fn create(&self, stream: &EventStream) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO eventstore.event_streams
                (id, tenant_id, aggregate_type, current_version, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&stream.id)
        .bind(&stream.tenant_id)
        .bind(&stream.aggregate_type)
        .bind(stream.current_version)
        .bind(stream.created_at)
        .bind(stream.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// テナント分離のため、クエリ実行前に set_config でテナント ID を設定する。
    async fn update_version(
        &self,
        tenant_id: &str,
        id: &str,
        new_version: i64,
    ) -> anyhow::Result<()> {
        // テナントIDをセッション変数に設定して RLS を有効化する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
            UPDATE eventstore.event_streams
            SET current_version = $2, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(new_version)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// テナント分離のため、クエリ実行前に set_config でテナント ID を設定する。
    async fn delete(&self, tenant_id: &str, id: &str) -> anyhow::Result<bool> {
        // テナントIDをセッション変数に設定して RLS を有効化する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let result = sqlx::query("DELETE FROM eventstore.event_streams WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(result.rows_affected() > 0)
    }
}
