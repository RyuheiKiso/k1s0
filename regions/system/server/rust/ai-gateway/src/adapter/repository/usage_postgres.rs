// 使用量レコードのPostgreSQLリポジトリ実装。
// sqlxを使用して使用量レコードの保存と検索を行う。

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::usage_record::UsageRecord;
use crate::domain::repository::UsageRepository;

/// `PostgreSQLベースの使用量リポジトリ`。
pub struct UsagePostgresRepository {
    pool: Option<Arc<PgPool>>,
}

impl UsagePostgresRepository {
    /// データベースプール付きでリポジトリを生成する。
    #[must_use] 
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool: Some(pool) }
    }

    /// インメモリフォールバック用リポジトリを生成する（保存は無操作）。
    #[must_use] 
    pub fn in_memory() -> Self {
        Self { pool: None }
    }
}

#[async_trait]
impl UsageRepository for UsagePostgresRepository {
    /// 使用量レコードを保存する。
    async fn save(&self, record: &UsageRecord) -> anyhow::Result<()> {
        if let Some(ref pool) = self.pool {
            sqlx::query(
                "INSERT INTO ai_usage_records (id, tenant_id, model_id, prompt_tokens, completion_tokens, cost_usd, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(&record.id)
            .bind(&record.tenant_id)
            .bind(&record.model_id)
            .bind(record.prompt_tokens)
            .bind(record.completion_tokens)
            .bind(record.cost_usd)
            .bind(record.created_at)
            .execute(pool.as_ref())
            .await?;
        } else {
            tracing::debug!(id = %record.id, "インメモリモードのため使用量レコードの保存をスキップ");
        }
        Ok(())
    }

    /// 指定テナントの期間内使用量レコードを取得する。
    async fn find_by_tenant(&self, tenant_id: &str, start: &str, end: &str) -> Vec<UsageRecord> {
        if let Some(ref pool) = self.pool {
            let rows = sqlx::query_as::<_, UsageRow>(
                "SELECT id, tenant_id, model_id, prompt_tokens, completion_tokens, cost_usd, created_at FROM ai_usage_records WHERE tenant_id = $1 AND created_at >= $2::timestamptz AND created_at <= $3::timestamptz ORDER BY created_at DESC",
            )
            .bind(tenant_id)
            .bind(start)
            .bind(end)
            .fetch_all(pool.as_ref())
            .await;

            match rows {
                Ok(rows) => rows.into_iter().map(std::convert::Into::into).collect(),
                Err(e) => {
                    tracing::warn!(error = %e, "使用量レコードのDB取得に失敗");
                    vec![]
                }
            }
        } else {
            vec![]
        }
    }
}

/// データベース行のマッピング用構造体
#[derive(sqlx::FromRow)]
struct UsageRow {
    id: String,
    tenant_id: String,
    model_id: String,
    prompt_tokens: i32,
    completion_tokens: i32,
    cost_usd: f64,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<UsageRow> for UsageRecord {
    fn from(row: UsageRow) -> Self {
        UsageRecord {
            id: row.id,
            tenant_id: row.tenant_id,
            model_id: row.model_id,
            prompt_tokens: row.prompt_tokens,
            completion_tokens: row.completion_tokens,
            cost_usd: row.cost_usd,
            created_at: row.created_at,
        }
    }
}
