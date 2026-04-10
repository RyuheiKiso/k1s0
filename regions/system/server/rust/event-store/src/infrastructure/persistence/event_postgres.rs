use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::event::{EventMetadata, StoredEvent};
use crate::domain::repository::EventRepository;

/// `EventPostgresRepository` は `PostgreSQL` 実装のイベントリポジトリ。
/// テナント分離のため、全クエリの前に `set_config` でテナント ID を設定し RLS を有効化する（ADR-0106）。
pub struct EventPostgresRepository {
    pool: PgPool,
}

impl EventPostgresRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// トランザクション内でイベントを一括INSERTする内部ヘルパー。
    /// 呼び出し元のトランザクション（tx）を受け取り、全件INSERTが成功した場合のみコミットは呼び出し元が行う。
    /// `tenant_id` を INSERT カラムに含め、RLS ポリシーに適合させる。
    pub async fn append_in_tx(
        tenant_id: &str,
        stream_id: &str,
        events: Vec<StoredEvent>,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> anyhow::Result<Vec<StoredEvent>> {
        // トランザクション内でテナントIDを設定し、RLS ポリシーを有効化する
        // lessons.md: set_config は SELECT set_config('app.current_tenant_id', $1, true) パターンを使用する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut **tx)
            .await?;

        let mut result = Vec::with_capacity(events.len());

        for event in events {
            // メタデータをJSONオブジェクトとしてシリアライズする
            let metadata = serde_json::json!({
                "actor_id": event.metadata.actor_id,
                "correlation_id": event.metadata.correlation_id,
                "causation_id": event.metadata.causation_id,
            });
            // トランザクション内でINSERTを実行し、採番されたシーケンスを含む行を返す
            // tenant_id カラムを明示指定してテナント分離を保証する
            let row = sqlx::query_as::<_, StoredEventRow>(
                r"
                INSERT INTO eventstore.events
                    (stream_id, tenant_id, event_type, version, payload, metadata, occurred_at, stored_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
                RETURNING stream_id, tenant_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
                ",
            )
            .bind(stream_id)
            .bind(tenant_id)
            .bind(&event.event_type)
            .bind(event.version)
            .bind(&event.payload)
            .bind(&metadata)
            .bind(event.occurred_at)
            .fetch_one(&mut **tx)
            .await?;

            result.push(row.into());
        }

        Ok(result)
    }
}

/// `PostgreSQL` の events テーブル行をマッピングする内部構造体。
/// `tenant_id` カラムを含む（migration 006 で追加）。
#[derive(sqlx::FromRow)]
struct StoredEventRow {
    stream_id: String,
    tenant_id: String,
    sequence: i64,
    event_type: String,
    version: i64,
    payload: serde_json::Value,
    metadata: serde_json::Value,
    occurred_at: chrono::DateTime<chrono::Utc>,
    stored_at: chrono::DateTime<chrono::Utc>,
}

impl From<StoredEventRow> for StoredEvent {
    fn from(row: StoredEventRow) -> Self {
        let actor_id = row
            .metadata
            .get("actor_id")
            .and_then(|v| v.as_str())
            .map(String::from);
        let correlation_id = row
            .metadata
            .get("correlation_id")
            .and_then(|v| v.as_str())
            .map(String::from);
        let causation_id = row
            .metadata
            .get("causation_id")
            .and_then(|v| v.as_str())
            .map(String::from);
        StoredEvent {
            stream_id: row.stream_id,
            tenant_id: row.tenant_id,
            // LOW-008: 安全な型変換（オーバーフロー防止）
            sequence: u64::try_from(row.sequence).unwrap_or(0),
            event_type: row.event_type,
            version: row.version,
            payload: row.payload,
            metadata: EventMetadata::new(actor_id, correlation_id, causation_id),
            occurred_at: row.occurred_at,
            stored_at: row.stored_at,
        }
    }
}

// EventRepository の実装はクエリ構築・テナント分離・ページング処理を含むため行数が多い
#[allow(clippy::too_many_lines)]
#[async_trait]
impl EventRepository for EventPostgresRepository {
    /// テナント分離のため、トランザクション開始後に `set_config` を呼び出してから INSERT する。
    async fn append(
        &self,
        tenant_id: &str,
        stream_id: &str,
        events: Vec<StoredEvent>,
    ) -> anyhow::Result<Vec<StoredEvent>> {
        // 全イベントのINSERTを単一トランザクションで包み、部分的な書き込みを防止する
        let mut tx = self.pool.begin().await?;

        // テナント分離: トランザクション内でイベントを一括INSERTする（set_config は内部で実行）
        let result = Self::append_in_tx(tenant_id, stream_id, events, &mut tx).await?;

        // 全件INSERT成功後にコミットする
        tx.commit().await?;

        Ok(result)
    }

    /// テナント分離のため、クエリ実行前に `set_config` でテナント ID を設定する。
    async fn find_by_stream(
        &self,
        tenant_id: &str,
        stream_id: &str,
        from_version: i64,
        to_version: Option<i64>,
        event_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<StoredEvent>, u64)> {
        let page = page.max(1);
        let page_size = page_size.clamp(1, 200);
        let offset = i64::from((page - 1) * page_size);

        // テナントIDをセッション変数に設定して RLS を有効化する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        // Build dynamic query for total count
        let total: i64 = if let Some(ref et) = event_type {
            if let Some(tv) = to_version {
                sqlx::query_scalar(
                    r"SELECT COUNT(*) FROM eventstore.events
                       WHERE stream_id = $1 AND version >= $2 AND version <= $3 AND event_type = $4",
                )
                .bind(stream_id)
                .bind(from_version)
                .bind(tv)
                .bind(et)
                .fetch_one(&mut *tx)
                .await?
            } else {
                sqlx::query_scalar(
                    r"SELECT COUNT(*) FROM eventstore.events
                       WHERE stream_id = $1 AND version >= $2 AND event_type = $3",
                )
                .bind(stream_id)
                .bind(from_version)
                .bind(et)
                .fetch_one(&mut *tx)
                .await?
            }
        } else if let Some(tv) = to_version {
            sqlx::query_scalar(
                r"SELECT COUNT(*) FROM eventstore.events
                   WHERE stream_id = $1 AND version >= $2 AND version <= $3",
            )
            .bind(stream_id)
            .bind(from_version)
            .bind(tv)
            .fetch_one(&mut *tx)
            .await?
        } else {
            sqlx::query_scalar(
                r"SELECT COUNT(*) FROM eventstore.events
                   WHERE stream_id = $1 AND version >= $2",
            )
            .bind(stream_id)
            .bind(from_version)
            .fetch_one(&mut *tx)
            .await?
        };

        // Build dynamic query for data
        let rows = if let Some(ref et) = event_type {
            if let Some(tv) = to_version {
                sqlx::query_as::<_, StoredEventRow>(
                    r"SELECT stream_id, tenant_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
                       FROM eventstore.events
                       WHERE stream_id = $1 AND version >= $2 AND version <= $3 AND event_type = $4
                       ORDER BY sequence ASC
                       LIMIT $5 OFFSET $6",
                )
                .bind(stream_id)
                .bind(from_version)
                .bind(tv)
                .bind(et)
                .bind(i64::from(page_size))
                .bind(offset)
                .fetch_all(&mut *tx)
                .await?
            } else {
                sqlx::query_as::<_, StoredEventRow>(
                    r"SELECT stream_id, tenant_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
                       FROM eventstore.events
                       WHERE stream_id = $1 AND version >= $2 AND event_type = $3
                       ORDER BY sequence ASC
                       LIMIT $4 OFFSET $5",
                )
                .bind(stream_id)
                .bind(from_version)
                .bind(et)
                .bind(i64::from(page_size))
                .bind(offset)
                .fetch_all(&mut *tx)
                .await?
            }
        } else if let Some(tv) = to_version {
            sqlx::query_as::<_, StoredEventRow>(
                r"SELECT stream_id, tenant_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
                   FROM eventstore.events
                   WHERE stream_id = $1 AND version >= $2 AND version <= $3
                   ORDER BY sequence ASC
                   LIMIT $4 OFFSET $5",
            )
            .bind(stream_id)
            .bind(from_version)
            .bind(tv)
            .bind(i64::from(page_size))
            .bind(offset)
            .fetch_all(&mut *tx)
            .await?
        } else {
            sqlx::query_as::<_, StoredEventRow>(
                r"SELECT stream_id, tenant_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
                   FROM eventstore.events
                   WHERE stream_id = $1 AND version >= $2
                   ORDER BY sequence ASC
                   LIMIT $3 OFFSET $4",
            )
            .bind(stream_id)
            .bind(from_version)
            .bind(i64::from(page_size))
            .bind(offset)
            .fetch_all(&mut *tx)
            .await?
        };

        tx.commit().await?;

        let events: Vec<StoredEvent> = rows.into_iter().map(Into::into).collect();
        // LOW-008: 安全な型変換（オーバーフロー防止）
        Ok((events, u64::try_from(total).unwrap_or(0)))
    }

    /// テナント分離のため、クエリ実行前に `set_config` でテナント ID を設定する。
    /// RLS が有効なため、設定したテナント ID のイベントのみ返される（全テナント漏洩を防止）。
    async fn find_all(
        &self,
        tenant_id: &str,
        event_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<StoredEvent>, u64)> {
        let page = page.max(1);
        let page_size = page_size.clamp(1, 200);
        let offset = i64::from((page - 1) * page_size);

        // テナントIDをセッション変数に設定して RLS を有効化する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let (total, rows) = if let Some(ref et) = event_type {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM eventstore.events WHERE event_type = $1")
                    .bind(et)
                    .fetch_one(&mut *tx)
                    .await?;

            let rows = sqlx::query_as::<_, StoredEventRow>(
                r"SELECT stream_id, tenant_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
                   FROM eventstore.events
                   WHERE event_type = $1
                   ORDER BY sequence DESC
                   LIMIT $2 OFFSET $3",
            )
            .bind(et)
            .bind(i64::from(page_size))
            .bind(offset)
            .fetch_all(&mut *tx)
            .await?;

            (total, rows)
        } else {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM eventstore.events")
                .fetch_one(&mut *tx)
                .await?;

            let rows = sqlx::query_as::<_, StoredEventRow>(
                r"SELECT stream_id, tenant_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
                   FROM eventstore.events
                   ORDER BY sequence DESC
                   LIMIT $1 OFFSET $2",
            )
            .bind(i64::from(page_size))
            .bind(offset)
            .fetch_all(&mut *tx)
            .await?;

            (total, rows)
        };

        tx.commit().await?;

        let events: Vec<StoredEvent> = rows.into_iter().map(Into::into).collect();
        // LOW-008: 安全な型変換（オーバーフロー防止）
        Ok((events, u64::try_from(total).unwrap_or(0)))
    }

    /// テナント分離のため、クエリ実行前に `set_config` でテナント ID を設定する。
    async fn find_by_sequence(
        &self,
        tenant_id: &str,
        stream_id: &str,
        sequence: u64,
    ) -> anyhow::Result<Option<StoredEvent>> {
        // テナントIDをセッション変数に設定して RLS を有効化する
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let row = sqlx::query_as::<_, StoredEventRow>(
            r"
            SELECT stream_id, tenant_id, sequence, event_type, version, payload, metadata, occurred_at, stored_at
            FROM eventstore.events
            WHERE stream_id = $1 AND sequence = $2
            ",
        )
        .bind(stream_id)
        // LOW-008: 安全な型変換（オーバーフロー防止）
        .bind(i64::try_from(sequence).unwrap_or(i64::MAX))
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

        let result = sqlx::query("DELETE FROM eventstore.events WHERE stream_id = $1")
            .bind(stream_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(result.rows_affected())
    }
}
