use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::notification_log::NotificationLog;
use crate::domain::repository::NotificationLogRepository;

/// 通知ログの PostgreSQL リポジトリ実装
/// RLS（Row Level Security）と set_config によるテナント境界を強制する
pub struct NotificationLogPostgresRepository {
    pool: Arc<PgPool>,
}

impl NotificationLogPostgresRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// PostgreSQL セッション変数 app.current_tenant_id を設定して RLS ポリシーを有効化する
    /// set_config の第3引数 true は SET LOCAL（トランザクションスコープ）を意味する
    async fn set_tenant_context(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        tenant_id: &str,
    ) -> anyhow::Result<()> {
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct NotificationLogRow {
    id: String,
    /// テナント識別子
    tenant_id: String,
    channel_id: String,
    template_id: Option<String>,
    recipient: String,
    subject: String,
    body: String,
    status: String,
    retry_count: i32,
    error_message: Option<String>,
    sent_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// NotificationLogRow からドメインエンティティへの変換
impl From<NotificationLogRow> for NotificationLog {
    fn from(r: NotificationLogRow) -> Self {
        NotificationLog {
            id: r.id,
            tenant_id: r.tenant_id,
            channel_id: r.channel_id,
            template_id: r.template_id,
            recipient: r.recipient,
            subject: if r.subject.is_empty() {
                None
            } else {
                Some(r.subject)
            },
            body: r.body,
            status: r.status,
            // LOW-008: 安全な型変換（オーバーフロー防止）
            retry_count: u32::try_from(r.retry_count.max(0)).unwrap_or(0),
            error_message: r.error_message,
            sent_at: r.sent_at,
            created_at: r.created_at,
        }
    }
}

#[async_trait]
impl NotificationLogRepository for NotificationLogPostgresRepository {
    /// テナントコンテキストを設定し、RLS スコープ内で ID による通知ログ検索を行う
    async fn find_by_id(&self, id: &str, tenant_id: &str) -> anyhow::Result<Option<NotificationLog>> {
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, tenant_id).await?;

        let row: Option<NotificationLogRow> = sqlx::query_as(
            "SELECT id, tenant_id, channel_id, template_id, recipient, subject, body, status, retry_count, error_message, sent_at, created_at, updated_at \
             FROM notification.notification_logs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(row.map(Into::into))
    }

    /// テナントコンテキストを設定し、RLS スコープ内でチャンネル ID による通知ログ検索を行う
    async fn find_by_channel_id(&self, channel_id: &str, tenant_id: &str) -> anyhow::Result<Vec<NotificationLog>> {
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, tenant_id).await?;

        let rows: Vec<NotificationLogRow> = sqlx::query_as(
            "SELECT id, tenant_id, channel_id, template_id, recipient, subject, body, status, retry_count, error_message, sent_at, created_at, updated_at \
             FROM notification.notification_logs WHERE channel_id = $1 ORDER BY created_at DESC",
        )
        .bind(channel_id)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// テナントコンテキストを設定し、RLS スコープ内でページネーション付き通知ログ一覧を取得する
    async fn find_all_paginated(
        &self,
        tenant_id: &str,
        page: u32,
        page_size: u32,
        channel_id: Option<String>,
        status: Option<String>,
    ) -> anyhow::Result<(Vec<NotificationLog>, u64)> {
        let offset = i64::from(page.saturating_sub(1) * page_size);
        let limit = i64::from(page_size);

        let mut conditions = Vec::new();
        let mut bind_index = 1u32;

        if channel_id.is_some() {
            conditions.push(format!("channel_id = ${bind_index}"));
            bind_index += 1;
        }
        if status.is_some() {
            conditions.push(format!("status = ${bind_index}"));
            bind_index += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_query =
            format!("SELECT COUNT(*) FROM notification.notification_logs {where_clause}");
        let data_query = format!(
            "SELECT id, tenant_id, channel_id, template_id, recipient, subject, body, status, retry_count, error_message, sent_at, created_at, updated_at \
             FROM notification.notification_logs {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, bind_index, bind_index + 1
        );

        // RLS セッション変数を設定してテナント分離を強制する
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, tenant_id).await?;

        let mut count_q = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(ref v) = channel_id {
            count_q = count_q.bind(v);
        }
        if let Some(ref v) = status {
            count_q = count_q.bind(v);
        }
        let total_count = count_q.fetch_one(&mut *tx).await?;

        let mut data_q = sqlx::query_as::<_, NotificationLogRow>(&data_query);
        if let Some(ref v) = channel_id {
            data_q = data_q.bind(v);
        }
        if let Some(ref v) = status {
            data_q = data_q.bind(v);
        }
        data_q = data_q.bind(limit);
        data_q = data_q.bind(offset);

        let rows: Vec<NotificationLogRow> = data_q.fetch_all(&mut *tx).await?;

        tx.commit().await?;

        Ok((
            rows.into_iter().map(Into::into).collect(),
            // LOW-008: 安全な型変換（オーバーフロー防止）
            u64::try_from(total_count).unwrap_or(0),
        ))
    }

    /// 通知ログを作成する。log.tenant_id を使用して RLS コンテキストを設定する。
    async fn create(&self, log: &NotificationLog) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, &log.tenant_id).await?;

        sqlx::query(
            "INSERT INTO notification.notification_logs \
             (id, tenant_id, channel_id, template_id, recipient, subject, body, status, retry_count, error_message, sent_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        )
        .bind(&log.id)
        .bind(&log.tenant_id)
        .bind(&log.channel_id)
        .bind(&log.template_id)
        .bind(&log.recipient)
        .bind(log.subject.as_deref().unwrap_or(""))
        .bind(&log.body)
        .bind(&log.status)
        // LOW-008: 安全な型変換（オーバーフロー防止）
        .bind(i32::try_from(log.retry_count).unwrap_or(i32::MAX))
        .bind(&log.error_message)
        .bind(log.sent_at)
        .bind(log.created_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// 通知ログを更新する。log.tenant_id を使用して RLS コンテキストを設定する。
    async fn update(&self, log: &NotificationLog) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, &log.tenant_id).await?;

        sqlx::query(
            "UPDATE notification.notification_logs \
             SET status = $2, retry_count = $3, error_message = $4, sent_at = $5, updated_at = NOW() \
             WHERE id = $1",
        )
        .bind(&log.id)
        .bind(&log.status)
        // LOW-008: 安全な型変換（オーバーフロー防止）
        .bind(i32::try_from(log.retry_count).unwrap_or(i32::MAX))
        .bind(&log.error_message)
        .bind(log.sent_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
