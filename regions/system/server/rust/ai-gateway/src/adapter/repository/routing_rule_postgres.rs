// ルーティングルールのPostgreSQLリポジトリ実装。
// sqlxを使用してルーティングルールを検索する。

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::routing_rule::RoutingRule;
use crate::domain::repository::RoutingRuleRepository;

/// `PostgreSQLベースのルーティングルールリポジトリ`。
pub struct RoutingRulePostgresRepository {
    pool: Option<Arc<PgPool>>,
}

impl RoutingRulePostgresRepository {
    /// データベースプール付きでリポジトリを生成する。
    #[must_use] 
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool: Some(pool) }
    }

    /// インメモリフォールバック用リポジトリを生成する。
    #[must_use] 
    pub fn in_memory() -> Self {
        Self { pool: None }
    }
}

#[async_trait]
impl RoutingRuleRepository for RoutingRulePostgresRepository {
    /// 指定モデルIDに対するアクティブなルーティングルールを取得する。
    /// 優先度が最も高い（数値が最小の）ルールを返す。
    async fn find_active_rule(&self, model_id: &str) -> Option<RoutingRule> {
        if let Some(ref pool) = self.pool {
            let row = sqlx::query_as::<_, RoutingRuleRow>(
                "SELECT id, model_id, priority, strategy FROM ai_routing_rules WHERE model_id = $1 ORDER BY priority ASC LIMIT 1",
            )
            .bind(model_id)
            .fetch_optional(pool.as_ref())
            .await;

            match row {
                Ok(row) => row.map(std::convert::Into::into),
                Err(e) => {
                    tracing::warn!(error = %e, "ルーティングルールのDB取得に失敗");
                    None
                }
            }
        } else {
            None
        }
    }
}

/// データベース行のマッピング用構造体
#[derive(sqlx::FromRow)]
struct RoutingRuleRow {
    id: String,
    model_id: String,
    priority: i32,
    strategy: String,
}

impl From<RoutingRuleRow> for RoutingRule {
    fn from(row: RoutingRuleRow) -> Self {
        RoutingRule::new(row.id, row.model_id, row.priority, row.strategy)
    }
}
