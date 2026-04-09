use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::notification_template::NotificationTemplate;
use crate::domain::repository::NotificationTemplateRepository;

/// テンプレートの PostgreSQL リポジトリ実装
/// RLS（Row Level Security）と set_config によるテナント境界を強制する
pub struct TemplatePostgresRepository {
    pool: Arc<PgPool>,
}

impl TemplatePostgresRepository {
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

/// DB から取得する生データ構造（sqlx::FromRow で自動マッピング）
#[derive(sqlx::FromRow)]
struct TemplateRow {
    id: String,
    /// テナント識別子
    tenant_id: String,
    name: String,
    channel_type: String,
    subject_template: String,
    body_template: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// TemplateRow からドメインエンティティへの変換
impl From<TemplateRow> for NotificationTemplate {
    fn from(r: TemplateRow) -> Self {
        NotificationTemplate {
            id: r.id,
            tenant_id: r.tenant_id,
            name: r.name,
            channel_type: r.channel_type,
            subject_template: if r.subject_template.is_empty() {
                None
            } else {
                Some(r.subject_template)
            },
            body_template: r.body_template,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[async_trait]
impl NotificationTemplateRepository for TemplatePostgresRepository {
    /// テナントコンテキストを設定し、RLS スコープ内で ID によるテンプレート検索を行う
    async fn find_by_id(&self, id: &str, tenant_id: &str) -> anyhow::Result<Option<NotificationTemplate>> {
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, tenant_id).await?;

        let row: Option<TemplateRow> = sqlx::query_as(
            "SELECT id, tenant_id, name, channel_type, subject_template, body_template, created_at, updated_at \
             FROM notification.templates WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(row.map(Into::into))
    }

    /// テナントコンテキストを設定し、RLS スコープ内で全テンプレートを取得する
    async fn find_all(&self, tenant_id: &str) -> anyhow::Result<Vec<NotificationTemplate>> {
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, tenant_id).await?;

        let rows: Vec<TemplateRow> = sqlx::query_as(
            "SELECT id, tenant_id, name, channel_type, subject_template, body_template, created_at, updated_at \
             FROM notification.templates ORDER BY created_at DESC",
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// テナントコンテキストを設定し、RLS スコープ内でページネーション付きテンプレート一覧を取得する
    async fn find_all_paginated(
        &self,
        tenant_id: &str,
        page: u32,
        page_size: u32,
        channel_type: Option<String>,
    ) -> anyhow::Result<(Vec<NotificationTemplate>, u64)> {
        let offset = i64::from(page.saturating_sub(1) * page_size);
        let limit = i64::from(page_size);

        let mut conditions = Vec::new();
        let mut bind_index = 1u32;

        if channel_type.is_some() {
            conditions.push(format!("channel_type = ${bind_index}"));
            bind_index += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_query = format!("SELECT COUNT(*) FROM notification.templates {where_clause}");
        let data_query = format!(
            "SELECT id, tenant_id, name, channel_type, subject_template, body_template, created_at, updated_at \
             FROM notification.templates {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, bind_index, bind_index + 1
        );

        // RLS セッション変数を設定してテナント分離を強制する
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, tenant_id).await?;

        let mut count_q = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(ref v) = channel_type {
            count_q = count_q.bind(v);
        }
        let total_count = count_q.fetch_one(&mut *tx).await?;

        let mut data_q = sqlx::query_as::<_, TemplateRow>(&data_query);
        if let Some(ref v) = channel_type {
            data_q = data_q.bind(v);
        }
        data_q = data_q.bind(limit);
        data_q = data_q.bind(offset);

        let rows: Vec<TemplateRow> = data_q.fetch_all(&mut *tx).await?;

        tx.commit().await?;

        // LOW-008: 安全な型変換（オーバーフロー防止）
        Ok((
            rows.into_iter().map(Into::into).collect(),
            u64::try_from(total_count).unwrap_or(0),
        ))
    }

    /// テンプレートを作成する。template.tenant_id を使用して RLS コンテキストを設定する。
    async fn create(&self, template: &NotificationTemplate) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, &template.tenant_id).await?;

        sqlx::query(
            "INSERT INTO notification.templates \
             (id, tenant_id, name, channel_type, subject_template, body_template, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(&template.id)
        .bind(&template.tenant_id)
        .bind(&template.name)
        .bind(&template.channel_type)
        .bind(template.subject_template.as_deref().unwrap_or(""))
        .bind(&template.body_template)
        .bind(template.created_at)
        .bind(template.updated_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// テンプレートを更新する。template.tenant_id を使用して RLS コンテキストを設定する。
    async fn update(&self, template: &NotificationTemplate) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, &template.tenant_id).await?;

        sqlx::query(
            "UPDATE notification.templates \
             SET name = $2, subject_template = $3, body_template = $4, updated_at = $5 \
             WHERE id = $1",
        )
        .bind(&template.id)
        .bind(&template.name)
        .bind(template.subject_template.as_deref().unwrap_or(""))
        .bind(&template.body_template)
        .bind(template.updated_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// テナントスコープでテンプレートを削除する
    async fn delete(&self, id: &str, tenant_id: &str) -> anyhow::Result<bool> {
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, tenant_id).await?;

        let result = sqlx::query("DELETE FROM notification.templates WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }
}
