use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::{DlqMessage, DlqStatus};
use crate::domain::repository::dlq_message_repository::DlqMessageRepository;

/// `DlqPostgresRepository` は `PostgreSQL` 実装の DLQ メッセージリポジトリ。
pub struct DlqPostgresRepository {
    pool: PgPool,
}

impl DlqPostgresRepository {
    #[must_use] 
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DlqMessageRepository for DlqPostgresRepository {
    /// CRIT-005 対応: トランザクション内で `set_config` を呼び出してテナント分離してから ID で DLQ メッセージを検索する。
    async fn find_by_id(&self, id: Uuid, tenant_id: &str) -> anyhow::Result<Option<DlqMessage>> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let row = sqlx::query_as::<_, DlqMessageRow>(
            r"
            SELECT id, original_topic, error_message, retry_count, max_retries, payload, status, created_at, updated_at, last_retry_at, tenant_id
            FROM dlq.dlq_messages
            WHERE id = $1
            ",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        row.map(std::convert::TryInto::try_into).transpose()
    }

    /// CRIT-005 対応: トランザクション内で `set_config` を呼び出してテナント分離してからトピック別に一覧を取得する。
    async fn find_by_topic(
        &self,
        topic: &str,
        page: i32,
        page_size: i32,
        tenant_id: &str,
    ) -> anyhow::Result<(Vec<DlqMessage>, i64)> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM dlq.dlq_messages WHERE original_topic = $1")
                .bind(topic)
                .fetch_one(&mut *tx)
                .await?;

        let page = page.max(1);
        let page_size = page_size.max(1);
        let offset = i64::from((page - 1) * page_size);

        let rows = sqlx::query_as::<_, DlqMessageRow>(
            r"
            SELECT id, original_topic, error_message, retry_count, max_retries, payload, status, created_at, updated_at, last_retry_at, tenant_id
            FROM dlq.dlq_messages
            WHERE original_topic = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            ",
        )
        .bind(topic)
        .bind(i64::from(page_size))
        .bind(offset)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        let messages: anyhow::Result<Vec<DlqMessage>> =
            rows.into_iter().map(std::convert::TryInto::try_into).collect();

        Ok((messages?, total))
    }

    /// DLQ `メッセージを作成する。tenant_id` はエンティティに含まれるため INSERT に設定する。
    async fn create(&self, message: &DlqMessage) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&message.tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r"
            INSERT INTO dlq.dlq_messages
                (id, original_topic, error_message, retry_count, max_retries, payload, status, created_at, updated_at, last_retry_at, tenant_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ",
        )
        .bind(message.id)
        .bind(&message.original_topic)
        .bind(&message.error_message)
        .bind(message.retry_count)
        .bind(message.max_retries)
        .bind(&message.payload)
        .bind(message.status.to_string())
        .bind(message.created_at)
        .bind(message.updated_at)
        .bind(message.last_retry_at)
        .bind(&message.tenant_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// DLQ `メッセージを更新する。tenant_id` はエンティティに含まれるため WHERE に使用する。
    async fn update(&self, message: &DlqMessage) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&message.tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r"
            UPDATE dlq.dlq_messages
            SET original_topic = $2, error_message = $3, retry_count = $4, max_retries = $5,
                payload = $6, status = $7, updated_at = $8, last_retry_at = $9
            WHERE id = $1 AND tenant_id = $10
            ",
        )
        .bind(message.id)
        .bind(&message.original_topic)
        .bind(&message.error_message)
        .bind(message.retry_count)
        .bind(message.max_retries)
        .bind(&message.payload)
        .bind(message.status.to_string())
        .bind(message.updated_at)
        .bind(message.last_retry_at)
        .bind(&message.tenant_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// CRIT-005 対応: トランザクション内で `set_config` を呼び出してテナント分離してから DLQ メッセージを削除する。
    async fn delete(&self, id: Uuid, tenant_id: &str) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM dlq.dlq_messages WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    /// CRIT-005 対応: トランザクション内で `set_config` を呼び出してテナント分離してからトピック別のメッセージ件数を取得する。
    async fn count_by_topic(&self, topic: &str, tenant_id: &str) -> anyhow::Result<i64> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM dlq.dlq_messages WHERE original_topic = $1")
                .bind(topic)
                .fetch_one(&mut *tx)
                .await?;

        tx.commit().await?;
        Ok(count)
    }
}

/// `DlqMessageRow` はDB行からのマッピング用。
#[derive(sqlx::FromRow)]
struct DlqMessageRow {
    id: Uuid,
    original_topic: String,
    error_message: String,
    retry_count: i32,
    max_retries: i32,
    payload: Option<serde_json::Value>,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    last_retry_at: Option<chrono::DateTime<chrono::Utc>>,
    tenant_id: String,
}

impl TryFrom<DlqMessageRow> for DlqMessage {
    type Error = anyhow::Error;

    fn try_from(row: DlqMessageRow) -> anyhow::Result<Self> {
        Ok(DlqMessage {
            id: row.id,
            original_topic: row.original_topic,
            error_message: row.error_message,
            retry_count: row.retry_count,
            max_retries: row.max_retries,
            payload: row.payload.unwrap_or(serde_json::Value::Null),
            status: DlqStatus::from_str_value(&row.status)?,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_retry_at: row.last_retry_at,
            tenant_id: row.tenant_id,
        })
    }
}
