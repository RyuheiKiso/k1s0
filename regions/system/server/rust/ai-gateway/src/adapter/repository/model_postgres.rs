// AIモデルのPostgreSQLリポジトリ実装。
// sqlxを使用してPostgreSQLからモデル情報を取得する。
// データベース未接続時はインメモリフォールバックを使用する。

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::model::AiModel;
use crate::domain::repository::ModelRepository;

/// PostgreSQLベースのモデルリポジトリ。
/// データベースプールがない場合はデフォルトモデル一覧を返す。
pub struct ModelPostgresRepository {
    pool: Option<Arc<PgPool>>,
}

impl ModelPostgresRepository {
    /// データベースプール付きでリポジトリを生成する。
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool: Some(pool) }
    }

    /// インメモリフォールバック用リポジトリを生成する。
    pub fn in_memory() -> Self {
        Self { pool: None }
    }

    /// デフォルトのモデル一覧を返す（インメモリフォールバック用）。
    fn default_models() -> Vec<AiModel> {
        vec![
            AiModel::new(
                "gpt-4".to_string(),
                "GPT-4".to_string(),
                "openai".to_string(),
                128000,
                true,
                0.03,
                0.06,
            ),
            AiModel::new(
                "gpt-3.5-turbo".to_string(),
                "GPT-3.5 Turbo".to_string(),
                "openai".to_string(),
                16385,
                true,
                0.0005,
                0.0015,
            ),
            AiModel::new(
                "text-embedding-3-small".to_string(),
                "Text Embedding 3 Small".to_string(),
                "openai".to_string(),
                8191,
                true,
                0.00002,
                0.0,
            ),
        ]
    }
}

#[async_trait]
impl ModelRepository for ModelPostgresRepository {
    /// 全モデルの一覧を取得する。
    /// データベース接続がある場合はDBから、ない場合はデフォルトモデルを返す。
    async fn find_all(&self) -> Vec<AiModel> {
        if let Some(ref pool) = self.pool {
            let rows = sqlx::query_as::<_, ModelRow>(
                "SELECT id, name, provider, context_window, enabled, cost_per_1k_input, cost_per_1k_output FROM ai_models",
            )
            .fetch_all(pool.as_ref())
            .await;

            match rows {
                Ok(rows) => rows.into_iter().map(|r| r.into()).collect(),
                Err(e) => {
                    tracing::warn!(error = %e, "モデル一覧のDB取得に失敗、フォールバック使用");
                    Self::default_models()
                }
            }
        } else {
            Self::default_models()
        }
    }

    /// 指定IDのモデルを取得する。
    async fn find_by_id(&self, id: &str) -> Option<AiModel> {
        if let Some(ref pool) = self.pool {
            let row = sqlx::query_as::<_, ModelRow>(
                "SELECT id, name, provider, context_window, enabled, cost_per_1k_input, cost_per_1k_output FROM ai_models WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(pool.as_ref())
            .await;

            match row {
                Ok(row) => row.map(|r| r.into()),
                Err(e) => {
                    tracing::warn!(error = %e, "モデルのDB取得に失敗");
                    Self::default_models().into_iter().find(|m| m.id == id)
                }
            }
        } else {
            Self::default_models().into_iter().find(|m| m.id == id)
        }
    }
}

/// データベース行のマッピング用構造体
#[derive(sqlx::FromRow)]
struct ModelRow {
    id: String,
    name: String,
    provider: String,
    context_window: i32,
    enabled: bool,
    cost_per_1k_input: f64,
    cost_per_1k_output: f64,
}

impl From<ModelRow> for AiModel {
    fn from(row: ModelRow) -> Self {
        AiModel::new(
            row.id,
            row.name,
            row.provider,
            row.context_window,
            row.enabled,
            row.cost_per_1k_input,
            row.cost_per_1k_output,
        )
    }
}
